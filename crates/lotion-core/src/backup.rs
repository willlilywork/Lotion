//! `.lotion` backup container (03-DATA-MODEL "Backup format"; T0-R8 Spike C).
//!
//! ZIP with `manifest.json` (deflate) and `workspace.sqlite` (stored), produced by
//! `VACUUM INTO` (a consistent, compacted snapshot). Restore writes to a temporary file,
//! checks entry paths, sizes, SHA-256, `integrity_check` and schema version, then moves the
//! file into place. Not yet covered: assets, encryption, swapping a live workspace directory.

use crate::{SCHEMA_VERSION, Workspace};
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "lotion-backup";
pub const FORMAT_VERSION: u32 = 1;
const DB_ENTRY: &str = "workspace.sqlite";

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("manifest: {0}")]
    Manifest(#[from] serde_json::Error),
    #[error("not a Lotion backup")]
    NotLotion,
    #[error("backup format {0} is newer than this build ({FORMAT_VERSION})")]
    FormatTooNew(u32),
    #[error("workspace schema version {0} is newer than this build ({SCHEMA_VERSION})")]
    SchemaTooNew(i64),
    #[error("unsafe entry path in backup: {0}")]
    UnsafePath(String),
    #[error("entry {0} is damaged (size or checksum mismatch)")]
    Damaged(String),
    #[error("integrity check failed: {0}")]
    Integrity(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub pages: i64,
    pub blocks: i64,
    pub databases: i64,
    pub rows: i64,
    pub assets: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub format_version: u32,
    pub app_version: String,
    pub schema_version: i64,
    pub created_at: i64,
    pub source_os: String,
    pub encrypted: bool,
    pub entries: Vec<Entry>,
    pub counts: Counts,
}

fn sha256_file(path: &Path) -> std::io::Result<(u64, String)> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut n = 0u64;
    loop {
        let k = f.read(&mut buf)?;
        if k == 0 {
            break;
        }
        h.update(&buf[..k]);
        n += k as u64;
    }
    let hex = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    Ok((n, hex))
}

fn counts(conn: &Connection) -> rusqlite::Result<Counts> {
    let q = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0));
    Ok(Counts {
        pages: q("SELECT count(*) FROM block WHERE type='page'")?,
        blocks: q("SELECT count(*) FROM block")?,
        databases: q("SELECT count(*) FROM block WHERE type='database'")?,
        rows: q(
            "SELECT count(*) FROM block b JOIN block d ON d.id=b.parent_id WHERE b.type='page' AND d.type='database'",
        )?,
        assets: q("SELECT count(*) FROM asset")?,
    })
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// Writes a `.lotion` backup of the open connection's database to `out`.
pub fn backup(conn: &Connection, out: &Path, created_at: i64) -> Result<Manifest, BackupError> {
    let snap = sibling(out, ".snapshot.sqlite");
    let _ = std::fs::remove_file(&snap);
    conn.execute("VACUUM INTO ?", [snap.to_string_lossy()])?;
    let result = (|| {
        let c = {
            let snap_conn = Connection::open(&snap)?;
            snap_conn.execute_batch("DELETE FROM ui_state;")?; // machine-specific, excluded by default
            let c = counts(&snap_conn)?;
            let schema: i64 = snap_conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
            (c, schema)
        };
        let (bytes, sha256) = sha256_file(&snap)?;
        let manifest = Manifest {
            format: FORMAT.into(),
            format_version: FORMAT_VERSION,
            app_version: env!("CARGO_PKG_VERSION").into(),
            schema_version: c.1,
            created_at,
            source_os: std::env::consts::OS.into(),
            encrypted: false,
            entries: vec![Entry {
                path: DB_ENTRY.into(),
                bytes,
                sha256,
            }],
            counts: c.0,
        };
        let tmp = sibling(out, ".partial");
        {
            let mut zw = zip::ZipWriter::new(File::create(&tmp)?);
            let deflate = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            let stored = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .large_file(bytes > u64::from(u32::MAX));
            zw.start_file("manifest.json", deflate)?;
            zw.write_all(&serde_json::to_vec_pretty(&manifest)?)?;
            zw.start_file(DB_ENTRY, stored)?;
            std::io::copy(&mut File::open(&snap)?, &mut zw)?;
            zw.finish()?.sync_all()?;
        }
        std::fs::rename(&tmp, out)?;
        Ok(manifest)
    })();
    let _ = std::fs::remove_file(&snap);
    result
}

fn safe_path(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && !p.contains('\\')
        && !p.contains(':')
        && p.split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

/// Reads and checks only the manifest.
pub fn read_manifest(file: &Path) -> Result<Manifest, BackupError> {
    let mut z = zip::ZipArchive::new(File::open(file)?).map_err(|_| BackupError::NotLotion)?;
    let m: Manifest = {
        let mut e = z
            .by_name("manifest.json")
            .map_err(|_| BackupError::NotLotion)?;
        let mut s = String::new();
        e.read_to_string(&mut s)?;
        serde_json::from_str(&s)?
    };
    if m.format != FORMAT {
        return Err(BackupError::NotLotion);
    }
    if m.format_version > FORMAT_VERSION {
        return Err(BackupError::FormatTooNew(m.format_version));
    }
    if m.schema_version > SCHEMA_VERSION {
        return Err(BackupError::SchemaTooNew(m.schema_version));
    }
    Ok(m)
}

/// Restores a `.lotion` backup to a new workspace file at `dest` (must not exist).
pub fn restore(file: &Path, dest: &Path) -> Result<Manifest, BackupError> {
    let m = read_manifest(file)?;
    let mut z = zip::ZipArchive::new(File::open(file)?)?;
    for i in 0..z.len() {
        let name = z.by_index_raw(i)?.name().to_owned();
        if !safe_path(&name) {
            return Err(BackupError::UnsafePath(name));
        }
    }
    let entry = m
        .entries
        .iter()
        .find(|e| e.path == DB_ENTRY)
        .ok_or(BackupError::NotLotion)?
        .clone();
    let tmp = sibling(dest, ".restoring");
    let _ = std::fs::remove_file(&tmp);
    let result = (|| {
        {
            let mut src = z.by_name(DB_ENTRY)?;
            let mut out = File::create(&tmp)?;
            std::io::copy(&mut src, &mut out)?;
            out.sync_all()?;
        }
        let (bytes, sha) = sha256_file(&tmp)?;
        if bytes != entry.bytes || sha != entry.sha256 {
            return Err(BackupError::Damaged(DB_ENTRY.into()));
        }
        {
            let c = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
            let ic: String = c.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
            if ic != "ok" {
                return Err(BackupError::Integrity(ic));
            }
        }
        // migrate forward / refuse newer through the normal open path
        drop(Workspace::open(&tmp).map_err(|e| match e {
            crate::Error::TooNew { found, .. } => BackupError::SchemaTooNew(found),
            crate::Error::Sqlite(s) => BackupError::Sqlite(s),
            crate::Error::WorkerFailed => BackupError::Integrity("open failed".into()),
        })?);
        for suffix in ["-wal", "-shm"] {
            let _ = std::fs::remove_file(sibling(&tmp, suffix));
        }
        if dest.exists() {
            return Err(BackupError::Io(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "destination exists",
            )));
        }
        std::fs::rename(&tmp, dest)?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&tmp);
    result.map(|()| m)
}

/// Deterministic text dump of every user table (FTS shadow tables and `ui_state` excluded),
/// rows sorted, for cross-machine equality checks. Returns its SHA-256 and row count.
pub fn canonical_dump_hash(conn: &Connection) -> rusqlite::Result<(String, u64)> {
    let tables: Vec<String> = conn
        .prepare(
            "SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'
             AND name NOT LIKE 'fts%' AND name <> 'ui_state' ORDER BY name",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_, _>>()?;
    let mut h = Sha256::new();
    let mut n = 0u64;
    for t in tables {
        h.update(format!("\u{1}table {t}\n").as_bytes());
        let mut st = conn.prepare(&format!("SELECT * FROM \"{t}\""))?;
        let cols = st.column_count();
        let mut lines: Vec<String> = Vec::new();
        let mut rows = st.query([])?;
        while let Some(r) = rows.next()? {
            let mut line = String::new();
            for i in 0..cols {
                match r.get_ref(i)? {
                    ValueRef::Null => line.push('N'),
                    ValueRef::Integer(v) => line.push_str(&format!("I{v}")),
                    ValueRef::Real(v) => line.push_str(&format!("R{:016x}", v.to_bits())),
                    ValueRef::Text(v) => {
                        line.push('T');
                        line.push_str(
                            &String::from_utf8_lossy(v)
                                .replace('\\', "\\\\")
                                .replace('\t', "\\t")
                                .replace('\n', "\\n"),
                        );
                    }
                    ValueRef::Blob(v) => {
                        line.push('B');
                        line.extend(v.iter().map(|b| format!("{b:02x}")));
                    }
                }
                line.push('\t');
            }
            lines.push(line);
        }
        lines.sort_unstable();
        n += lines.len() as u64;
        for l in lines {
            h.update(l.as_bytes());
            h.update(b"\n");
        }
    }
    // FTS content is checked by query, not by shadow-table bytes
    let fts: i64 = conn.query_row("SELECT count(*) FROM fts", [], |r| r.get(0))?;
    h.update(format!("\u{1}fts rows {fts}\n").as_bytes());
    Ok((h.finalize().iter().map(|b| format!("{b:02x}")).collect(), n))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lotion-bk-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn sample(path: &Path) -> Connection {
        drop(Workspace::open(path).unwrap());
        let c = Connection::open(path).unwrap();
        c.execute_batch(
            "INSERT INTO block (id,type,parent_id,page_id,sort_key,content,plain_text,created_at,updated_at)
               VALUES ('p1','page',NULL,'p1','a0001','{}','مرحبا Lotion',1,1),
                      ('b1','paragraph','p1','p1','a0001','{}','ملاحظات تصميم',1,1);
             INSERT INTO fts (block_id,page_id,text) VALUES ('p1','p1','مرحبا Lotion'),('b1','p1','ملاحظات تصميم');",
        )
        .unwrap();
        c
    }

    #[test]
    fn round_trip_keeps_canonical_dump() {
        let d = tmpdir("rt");
        let c = sample(&d.join("w.sqlite"));
        let m = backup(&c, &d.join("DataBackup.lotion"), 1).unwrap();
        assert_eq!(m.counts.blocks, 2);
        let back = restore(&d.join("DataBackup.lotion"), &d.join("r.sqlite")).unwrap();
        assert_eq!(back, m);
        let r = Connection::open(d.join("r.sqlite")).unwrap();
        assert_eq!(
            canonical_dump_hash(&c).unwrap(),
            canonical_dump_hash(&r).unwrap()
        );
        let hit: String = r
            .query_row(
                "SELECT block_id FROM fts WHERE fts MATCH 'ملاحظات'",
                [],
                |x| x.get(0),
            )
            .unwrap();
        assert_eq!(hit, "b1");
        assert!(
            restore(&d.join("DataBackup.lotion"), &d.join("r.sqlite")).is_err(),
            "refuses to overwrite"
        );
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn damaged_and_foreign_files_are_rejected() {
        let d = tmpdir("bad");
        let c = sample(&d.join("w.sqlite"));
        let file = d.join("b.lotion");
        backup(&c, &file, 1).unwrap();
        // flip one byte inside the stored DB entry
        let mut bytes = std::fs::read(&file).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xff;
        std::fs::write(d.join("bad.lotion"), &bytes).unwrap();
        assert!(matches!(
            restore(&d.join("bad.lotion"), &d.join("x.sqlite")),
            Err(BackupError::Damaged(_) | BackupError::Zip(_) | BackupError::Io(_))
        ));
        assert!(!d.join("x.sqlite").exists());
        std::fs::write(d.join("not.lotion"), b"hello").unwrap();
        assert!(matches!(
            restore(&d.join("not.lotion"), &d.join("y.sqlite")),
            Err(BackupError::NotLotion)
        ));
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn unsafe_paths_are_detected() {
        for p in ["../x", "/etc/passwd", "a/../b", "C:/x", "a\\b", ""] {
            assert!(!safe_path(p), "{p}");
        }
        assert!(safe_path("assets/ab12.png"));
    }
}
