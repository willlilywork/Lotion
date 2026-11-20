//! Lotion desktop shell (T0-R3, ADR-019).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lotion_core::host::{Command, DbHost, HostError, Location, Response};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// The single entry point for the UI (ADR-019). The command is validated by serde;
/// the blocking wait runs off the UI thread.
#[tauri::command]
async fn request(host: tauri::State<'_, DbHost>, cmd: Command) -> Result<Response, HostError> {
    let host = host.inner().clone();
    tauri::async_runtime::spawn_blocking(move || host.request(cmd))
        .await
        .map_err(|_| HostError::WorkerGone)?
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
            let host = DbHost::spawn(Location::File(dir.join("workspace.lotiondb")))?;
            app.manage(host);
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
        .invoke_handler(tauri::generate_handler![request, smoke_report])
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
