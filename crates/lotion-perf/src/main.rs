//! Lotion perf harness (T0-R6, port of `scripts/perf.ts`).
//!
//! `cargo run -p lotion-perf --release -- --smoke` generates a small dataset and runs every
//! probe (proves the harness works; numbers are not judged). Without `--smoke` it builds the
//! full reference dataset and judges p50 against docs/05-QUALITY.md budgets.
//! `--gen <out.sqlite> [--seed N]` only writes a dataset.

mod dataset;
mod db_engine;

use rusqlite::{Connection, OpenFlags, params};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

type Res<T> = Result<T, Box<dyn std::error::Error>>;

struct Ctx {
    largest_page: String,
    second_db: String,
    seq: u64,
    tmp: PathBuf,
}

struct Probe {
    id: &'static str,
    budget_ms: Option<f64>,
    write: bool,
    reps: usize,
    run: fn(&Connection, &mut Ctx) -> Res<i64>,
}

fn count(conn: &Connection, sql: &str, p: impl rusqlite::Params) -> Res<i64> {
    let mut st = conn.prepare_cached(sql)?;
    let mut rows = st.query(p)?;
    let mut n = 0;
    while rows.next()?.is_some() {
        n += 1;
    }
    Ok(n)
}

fn backup(conn: &Connection, cx: &mut Ctx) -> Res<i64> {
    cx.seq += 1;
    let out = cx.tmp.join(format!("backup-{}.sqlite", cx.seq));
    conn.execute("VACUUM INTO ?", [out.to_string_lossy()])?;
    let ok = {
        let copy = Connection::open_with_flags(&out, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let r: String = copy.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        i64::from(r == "ok")
    };
    std::fs::remove_file(&out)?;
    Ok(ok)
}

fn save(conn: &Connection, cx: &mut Ctx) -> Res<i64> {
    let pg = cx.largest_page.clone();
    let keys: Vec<String> = conn
        .prepare_cached(
            "SELECT sort_key FROM block WHERE parent_id=?1 AND id NOT LIKE 'perf%' ORDER BY sort_key
             LIMIT 2 OFFSET (SELECT count(*) / 2 FROM block WHERE parent_id=?1)",
        )?
        .query_map([&pg], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let [left, right] = keys.as_slice() else {
        return Ok(0);
    };
    // between left and right: same prefix + 'V' + sequence (real algorithm is T1-02)
    cx.seq += 1;
    let sk = format!("{left}V{}", cx.seq);
    let id = format!("perf{}", cx.seq);
    let now = 1_800_000_000_000_i64 + cx.seq as i64;
    let tx = conn.unchecked_transaction()?;
    tx.prepare_cached(
        "INSERT INTO block (id,type,parent_id,page_id,sort_key,content,plain_text,created_at,updated_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
    )?
    .execute(params![id, "paragraph", pg, pg, sk, r#"{"text":[{"t":"مرحبا"}]}"#, "مرحبا", now, now])?;
    tx.prepare_cached("INSERT INTO fts (block_id,page_id,text) VALUES (?,?,?)")?
        .execute(params![id, pg, "مرحبا"])?;
    let payload = serde_json::json!({ "id": id, "parent": pg, "sk": sk }).to_string();
    tx.prepare_cached("INSERT INTO change_log (at,client_op_id,command,payload) VALUES (?,?,?,?)")?
        .execute(params![now, id, "block.insert", payload])?;
    tx.commit()?;
    Ok(i64::from(
        sk.as_str() > left.as_str() && sk.as_str() < right.as_str(),
    ))
}

const PROBES: &[Probe] = &[
    Probe {
        id: "P-BACKUP db share (VACUUM INTO, no ZIP yet)",
        budget_ms: Some(20_000.0),
        write: false,
        reps: 3,
        run: backup,
    },
    Probe {
        id: "P-SAVE db share (sibling insert in largest page + fts + change_log)",
        budget_ms: Some(10.0),
        write: true,
        reps: 15,
        run: save,
    },
    Probe {
        id: "P-SEARCH (FTS prefix, English, first 20)",
        budget_ms: Some(100.0),
        write: false,
        reps: 15,
        run: |c, _| {
            count(
                c,
                "SELECT block_id FROM fts WHERE fts MATCH ? LIMIT 20",
                ["budg*"],
            )
        },
    },
    Probe {
        id: "P-SEARCH (FTS phrase, Arabic, ranked, first 20)",
        budget_ms: Some(100.0),
        write: false,
        reps: 15,
        run: |c, _| {
            count(
                c,
                "SELECT block_id FROM fts WHERE fts MATCH ? ORDER BY rank LIMIT 20",
                ["\"ملاحظات تصميم\""],
            )
        },
    },
    Probe {
        id: "page-load (largest page, all blocks ordered)",
        budget_ms: None, // P-PAGE is a paint budget; this is the DB share
        write: false,
        reps: 15,
        run: |c, cx| {
            count(
                c,
                "SELECT id,parent_id,type,sort_key,content FROM block WHERE page_id=? ORDER BY parent_id,sort_key",
                [&cx.largest_page],
            )
        },
    },
    Probe {
        id: "P-SEARCH (FTS, Arabic term, first 20)",
        budget_ms: Some(100.0),
        write: false,
        reps: 15,
        run: |c, _| {
            count(
                c,
                "SELECT block_id FROM fts WHERE fts MATCH ? LIMIT 20",
                ["ميزانية"],
            )
        },
    },
    Probe {
        id: "db-view (filter+sort, first 50)",
        budget_ms: None, // P-DBVIEW proper (rollups, formulas, group) is T0-R7
        write: false,
        reps: 15,
        run: |c, cx| {
            count(
                c,
                "SELECT b.id, n.v_num FROM block b
                 JOIN property pn ON pn.database_id=b.parent_id AND pn.type='number'
                 JOIN prop_value n ON n.row_id=b.id AND n.property_id=pn.id
                 JOIN property pc ON pc.database_id=b.parent_id AND pc.type='checkbox'
                 JOIN prop_value c ON c.row_id=b.id AND c.property_id=pc.id
                 WHERE b.parent_id=? AND c.v_bool=1 AND n.v_num > 100
                 ORDER BY n.v_num DESC LIMIT 50",
                [&cx.second_db],
            )
        },
    },
];

fn run_probes(file: &Path, tmp: &Path, smoke: bool) -> Res<usize> {
    let ro = Connection::open_with_flags(file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let rw = Connection::open(file)?;
    rw.pragma_update(None, "journal_mode", "WAL")?;
    rw.pragma_update(None, "synchronous", "NORMAL")?;
    rw.pragma_update(None, "foreign_keys", "ON")?;
    // setup lookups are done once so they never count against a probe's time
    let largest_page: String = ro.query_row(
        "SELECT page_id FROM block WHERE type!='page' GROUP BY page_id ORDER BY count(*) DESC LIMIT 1",
        [],
        |r| r.get(0),
    )?;
    let second_db: String = ro.query_row(
        "SELECT id FROM block WHERE type='database' LIMIT 1 OFFSET 1",
        [],
        |r| r.get(0),
    )?;
    let mut cx = Ctx {
        largest_page,
        second_db,
        seq: 0,
        tmp: tmp.to_owned(),
    };
    let mut failed = 0;
    for p in PROBES {
        let conn = if p.write { &rw } else { &ro };
        (p.run)(conn, &mut cx)?; // warm-up, not timed
        let reps = if smoke { 3 } else { p.reps };
        let mut xs = Vec::with_capacity(reps);
        let mut out = 0;
        for _ in 0..reps {
            let t = Instant::now();
            out = (p.run)(conn, &mut cx)?;
            xs.push(t.elapsed().as_secs_f64() * 1000.0);
        }
        xs.sort_by(f64::total_cmp);
        let (p50, max) = (xs[xs.len() / 2], xs[xs.len() - 1]);
        let over = !smoke && p.budget_ms.is_some_and(|b| p50 > b);
        if over {
            failed += 1;
        }
        let budget = p
            .budget_ms
            .map(|b| format!(" (budget {b})"))
            .unwrap_or_default();
        println!(
            "{} {}: p50 {p50:.1} ms, max {max:.1} ms{budget} -> {out}",
            if over { "OVER" } else { "ok  " },
            p.id
        );
        if smoke && out == 0 {
            eprintln!("probe returned nothing: {}", p.id);
            failed += 1;
        }
    }
    failed += spike_b(&ro, &rw, smoke)?;
    failed += spike_c(&rw, tmp, smoke)?;
    Ok(failed)
}

/// Prints one result line; returns 1 when the probe failed.
fn report(id: &str, budget_ms: Option<f64>, mut xs: Vec<f64>, out: usize, smoke: bool) -> usize {
    xs.sort_by(f64::total_cmp);
    let (p50, max) = (xs[xs.len() / 2], xs[xs.len() - 1]);
    let over = !smoke && budget_ms.is_some_and(|b| p50 > b);
    let budget = budget_ms
        .map(|b| format!(" (budget {b})"))
        .unwrap_or_default();
    println!(
        "{} {id}: p50 {p50:.1} ms, max {max:.1} ms{budget} -> {out}",
        if over { "OVER" } else { "ok  " }
    );
    if smoke && out == 0 {
        eprintln!("probe returned nothing: {id}");
    }
    usize::from(over || (smoke && out == 0))
}

/// T0-R7 Spike B: computed values (3 relations, 3 rollups, 5 formulas) on a database of
/// the dataset, the P-DBVIEW query and P-RECOMP dirty-set recompute, then verify-computed.
fn spike_b(ro: &Connection, rw: &Connection, smoke: bool) -> Res<usize> {
    let t = Instant::now();
    let s = db_engine::setup(rw)?;
    println!(
        "spike B setup: {} computed values materialized in {:.0} ms; hub edit feeds {}",
        s.materialized,
        t.elapsed().as_secs_f64() * 1000.0,
        s.hub_dependents
    );
    let reps = if smoke { 3 } else { 15 };
    let mut failed = 0;
    s.view(ro)?;
    let mut xs = Vec::new();
    let mut out = 0;
    for _ in 0..reps {
        let t = Instant::now();
        out = s.view(ro)?;
        xs.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    failed += report(
        "P-DBVIEW (3 relations, 3 rollups, 5 formulas; nested filter + group + 2 sorts; first 50 with cells)",
        Some(150.0),
        xs,
        out,
        smoke,
    );
    let mut xs = Vec::new();
    for i in 0..=reps {
        let t = Instant::now();
        out = s
            .engine
            .edit_number(rw, &s.hub, &s.s_amount, 100.0 + i as f64)?;
        if i > 0 {
            xs.push(t.elapsed().as_secs_f64() * 1000.0); // first edit is the warm-up
        }
    }
    failed += report(
        "P-RECOMP (one cell -> dependents across 2 databases)",
        Some(50.0),
        xs,
        out,
        smoke,
    );
    if out < 1000 {
        eprintln!("P-RECOMP touched {out} values, expected >= 1000");
        failed += 1;
    }
    if smoke {
        let bad = s.engine.verify(rw)?;
        println!("verify-computed: {bad} mismatches");
        failed += usize::from(bad > 0);
    }
    Ok(failed)
}

fn arg_after(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seed = arg_after(&args, "--seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let scale_full = arg_after(&args, "--scale").as_deref() == Some("full");
    if let Some(out) = arg_after(&args, "--gen") {
        let scale = if scale_full {
            dataset::FULL
        } else {
            dataset::SMOKE
        };
        return match dataset::generate(Path::new(&out), scale, seed) {
            Ok((ms, c)) => {
                println!("{c:?} in {ms} ms");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("gen failed: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(out) = arg_after(&args, "--backup-smoke") {
        return exit(backup_smoke(Path::new(&out), seed));
    }
    if let Some(file) = arg_after(&args, "--restore-check") {
        return exit(restore_check(Path::new(&file)));
    }
    let smoke = args.iter().any(|a| a == "--smoke");
    let tmp = std::env::temp_dir().join(format!("lotion-perf-{}", std::process::id()));
    let result = (|| -> Res<usize> {
        std::fs::create_dir_all(&tmp)?;
        let file = tmp.join("ref.sqlite");
        let (ms, c) = dataset::generate(
            &file,
            if smoke { dataset::SMOKE } else { dataset::FULL },
            seed,
        )?;
        println!(
            "dataset {}: {c:?} in {ms} ms",
            if smoke { "smoke" } else { "full" }
        );
        run_probes(&file, &tmp, smoke)
    })();
    let _ = std::fs::remove_dir_all(&tmp);
    match result {
        Ok(0) => ExitCode::SUCCESS,
        Ok(n) => {
            eprintln!("{n} probe(s) failed");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("perf failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// T0-R8 Spike C: `.lotion` backup (VACUUM INTO + ZIP + manifest) and restore of the dataset,
/// then canonical dump equality between the live file and the restored copy.
fn spike_c(rw: &Connection, tmp: &Path, smoke: bool) -> Res<usize> {
    use lotion_core::backup;
    let reps = if smoke { 1 } else { 3 };
    let (mut xb, mut xr) = (Vec::new(), Vec::new());
    let mut last = PathBuf::new();
    for i in 0..reps {
        let file = tmp.join(format!("DataBackup-{i}.lotion"));
        let dest = tmp.join(format!("restored-{i}.sqlite"));
        let t = Instant::now();
        backup::backup(rw, &file, 0)?;
        xb.push(t.elapsed().as_secs_f64() * 1000.0);
        let t = Instant::now();
        backup::restore(&file, &dest)?;
        xr.push(t.elapsed().as_secs_f64() * 1000.0);
        std::fs::remove_file(&file)?;
        if i + 1 < reps {
            std::fs::remove_file(&dest)?;
        } else {
            last = dest;
        }
    }
    let (a, n) = backup::canonical_dump_hash(rw)?;
    let (b, _) = backup::canonical_dump_hash(&Connection::open(&last)?)?;
    let equal = usize::from(a == b);
    let mut failed = report(
        "P-BACKUP (.lotion: VACUUM INTO + manifest + ZIP)",
        Some(20_000.0),
        xb,
        equal,
        smoke,
    );
    failed += report(
        "P-RESTORE (unzip + sha256 + integrity_check + open)",
        Some(30_000.0),
        xr,
        equal,
        smoke,
    );
    println!("canonical dump: {n} rows, live == restored: {}", a == b);
    if a != b {
        failed += 1;
    }
    Ok(failed)
}

fn exit(r: Res<()>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("failed: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Cross-OS round-trip, producer side: smoke dataset -> `<out>` plus `<out>.dump-sha256`.
fn backup_smoke(out: &Path, seed: u32) -> Res<()> {
    let tmp = std::env::temp_dir().join(format!("lotion-bks-{}", std::process::id()));
    std::fs::create_dir_all(&tmp)?;
    let src = tmp.join("src.sqlite");
    dataset::generate(&src, dataset::SMOKE, seed)?;
    let conn = Connection::open(&src)?;
    let m = lotion_core::backup::backup(&conn, out, 0)?;
    let (hash, rows) = lotion_core::backup::canonical_dump_hash(&conn)?;
    std::fs::write(
        PathBuf::from(format!("{}.dump-sha256", out.display())),
        &hash,
    )?;
    println!(
        "backup {} from {}: {rows} rows, dump {hash}",
        out.display(),
        m.source_os
    );
    drop(conn);
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(())
}

/// Consumer side: restore `<file>` and compare with `<file>.dump-sha256`.
fn restore_check(file: &Path) -> Res<()> {
    let want = std::fs::read_to_string(format!("{}.dump-sha256", file.display()))?;
    let tmp = std::env::temp_dir().join(format!("lotion-rc-{}", std::process::id()));
    std::fs::create_dir_all(&tmp)?;
    let dest = tmp.join("restored.sqlite");
    let m = lotion_core::backup::restore(file, &dest)?;
    let (got, rows) = lotion_core::backup::canonical_dump_hash(&Connection::open(&dest)?)?;
    let _ = std::fs::remove_dir_all(&tmp);
    println!(
        "restored backup made on {} ({rows} rows) on {}",
        m.source_os,
        std::env::consts::OS
    );
    if got.trim() != want.trim() {
        return Err(format!("canonical dump differs: {got} != {}", want.trim()).into());
    }
    println!("canonical dump equal: {got}");
    Ok(())
}
