//! Deterministic synthetic dataset generator (port of `scripts/gen-dataset.ts`).
//! The schema is `lotion_core::SCHEMA_V1`, which a core test keeps equal to 03-DATA-MODEL.

use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy, Debug)]
pub struct Scale {
    /// free-standing pages (database rows are counted separately)
    pub pages: usize,
    /// total blocks including pages, databases and rows
    pub blocks: usize,
    pub dbs: usize,
    pub rows_per_db: usize,
    pub arabic_share: f64,
}

pub const SMOKE: Scale = Scale {
    pages: 500,
    blocks: 10_000,
    dbs: 2,
    rows_per_db: 500,
    arabic_share: 0.3,
};
pub const FULL: Scale = Scale {
    pages: 50_000,
    blocks: 1_000_000,
    dbs: 20,
    rows_per_db: 10_000,
    arabic_share: 0.3,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    pub pages: usize,
    pub blocks: usize,
    pub dbs: usize,
    pub rows: usize,
    pub edges: usize,
}

/// mulberry32: small, fast, deterministic; same sequence as the TypeScript version.
pub struct Rng(u32);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }
    pub fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
    pub fn below(&mut self, n: usize) -> usize {
        ((self.next_f64() * n as f64) as usize).min(n.saturating_sub(1))
    }
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
}

const B62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Evenly spaced fractional keys for generated siblings (real inserts use T1-02).
pub fn key(i: usize) -> String {
    let mut n = i + 1;
    let mut digits = Vec::new();
    while n > 0 {
        digits.push(B62[n % 62]);
        n /= 62;
    }
    digits.reverse();
    let s = String::from_utf8(digits).unwrap_or_default();
    format!("a{s:0>4}")
}

const EN: &[&str] = &[
    "project", "meeting", "notes", "design", "review", "budget", "roadmap", "invoice", "travel",
    "garden", "recipe", "research", "draft", "summary", "decision", "follow", "up", "client",
    "launch",
];
const AR: &[&str] = &[
    "مشروع",
    "اجتماع",
    "ملاحظات",
    "تصميم",
    "مراجعة",
    "ميزانية",
    "خطة",
    "فاتورة",
    "سفر",
    "حديقة",
    "وصفة",
    "بحث",
    "مسودة",
    "ملخص",
    "قرار",
    "متابعة",
    "عميل",
    "إطلاق",
];
const CONTENT_TYPES: &[&str] = &[
    "paragraph",
    "paragraph",
    "paragraph",
    "bulleted",
    "todo",
    "heading2",
    "numbered",
    "quote",
];
const NOW: i64 = 1_763_000_000_000;

struct Gen<'c> {
    tx: &'c Connection,
    r: Rng,
    idn: u64,
    child_count: HashMap<Option<String>, usize>,
    arabic_share: f64,
}

impl Gen<'_> {
    fn id(&mut self) -> String {
        self.idn += 1;
        let mut n = self.idn;
        let mut s = Vec::new();
        while n > 0 {
            s.push(b"0123456789abcdefghijklmnopqrstuvwxyz"[(n % 36) as usize]);
            n /= 36;
        }
        s.reverse();
        format!("{:0>8}", String::from_utf8(s).unwrap_or_default())
    }
    fn text(&mut self) -> String {
        let words = if self.r.next_f64() < self.arabic_share {
            AR
        } else {
            EN
        };
        let n = 3 + self.r.below(12);
        (0..n)
            .map(|_| *self.r.pick(words))
            .collect::<Vec<_>>()
            .join(" ")
    }
    fn block(
        &mut self,
        ty: &str,
        parent: Option<&str>,
        page: Option<&str>,
        t: &str,
        page_dir: Option<&str>,
    ) -> rusqlite::Result<String> {
        let bid = self.id();
        let slot = self
            .child_count
            .entry(parent.map(str::to_owned))
            .or_insert(0);
        let sk = key(*slot);
        *slot += 1;
        let pid = page.unwrap_or(&bid).to_owned();
        let content = serde_json::json!({ "text": [{ "t": t }] }).to_string();
        self.tx
            .prepare_cached(
                "INSERT INTO block (id,type,parent_id,page_id,sort_key,content,plain_text,direction,page_dir,created_at,updated_at)
                 VALUES (?,?,?,?,?,?,?,NULL,?,?,?)",
            )?
            .execute(params![bid, ty, parent, pid, sk, content, t, page_dir, NOW, NOW])?;
        if !t.is_empty() {
            self.tx
                .prepare_cached("INSERT INTO fts (block_id,page_id,text) VALUES (?,?,?)")?
                .execute(params![bid, pid, t])?;
        }
        Ok(bid)
    }
    fn prop(
        &self,
        id: &str,
        db: &str,
        name: &str,
        ty: &str,
        cfg: &str,
        i: usize,
    ) -> rusqlite::Result<()> {
        self.tx
            .prepare_cached("INSERT INTO property (id,database_id,name,type,config,sort_key) VALUES (?,?,?,?,?,?)")?
            .execute(params![id, db, name, ty, cfg, key(i)])?;
        Ok(())
    }
    fn pv(
        &self,
        row: &str,
        prop: &str,
        text: Option<&str>,
        num: Option<f64>,
        date: Option<i64>,
        b: Option<i64>,
    ) -> rusqlite::Result<()> {
        self.tx
            .prepare_cached("INSERT INTO prop_value (row_id,property_id,v_text,v_num,v_date_start,v_bool) VALUES (?,?,?,?,?,?)")?
            .execute(params![row, prop, text, num, date, b])?;
        Ok(())
    }
}

/// Generate a dataset at `out` (overwritten). Returns elapsed ms and counts.
pub fn generate(
    out: &Path,
    scale: Scale,
    seed: u32,
) -> Result<(u128, Counts), Box<dyn std::error::Error>> {
    let t0 = Instant::now();
    for suffix in ["", "-wal", "-shm"] {
        let mut p = out.as_os_str().to_owned();
        p.push(suffix);
        let _ = std::fs::remove_file(p);
    }
    let mut conn = Connection::open(out)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "OFF")?; // generation only
    conn.pragma_update(None, "foreign_keys", "OFF")?; // bulk load; checked at the end
    conn.execute_batch(lotion_core::SCHEMA_V1)?;
    conn.pragma_update(None, "user_version", lotion_core::SCHEMA_VERSION)?;
    let mut counts = Counts::default();
    let tx = conn.transaction()?;
    {
        let mut g = Gen {
            tx: &tx,
            r: Rng::new(seed),
            idn: 0,
            child_count: HashMap::new(),
            arabic_share: scale.arabic_share,
        };
        // page tree: 20 roots, the rest under a random earlier page
        let mut pages: Vec<String> = Vec::with_capacity(scale.pages);
        for i in 0..scale.pages {
            let parent = if i < 20 {
                None
            } else {
                Some(g.r.pick(&pages).clone())
            };
            let ar = g.r.next_f64() < scale.arabic_share;
            let t = g.text();
            let p = g.block("page", parent.as_deref(), None, &t, ar.then_some("auto"))?;
            pages.push(p);
        }
        counts.pages = scale.pages;
        // databases with rows, properties, values, relations to the previous db
        let mut dbs: Vec<(String, Vec<String>)> = Vec::new();
        for d in 0..scale.dbs {
            let host = pages[d % pages.len()].clone();
            let db_id = g.block("database", Some(&host), None, &format!("DB {d}"), None)?;
            let (title, num, date, done, status) = (g.id(), g.id(), g.id(), g.id(), g.id());
            let rel = (d > 0).then(|| g.id());
            g.prop(&title, &db_id, "Name", "title", "{}", 0)?;
            g.prop(&num, &db_id, "Amount", "number", "{}", 1)?;
            g.prop(&date, &db_id, "Due", "date", "{}", 2)?;
            g.prop(&done, &db_id, "Done", "checkbox", "{}", 3)?;
            g.prop(
                &status,
                &db_id,
                "Status",
                "status",
                r#"{"options":["todo","doing","done"]}"#,
                4,
            )?;
            if let Some(rel) = &rel {
                let cfg = serde_json::json!({ "target_db": dbs[d - 1].0 }).to_string();
                g.prop(rel, &db_id, "Link", "relation", &cfg, 5)?;
            }
            let mut rows = Vec::with_capacity(scale.rows_per_db);
            for _ in 0..scale.rows_per_db {
                let t = g.text();
                let row = g.block("page", Some(&db_id), None, &t, None)?;
                g.pv(&row, &title, Some(&t), None, None, None)?;
                let amount = (g.r.next_f64() * 100_000.0).round() / 100.0;
                g.pv(&row, &num, None, Some(amount), None, None)?;
                let due = NOW + g.r.below(365) as i64 * 86_400_000;
                g.pv(&row, &date, None, None, Some(due), None)?;
                let b = i64::from(g.r.next_f64() < 0.5);
                g.pv(&row, &done, None, None, None, Some(b))?;
                g.pv(&row, &status, None, None, None, None)?;
                let opt = *g.r.pick(&["todo", "doing", "done"]);
                tx.prepare_cached(
                    "INSERT INTO prop_option_value (row_id,property_id,option_id) VALUES (?,?,?)",
                )?
                .execute(params![row, status, opt])?;
                if let Some(rel) = &rel {
                    g.pv(&row, rel, None, None, None, None)?;
                    let k = 1 + g.r.below(3);
                    for j in 0..k {
                        let to = g.r.pick(&dbs[d - 1].1).clone();
                        tx.prepare_cached(
                            "INSERT OR IGNORE INTO relation_edge (property_id,from_row,to_row,sort_key) VALUES (?,?,?,?)",
                        )?
                        .execute(params![rel, row, to, key(j)])?;
                        counts.edges += 1;
                    }
                }
                rows.push(row);
            }
            counts.rows += rows.len();
            dbs.push((db_id, rows));
        }
        counts.dbs = scale.dbs;
        // content blocks: spread over free pages (skewed), some nested one level
        let remaining = scale.blocks - scale.pages - scale.dbs - scale.dbs * scale.rows_per_db;
        let mut per_page: HashMap<usize, usize> = HashMap::new();
        let mut last_child: HashMap<usize, String> = HashMap::new();
        for _ in 0..remaining {
            let x = g.r.next_f64();
            let pi = ((x * x * pages.len() as f64) as usize).min(pages.len() - 1);
            *per_page.entry(pi).or_insert(0) += 1;
            let nest = g.r.next_f64() < 0.15 && last_child.contains_key(&pi);
            let parent = if nest {
                last_child[&pi].clone()
            } else {
                pages[pi].clone()
            };
            let ty = *g.r.pick(CONTENT_TYPES);
            let t = g.text();
            let b = g.block(ty, Some(&parent), Some(&pages[pi]), &t, None)?;
            if !nest {
                last_child.insert(pi, b);
            }
        }
        counts.blocks = scale.blocks;
    }
    tx.commit()?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    let violations: i64 =
        conn.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
            r.get(0)
        })?;
    if violations > 0 {
        return Err(format!("foreign_key_check: {violations} violations").into());
    }
    conn.execute_batch("ANALYZE")?;
    Ok((t0.elapsed().as_millis(), counts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_matches_typescript_mulberry32() {
        // first value of mulberry32(1) in the TS generator
        let mut r = Rng::new(1);
        let v = r.next_f64();
        assert!((v - 0.627_073_940_588_161_3).abs() < 1e-12, "{v}");
    }

    #[test]
    fn keys_sort_in_document_order() {
        let ks: Vec<String> = (0..5000).map(key).collect();
        assert!(ks.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(key(0), "a0001");
    }

    #[test]
    fn tiny_dataset_is_consistent_and_deterministic() {
        let dir = std::env::temp_dir().join(format!("lotion-gen-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tiny = Scale {
            pages: 30,
            blocks: 300,
            dbs: 2,
            rows_per_db: 20,
            arabic_share: 0.3,
        };
        let a = dir.join("a.sqlite");
        let b = dir.join("b.sqlite");
        let (_, ca) = generate(&a, tiny, 7).unwrap();
        let (_, cb) = generate(&b, tiny, 7).unwrap();
        assert_eq!(ca, cb);
        let n = |p: &Path| -> i64 {
            Connection::open(p)
                .unwrap()
                .query_row("SELECT count(*) FROM block", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(n(&a), 300);
        let dump = |p: &Path| -> String {
            let c = Connection::open(p).unwrap();
            let mut s = c
                .prepare("SELECT id||type||sort_key||plain_text FROM block ORDER BY id")
                .unwrap();
            s.query_map([], |r| r.get::<_, String>(0))
                .unwrap()
                .map(Result::unwrap)
                .collect()
        };
        assert_eq!(dump(&a), dump(&b));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
