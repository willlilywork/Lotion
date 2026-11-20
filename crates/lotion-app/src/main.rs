//! Lotion desktop shell (T0-R3, ADR-019).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lotion_core::Workspace;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Open workspace. T0-R4 moves it to a dedicated DB worker thread.
struct Db(Mutex<Workspace>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PongDto {
    sqlite_version: String,
    schema_version: i64,
}

#[tauri::command]
fn ping(db: tauri::State<'_, Db>) -> Result<PongDto, String> {
    let ws =
        db.0.lock()
            .map_err(|_| "workspace lock poisoned".to_string())?;
    let p = ws.ping().map_err(|e| e.to_string())?;
    Ok(PongDto {
        sqlite_version: p.sqlite_version,
        schema_version: p.schema_version,
    })
}

/// Called by the UI in smoke mode (`LOTION_SMOKE=1`): prints the result and quits.
#[tauri::command]
fn smoke_report(app: tauri::AppHandle, db_ok: bool) {
    if std::env::var_os("LOTION_SMOKE").is_some() {
        println!(
            "LOTION_SMOKE ready db={}",
            if db_ok { "ok" } else { "fail" }
        );
        app.exit(if db_ok { 0 } else { 1 });
    }
}

/// Only the app's own pages may load in the webview.
fn is_internal(url: &tauri::Url) -> bool {
    matches!(url.scheme(), "tauri")
        || (matches!(url.scheme(), "http" | "https") && url.host_str() == Some("tauri.localhost"))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let ws = Workspace::open(dir.join("workspace.lotiondb"))?;
            app.manage(Db(Mutex::new(ws)));
            let smoke = std::env::var_os("LOTION_SMOKE").is_some();
            let url = if smoke {
                "index.html?smoke=1"
            } else {
                "index.html"
            };
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App(url.into()))
                .title("Lotion")
                .inner_size(1200.0, 800.0)
                .on_navigation(is_internal)
                .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                .build()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ping, smoke_report])
        .run(tauri::generate_context!())
        .expect("error while running Lotion");
}

#[cfg(test)]
mod tests {
    use super::is_internal;

    #[test]
    fn navigation_is_locked_to_app_pages() {
        let ok = |s: &str| is_internal(&s.parse().unwrap());
        assert!(ok("tauri://localhost/index.html"));
        assert!(ok("http://tauri.localhost/index.html"));
        assert!(!ok("https://example.com/"));
        assert!(!ok("file:///etc/passwd"));
        assert!(!ok("http://tauri.localhost.evil.com/"));
    }
}
