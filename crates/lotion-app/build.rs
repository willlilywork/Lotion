fn main() {
    // Only these app commands exist for the webview; capabilities grant them explicitly.
    let manifest = tauri_build::AppManifest::new().commands(&["request", "smoke_report"]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("tauri build script");
}
