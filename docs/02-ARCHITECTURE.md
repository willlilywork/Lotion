# 02 — Architecture

Rust core, Tauri 2 shell, web UI in the system webview (ADR-018, ADR-019). The Electron scaffold of Phase 0 is kept until T0-R8 as a record and is described at the end.

## Stack (pinned in DECISIONS)

| Layer | Choice | Why |
|---|---|---|
| Core language | Rust (toolchain pinned in `rust-toolchain.toml`), `unsafe_code = forbid`, clippy deny | Owner's direction; one fast, safe core for storage, databases, formulas and backup |
| Shell | Tauri 2 | Rust host, small installers, system webview (WebView2 on Windows, WebKitGTK on Linux) |
| Database | SQLite via `rusqlite` (`bundled`, FTS5), WAL | Same SQLite version on every OS; schema unchanged from 03 |
| UI | TypeScript + React 19 in the webview; ProseMirror, one EditorView per page | Interface and shortcut parity with Notion (00-CHARTER); mature Arabic shaping, bidi and IME in the browser engine (ADR-009) |
| IPC | Tauri `invoke` with one command entry point; serde-typed command enum, validated in Rust | Narrow, typed boundary; no raw filesystem or shell access from the UI |
| Tests | `cargo test` (core), Vitest (UI units), WebDriver via `tauri-driver` (E2E) | All headless in CI on Windows and Ubuntu |
| Packaging | Tauri bundler: NSIS `.exe`, `.deb` | Required targets |

## Processes and threads

```
┌──────────────── lotion-app (one process) ────────────────┐
│ main thread: Tauri event loop, windows, menus, dialogs    │
│   invoke("cmd", {...}) ─► validate ─► channel             │
│ DB worker thread: owns the only rusqlite Connection       │
│   commands, queries, FTS, computed values, undo, backup   │
└──────────────▲────────────────────────────────────────────┘
               │ invoke / events
┌──────────────┴──────────────┐
│ webview: UI (React + PM)    │
│ CSP, no remote content      │
└─────────────────────────────┘
```

- One DB worker thread owns the connection; the UI thread never blocks on SQLite. A panic in the worker is caught, the worker restarts once, and the UI replays unacknowledged commands by `clientOpId` (idempotent).
- Capabilities: the webview may call only Lotion's own commands; no `fs`, `shell` or `http` plugins. Strict CSP (`default-src 'self'; img-src 'self' lotion-asset: data:`), navigation and new windows denied.
- Assets live in `<workspace>/assets/<sha256>.<ext>`, served by a custom `lotion-asset:` protocol that resolves hashes only, never paths.

## Write path (single source of truth)

All mutations are **commands** (Rust enum `Command` in `lotion-core`, mirrored as TypeScript types generated from it), e.g. `block.insert`, `block.updateText`, `block.move`, `prop.set`, `page.trash`.

1. The UI computes a command from a ProseMirror transaction.
2. The DB worker applies it inside one SQLite transaction: rows, FTS rows, `change_log` row, `undo` entry, computed values.
3. The worker emits `changed {seq, pageIds, blockIds}`; every window reconciles and ignores its own echo by `clientOpId`.

Typing is coalesced in the UI (300 ms idle or 1 s max) into one `block.updateText` per block, bounding data loss to 1 s (AC-ED-001).

## Read path

- `page.load(pageId)` returns the block tree for visible depth plus collapsed counts; large pages stream in chunks of 500 blocks.
- `db.query(viewId, window)` compiles filters, sorts, groups and calculations to SQL over `prop_value`, `prop_option_value`, `relation_edge` and `computed_value` (see 03) and returns a window of rows, group headers, totals and calculation results.
- Live queries: per subscription, the worker keeps the tables/properties a query depends on; a change touching them triggers a re-query.

## Computed values engine (databases)

In `lotion-core` (`src/compute/`): formula parser, type checker and compiler (06-FORMULA), rollup aggregators, dependency graph, dirty-set propagator. It runs synchronously inside the command's transaction, so the UI never sees a stale formula. Views read `computed_value` like any other column.

## Undo

One undo stack per page, owned by the DB worker, not ProseMirror. Each command stores its inverse; Ctrl+Z sends `undo(pageId)`; typing bursts grouped by `groupId` (500 ms).

## Repository layout

```
Cargo.toml              workspace
crates/
  lotion-core/          schema + migrations, commands, queries, FTS, compute, undo, backup
  lotion-app/           Tauri shell (T0-R3)
  lotion-perf/          dataset generator + perf harness (T0-R6)
ui/                     webview UI: shell, sidebar, editor, database views, keymap (T0-R3)
docs/                   charter, spec, architecture, data model, plan, quality, formula, decisions
scripts/check-ids.ts    traceability checker (kept)
src/, tests/, scripts/  Electron scaffold, removed in T0-R8
```

## Failure handling

- DB worker panic → restart once, non-blocking banner, replay of unacknowledged commands.
- Startup runs `PRAGMA quick_check`; on failure offers restore from the latest automatic backup (daily, 7 kept).
- A workspace whose `user_version` is newer than the build is refused, never opened read-write.
- Migrations run in a transaction with `foreign_keys=OFF` + `PRAGMA foreign_key_check` before commit; a pre-migration backup is always taken.

## Record: Electron scaffold (Phase 0, superseded)

Electron 45 main process, sandboxed renderer with a preload bridge, `better-sqlite3` in a `utilityProcess` reached by a per-document `MessagePort`, zod-validated commands, electron-builder NSIS + `.deb` (LOG-0003…0012). Lessons carried over: no session access before `ready`, CSP checks must go through the page, and E2E must never open external URLs on Linux.
