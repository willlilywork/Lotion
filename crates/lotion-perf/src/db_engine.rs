//! T0-R7 Spike B: database engine at scale.
//!
//! A minimal, generic version of the engine 03-DATA-MODEL describes: computed properties
//! (rollups and formulas) materialized in `computed_value`, a property dependency DAG in
//! `prop_dependency` (cycles rejected), and dirty-set recompute that walks the DAG in
//! topological order, following `relation_edge` in reverse for rollups. Formulas here are
//! native closures standing in for the compiled formula language (Phase 3); what the spike
//! measures is storage, dependency walking and the view query, not formula parsing.

use crate::dataset::Rng;
use rusqlite::{Connection, OptionalExtension, params, params_from_iter};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Empty,
    Num(f64),
    Bool(bool),
    Date(i64),
    Text(String),
}

impl Val {
    fn num(&self) -> Option<f64> {
        match self {
            Val::Num(n) => Some(*n),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Agg {
    Sum,
    Count,
    MaxDate,
}

#[derive(Clone)]
pub enum Def {
    Rollup {
        relation: String,
        target: String,
        target_computed: bool,
        agg: Agg,
    },
    Formula {
        inputs: Vec<(String, bool)>,
        f: fn(&[Val]) -> Val,
    },
}

#[derive(Clone)]
pub struct Computed {
    pub id: String,
    pub db: String,
    pub def: Def,
}

impl Computed {
    /// (property it reads, relation it reads through; None = same row)
    fn deps(&self) -> Vec<(&str, Option<&str>)> {
        match &self.def {
            // a count reads only the edges; edge changes are outside this spike
            Def::Rollup {
                agg: Agg::Count, ..
            } => vec![],
            Def::Rollup {
                relation, target, ..
            } => vec![(target.as_str(), Some(relation.as_str()))],
            Def::Formula { inputs, .. } => inputs.iter().map(|(p, _)| (p.as_str(), None)).collect(),
        }
    }
}

/// Computed properties in topological order.
pub struct Engine {
    pub computed: Vec<Computed>,
}

impl Engine {
    /// Orders by dependencies; a cycle is rejected with its members.
    pub fn new(mut pending: Vec<Computed>) -> Result<Self, String> {
        let ids: BTreeSet<String> = pending.iter().map(|c| c.id.clone()).collect();
        let mut done: BTreeSet<String> = BTreeSet::new();
        let mut computed = Vec::with_capacity(pending.len());
        while !pending.is_empty() {
            let before = pending.len();
            let mut rest = Vec::new();
            for c in pending {
                let ready = c
                    .deps()
                    .iter()
                    .all(|(p, _)| !ids.contains(*p) || done.contains(*p));
                if ready {
                    done.insert(c.id.clone());
                    computed.push(c);
                } else {
                    rest.push(c);
                }
            }
            if rest.len() == before {
                let cycle: Vec<&str> = rest.iter().map(|c| c.id.as_str()).collect();
                return Err(format!("dependency cycle among {cycle:?}"));
            }
            pending = rest;
        }
        Ok(Self { computed })
    }

    fn eval(&self, conn: &Connection, c: &Computed, row: &str) -> rusqlite::Result<Val> {
        match &c.def {
            Def::Rollup {
                relation,
                target,
                target_computed,
                agg,
            } => {
                let table = if *target_computed {
                    "computed_value"
                } else {
                    "prop_value"
                };
                let sql = match agg {
                    Agg::Count => {
                        "SELECT count(*) FROM relation_edge WHERE property_id=?1 AND from_row=?2 AND ?3 IS NOT NULL"
                            .to_owned()
                    }
                    Agg::Sum => format!(
                        "SELECT total(v.v_num) FROM relation_edge e JOIN {table} v ON v.row_id=e.to_row AND v.property_id=?3
                         WHERE e.property_id=?1 AND e.from_row=?2"
                    ),
                    Agg::MaxDate => format!(
                        "SELECT max(v.v_date_start) FROM relation_edge e JOIN {table} v ON v.row_id=e.to_row AND v.property_id=?3
                         WHERE e.property_id=?1 AND e.from_row=?2"
                    ),
                };
                let mut st = conn.prepare_cached(&sql)?;
                Ok(match agg {
                    Agg::Count => Val::Num(
                        st.query_row(params![relation, row, target], |r| r.get::<_, i64>(0))?
                            as f64,
                    ),
                    Agg::Sum => {
                        Val::Num(st.query_row(params![relation, row, target], |r| r.get(0))?)
                    }
                    Agg::MaxDate => st
                        .query_row(params![relation, row, target], |r| {
                            r.get::<_, Option<i64>>(0)
                        })?
                        .map_or(Val::Empty, Val::Date),
                })
            }
            Def::Formula { inputs, f } => {
                let mut vals = Vec::with_capacity(inputs.len());
                for (p, computed) in inputs {
                    vals.push(read(conn, row, p, *computed)?);
                }
                Ok(f(&vals))
            }
        }
    }

    /// Recompute one column for every row of its database (definition change / first build).
    pub fn materialize_all(&self, conn: &Connection) -> rusqlite::Result<usize> {
        let tx = conn.unchecked_transaction()?;
        let mut n = 0;
        for c in &self.computed {
            for row in rows_of(&tx, &c.db)? {
                let v = self.eval(&tx, c, &row)?;
                write(&tx, &row, &c.id, &v)?;
                n += 1;
            }
        }
        tx.commit()?;
        Ok(n)
    }

    /// Set a number cell and recompute its dirty set in the same transaction.
    /// Returns the number of computed values rewritten.
    pub fn edit_number(
        &self,
        conn: &Connection,
        row: &str,
        prop: &str,
        v: f64,
    ) -> rusqlite::Result<usize> {
        let tx = conn.unchecked_transaction()?;
        tx.prepare_cached("UPDATE prop_value SET v_num=?3 WHERE row_id=?1 AND property_id=?2")?
            .execute(params![row, prop, v])?;
        let mut dirty: HashMap<String, BTreeSet<String>> = HashMap::new();
        dirty.insert(prop.to_owned(), BTreeSet::from([row.to_owned()]));
        let mut n = 0;
        for c in &self.computed {
            let mut rows = BTreeSet::new();
            for (dep, via) in c.deps() {
                let Some(changed) = dirty.get(dep) else {
                    continue;
                };
                match via {
                    None => rows.extend(changed.iter().cloned()),
                    Some(rel) => {
                        let mut st = tx.prepare_cached(
                            "SELECT from_row FROM relation_edge WHERE property_id=?1 AND to_row=?2",
                        )?;
                        for r in changed {
                            for from in st.query_map(params![rel, r], |x| x.get::<_, String>(0))? {
                                rows.insert(from?);
                            }
                        }
                    }
                }
            }
            if rows.is_empty() {
                continue;
            }
            for r in &rows {
                let v = self.eval(&tx, c, r)?;
                write(&tx, r, &c.id, &v)?;
            }
            n += rows.len();
            dirty.insert(c.id.clone(), rows);
        }
        tx.commit()?;
        Ok(n)
    }

    /// `verify-computed`: recompute everything from scratch and count mismatches.
    pub fn verify(&self, conn: &Connection) -> rusqlite::Result<usize> {
        let mut bad = 0;
        for c in &self.computed {
            for row in rows_of(conn, &c.db)? {
                if self.eval(conn, c, &row)? != read(conn, &row, &c.id, true)? {
                    bad += 1;
                }
            }
        }
        Ok(bad)
    }
}

fn rows_of(conn: &Connection, db: &str) -> rusqlite::Result<Vec<String>> {
    conn.prepare_cached("SELECT id FROM block WHERE parent_id=? AND type='page' ORDER BY sort_key")?
        .query_map([db], |r| r.get(0))?
        .collect()
}

fn read(conn: &Connection, row: &str, prop: &str, computed: bool) -> rusqlite::Result<Val> {
    let sql = if computed {
        "SELECT v_bool,v_num,v_date_start,v_text FROM computed_value WHERE row_id=? AND property_id=?"
    } else {
        "SELECT v_bool,v_num,v_date_start,v_text FROM prop_value WHERE row_id=? AND property_id=?"
    };
    let v = conn
        .prepare_cached(sql)?
        .query_row([row, prop], |r| {
            Ok(if let Some(b) = r.get::<_, Option<i64>>(0)? {
                Val::Bool(b != 0)
            } else if let Some(n) = r.get::<_, Option<f64>>(1)? {
                Val::Num(n)
            } else if let Some(d) = r.get::<_, Option<i64>>(2)? {
                Val::Date(d)
            } else if let Some(t) = r.get::<_, Option<String>>(3)? {
                Val::Text(t)
            } else {
                Val::Empty
            })
        })
        .optional()?;
    Ok(v.unwrap_or(Val::Empty))
}

fn write(conn: &Connection, row: &str, prop: &str, v: &Val) -> rusqlite::Result<()> {
    let (ty, t, n, d, b): (&str, Option<&str>, Option<f64>, Option<i64>, Option<i64>) = match v {
        Val::Empty => ("empty", None, None, None, None),
        Val::Num(x) => ("number", None, Some(*x), None, None),
        Val::Bool(x) => ("boolean", None, None, None, Some(i64::from(*x))),
        Val::Date(x) => ("date", None, None, Some(*x), None),
        Val::Text(x) => ("text", Some(x.as_str()), None, None, None),
    };
    conn.prepare_cached(
        "INSERT INTO computed_value (row_id,property_id,v_type,v_text,v_num,v_date_start,v_bool) VALUES (?,?,?,?,?,?,?)
         ON CONFLICT(row_id,property_id) DO UPDATE SET v_type=excluded.v_type, v_text=excluded.v_text,
           v_num=excluded.v_num, v_date_start=excluded.v_date_start, v_bool=excluded.v_bool",
    )?
    .execute(params![row, prop, ty, t, n, d, b])?;
    Ok(())
}

/// Everything the Spike B probes need, built once on a generated dataset.
pub struct Spike {
    pub engine: Engine,
    pub t_db: String,
    pub hub: String,
    pub s_amount: String,
    pub hub_dependents: usize,
    pub materialized: usize,
    status: String,
    done: String,
    due: String,
    f1: String,
    r1: String,
    f4: String,
}

fn prop_id(conn: &Connection, db: &str, name: &str) -> rusqlite::Result<String> {
    conn.query_row(
        "SELECT id FROM property WHERE database_id=? AND name=?",
        [db, name],
        |r| r.get(0),
    )
}

/// Adds 2 more relations, 3 rollups and 5 formulas to the second generated database (T),
/// all reading the first (S), plus one formula on S; links one S row ("hub") from up to 334
/// T rows so one edit feeds 1 + 3 × 334 = 1,003 computed values across both databases.
pub fn setup(conn: &Connection) -> Result<Spike, Box<dyn std::error::Error>> {
    let db_at = |i: u32| -> rusqlite::Result<String> {
        conn.query_row(
            "SELECT id FROM block WHERE type='database' ORDER BY id LIMIT 1 OFFSET ?",
            [i],
            |r| r.get(0),
        )
    };
    let (s_db, t_db) = (db_at(0)?, db_at(1)?);
    let s_rows = rows_of(conn, &s_db)?;
    let t_rows = rows_of(conn, &t_db)?;
    let s_amount = prop_id(conn, &s_db, "Amount")?;
    let s_due = prop_id(conn, &s_db, "Due")?;
    let link = prop_id(conn, &t_db, "Link")?;
    let (status, done, due, amount) = (
        prop_id(conn, &t_db, "Status")?,
        prop_id(conn, &t_db, "Done")?,
        prop_id(conn, &t_db, "Due")?,
        prop_id(conn, &t_db, "Amount")?,
    );
    let id = |s: &str| format!("spikeB-{s}");
    let tx = conn.unchecked_transaction()?;
    let mut r = Rng::new(42);
    let add_prop = |pid: &str, db: &str, name: &str, ty: &str, cfg: String, i: usize| {
        tx.execute(
            "INSERT INTO property (id,database_id,name,type,config,sort_key) VALUES (?,?,?,?,?,?)",
            params![pid, db, name, ty, cfg, crate::dataset::key(100 + i)],
        )
    };
    let rel_cfg = serde_json::json!({ "target_db": s_db }).to_string();
    for (i, name) in ["Link2", "Link3"].iter().enumerate() {
        let pid = id(name);
        add_prop(&pid, &t_db, name, "relation", rel_cfg.clone(), i)?;
        for t in &t_rows {
            tx.execute(
                "INSERT INTO prop_value (row_id,property_id) VALUES (?,?)",
                [t, &pid],
            )?;
            for j in 0..1 + r.below(3) {
                tx.execute(
                    "INSERT OR IGNORE INTO relation_edge (property_id,from_row,to_row,sort_key) VALUES (?,?,?,?)",
                    params![pid, t, r.pick(&s_rows), crate::dataset::key(j)],
                )?;
            }
        }
    }
    let hub = s_rows[0].clone();
    for t in t_rows.iter().take(334) {
        tx.execute(
            "INSERT OR IGNORE INTO relation_edge (property_id,from_row,to_row,sort_key) VALUES (?,?,?,'z')",
            params![link, t, hub],
        )?;
    }
    let defs = vec![
        Computed {
            id: id("sf"),
            db: s_db.clone(),
            def: Def::Formula {
                inputs: vec![(s_amount.clone(), false)],
                f: |v| Val::Num(v[0].num().unwrap_or(0.0) * 2.0),
            },
        },
        Computed {
            id: id("r1"),
            db: t_db.clone(),
            def: Def::Rollup {
                relation: link.clone(),
                target: s_amount.clone(),
                target_computed: false,
                agg: Agg::Sum,
            },
        },
        Computed {
            id: id("r2"),
            db: t_db.clone(),
            def: Def::Rollup {
                relation: id("Link2"),
                target: s_amount.clone(),
                target_computed: false,
                agg: Agg::Count,
            },
        },
        Computed {
            id: id("r3"),
            db: t_db.clone(),
            def: Def::Rollup {
                relation: id("Link3"),
                target: s_due.clone(),
                target_computed: false,
                agg: Agg::MaxDate,
            },
        },
        Computed {
            id: id("f1"),
            db: t_db.clone(),
            def: Def::Formula {
                inputs: vec![(amount.clone(), false)],
                f: |v| Val::Num(v[0].num().unwrap_or(0.0) * 1.2),
            },
        },
        Computed {
            id: id("f2"),
            db: t_db.clone(),
            def: Def::Formula {
                inputs: vec![(id("r1"), true), (amount.clone(), false)],
                f: |v| Val::Num(v[0].num().unwrap_or(0.0) - v[1].num().unwrap_or(0.0)),
            },
        },
        Computed {
            id: id("f3"),
            db: t_db.clone(),
            def: Def::Formula {
                inputs: vec![(done.clone(), false), (id("f1"), true)],
                f: |v| Val::Bool(v[0] == Val::Bool(true) && v[1].num().unwrap_or(0.0) > 500.0),
            },
        },
        Computed {
            id: id("f4"),
            db: t_db.clone(),
            def: Def::Formula {
                inputs: vec![(id("r2"), true)],
                f: |v| {
                    Val::Text(
                        if v[0].num().unwrap_or(0.0) > 2.0 {
                            "many"
                        } else {
                            "few"
                        }
                        .to_owned(),
                    )
                },
            },
        },
        Computed {
            id: id("f5"),
            db: t_db.clone(),
            def: Def::Formula {
                inputs: vec![(id("f2"), true), (id("f1"), true)],
                f: |v| Val::Num(v[0].num().unwrap_or(0.0) + v[1].num().unwrap_or(0.0)),
            },
        },
    ];
    for (i, c) in defs.iter().enumerate() {
        let (ty, cfg) = match &c.def {
            Def::Rollup { relation, target, agg, .. } => (
                "rollup",
                serde_json::json!({ "relation_prop": relation, "target_prop": target, "fn": format!("{agg:?}") }).to_string(),
            ),
            Def::Formula { .. } => ("formula", r#"{"source":"(spike closure)","time_dependent":false}"#.to_owned()),
        };
        add_prop(&c.id, &c.db, &c.id, ty, cfg, 10 + i)?;
        for (dep, via) in c.deps() {
            tx.execute(
                "INSERT INTO prop_dependency (property_id,depends_on,via_relation) VALUES (?,?,?)",
                params![c.id, dep, via],
            )?;
        }
    }
    tx.commit()?;
    let engine = Engine::new(defs)?;
    let materialized = engine.materialize_all(conn)?;
    let linked: usize = conn.query_row(
        "SELECT count(*) FROM relation_edge WHERE property_id=? AND to_row=?",
        [&link, &hub],
        |r| r.get::<_, i64>(0),
    )? as usize;
    Ok(Spike {
        engine,
        t_db,
        hub,
        s_amount,
        hub_dependents: 1 + 3 * linked,
        materialized,
        status,
        done,
        due,
        f1: id("f1"),
        r1: id("r1"),
        f4: id("f4"),
    })
}

impl Spike {
    /// P-DBVIEW: nested filter + group + 2 sorts over computed columns, first window of 50
    /// with every cell of those rows (values, computed values, relation edges) and group counts.
    pub fn view(&self, conn: &Connection) -> rusqlite::Result<usize> {
        const FROM: &str = "FROM block b
            JOIN prop_option_value so ON so.row_id=b.id AND so.property_id=:status
            JOIN prop_value dn ON dn.row_id=b.id AND dn.property_id=:done
            JOIN prop_value du ON du.row_id=b.id AND du.property_id=:due
            JOIN computed_value f1 ON f1.row_id=b.id AND f1.property_id=:f1
            JOIN computed_value r1 ON r1.row_id=b.id AND r1.property_id=:r1
            JOIN computed_value f4 ON f4.row_id=b.id AND f4.property_id=:f4
            WHERE b.parent_id=:db AND b.type='page'
              AND (dn.v_bool=1 OR f1.v_num > 500) AND (r1.v_num > 100 OR f4.v_text='many')";
        let p = rusqlite::named_params! {
            ":status": self.status, ":done": self.done, ":due": self.due,
            ":f1": self.f1, ":r1": self.r1, ":f4": self.f4, ":db": self.t_db,
        };
        let groups: usize = conn
            .prepare_cached(&format!(
                "SELECT so.option_id, count(*) {FROM} GROUP BY so.option_id"
            ))?
            .query_map(p, |r| r.get::<_, i64>(1))?
            .filter_map(Result::ok)
            .count();
        let ids: Vec<String> = conn
            .prepare_cached(&format!(
                "SELECT b.id {FROM} ORDER BY so.option_id, r1.v_num DESC, du.v_date_start ASC LIMIT 50"
            ))?
            .query_map(p, |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        if ids.is_empty() || groups == 0 {
            return Ok(0);
        }
        let marks = vec!["?"; ids.len()].join(",");
        let mut cells = 0;
        for sql in [
            format!(
                "SELECT property_id, v_text, v_num, v_date_start, v_bool FROM prop_value WHERE row_id IN ({marks})"
            ),
            format!(
                "SELECT property_id, v_text, v_num, v_date_start, v_bool FROM computed_value WHERE row_id IN ({marks})"
            ),
            format!(
                "SELECT property_id, to_row, sort_key, 0, 0 FROM relation_edge WHERE from_row IN ({marks}) ORDER BY sort_key"
            ),
        ] {
            let mut st = conn.prepare_cached(&sql)?;
            let mut rows = st.query(params_from_iter(ids.iter()))?;
            while rows.next()?.is_some() {
                cells += 1;
            }
        }
        Ok(if cells > 0 { ids.len() } else { 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{Scale, generate};

    fn f(_: &[Val]) -> Val {
        Val::Empty
    }

    #[test]
    fn cycles_are_rejected() {
        let c = |id: &str, dep: &str| Computed {
            id: id.into(),
            db: "d".into(),
            def: Def::Formula {
                inputs: vec![(dep.into(), true)],
                f,
            },
        };
        assert!(Engine::new(vec![c("a", "b"), c("b", "a")]).is_err());
        let e = Engine::new(vec![c("b", "a"), c("a", "base")]).unwrap();
        let order: Vec<&str> = e.computed.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(order, ["a", "b"]);
    }

    #[test]
    fn incremental_recompute_matches_full_recompute() {
        let dir = std::env::temp_dir().join(format!("lotion-spikeb-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("t.sqlite");
        let scale = Scale {
            pages: 40,
            blocks: 1_000,
            dbs: 2,
            rows_per_db: 400,
            arabic_share: 0.3,
        };
        generate(&file, scale, 3).unwrap();
        let conn = Connection::open(&file).unwrap();
        let s = setup(&conn).unwrap();
        assert_eq!(s.engine.verify(&conn).unwrap(), 0);
        assert_eq!(s.hub_dependents, 1 + 3 * 334);
        let n = s
            .engine
            .edit_number(&conn, &s.hub, &s.s_amount, 12_345.0)
            .unwrap();
        assert_eq!(n, s.hub_dependents);
        assert_eq!(s.engine.verify(&conn).unwrap(), 0);
        assert!(s.view(&conn).unwrap() > 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
