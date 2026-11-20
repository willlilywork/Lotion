use super::*;

#[test]
fn schema_matches_data_model_doc() {
    let md = include_str!("../../../docs/03-DATA-MODEL.md");
    let start = md.find("```sql\n").expect("sql block") + "```sql\n".len();
    let end = start + md[start..].find("```").expect("end of sql block");
    let body: String = SCHEMA_V1
        .lines()
        .skip_while(|l| l.starts_with("--"))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(body, &md[start..end], "regenerate migrations/0001_init.sql");
}

#[test]
fn ping_reports_sqlite_and_schema() {
    let ws = Workspace::open_in_memory().unwrap();
    let pong = ws.ping().unwrap();
    assert!(pong.sqlite_version.starts_with("3."));
    assert_eq!(pong.schema_version, SCHEMA_VERSION);
}

#[test]
fn reopen_is_idempotent_and_too_new_is_refused() {
    let dir = std::env::temp_dir().join(format!("lotion-test-{}", new_id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("ws.lotiondb");
    drop(Workspace::open(&file).unwrap());
    let ws = Workspace::open(&file).unwrap();
    assert_eq!(ws.ping().unwrap().schema_version, 1);
    ws.conn().pragma_update(None, "user_version", 99).unwrap();
    drop(ws);
    assert!(matches!(
        Workspace::open(&file),
        Err(Error::TooNew { found: 99, .. })
    ));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn foreign_keys_enforced() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws.conn().execute(
        "INSERT INTO block(id,type,parent_id,page_id,sort_key,created_at,updated_at) \
         VALUES ('b','paragraph','missing','p','a',0,0)",
        [],
    );
    assert!(err.is_err());
}

#[test]
fn fts5_finds_arabic_and_english() {
    let ws = Workspace::open_in_memory().unwrap();
    let c = ws.conn();
    c.execute(
        "INSERT INTO fts(block_id,page_id,text) VALUES ('1','p','مرحبا بالعالم'),('2','p','hello world')",
        [],
    )
    .unwrap();
    let hit = |q: &str| -> String {
        c.query_row("SELECT block_id FROM fts WHERE fts MATCH ?1", [q], |r| {
            r.get(0)
        })
        .unwrap()
    };
    assert_eq!(hit("مرحبا"), "1");
    assert_eq!(hit("hel*"), "2");
}

#[test]
fn ids_are_time_ordered() {
    let a = new_id();
    let b = new_id();
    assert!(a < b);
}
