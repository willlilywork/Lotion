/* global window, location, document */
// T0-R3 placeholder UI: proves the webview reaches SQLite through the Rust core.
const { invoke } = window.__TAURI__.core;
const smoke = new URLSearchParams(location.search).has('smoke');
const status = document.getElementById('db-status');

invoke('ping')
  .then((p) => {
    status.textContent = `SQLite ${p.sqliteVersion} · schema v${p.schemaVersion}`;
    if (smoke) return invoke('smoke_report', { dbOk: p.schemaVersion >= 1 });
  })
  .catch((e) => {
    status.textContent = `DB error: ${e}`;
    if (smoke) return invoke('smoke_report', { dbOk: false });
  });
