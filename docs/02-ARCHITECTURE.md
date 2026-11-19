# 02 — Architecture

## Stack (pinned in DECISIONS)

| Layer | Choice | Why |
|---|---|---|
| Shell | Electron (current stable at bootstrap, pinned exact) | Same Chromium on Win/Linux → one bidi/IME behavior to test |
| Language | TypeScript strict, ESM | One language end to end |
| UI | React 19 + Vite | Mature, virtualization libs, testing ecosystem |
| Editor | ProseMirror, **one EditorView per page**, our own schema | Bidi/IME handled by contenteditable in Chromium; full control of commands; cross-block selection is native |
| Database | SQLite via `better-sqlite3`, WAL, FTS5 | Synchronous, fast, one file |
| DB host | Electron `utilityProcess` | Crash isolation from main; no blocking of main event loop; native module loaded once |
| Virtualization | `@tanstack/react-virtual` | Long pages and DB views |
| Tests | Vitest (unit), Playwright `_electron` (E2E) | Both run headless in CI |
| Packaging | electron-builder: NSIS `.exe`, `.deb` | Required targets |

## Processes

```
┌──────────── main process ────────────┐
│ windows, menus, dialogs, file I/O     │
│ brokers MessagePorts, no DB access    │
└──────┬─────────────────────┬─────────┘
       │ MessagePort          │ MessagePort
┌──────▼──────┐        ┌──────▼─────────────┐
│ renderer(s) │◄──────►│ utilityProcess: db  │
│ React + PM  │  port  │ better-sqlite3, FTS │
│ sandboxed   │        │ command handlers    │
└─────────────┘        └────────────────────┘
```

- Renderer: `sandbox: true`, `contextIsolation: true`, `nodeIntegration: false`, strict CSP (`default-src 'self'; img-src 'self' lotion-asset: data:`), no remote content, navigation and `window.open` denied.
- Preload exposes exactly one typed API: `lotion.request(cmd)`, `lotion.subscribe(topic)`. No raw `ipcRenderer`.
- The renderer talks to the DB process over a direct `MessagePort` (main only brokers). Messages are validated with zod schemas on the DB side.
- Assets (images) live in `<workspace>/assets/<sha256>.<ext>` and are served through a custom `lotion-asset:` protocol that only resolves hashes, never paths.

## Write path (single source of truth)

All mutations are **commands** (`src/shared/commands/*.ts`), e.g. `block.insert`, `block.updateText`, `block.move`, `prop.set`, `page.trash`.

1. Renderer computes a command from a ProseMirror transaction (mapping in `src/renderer/editor/pm-to-commands.ts`).
2. DB process applies it inside one SQLite transaction: rows, FTS rows, `change_log` row, `undo` entry.
3. DB process publishes `changed {seq, pageIds, blockIds}`; every subscribed renderer reconciles. The originating renderer ignores its own echo by `clientOpId`.

Text typing is coalesced in the renderer (300 ms idle or 1 s max) into one `block.updateText` per block, which bounds data loss to 1 s (AC-ED-001).

## Read path

- `page.load(pageId)` returns the block tree for visible depth plus collapsed counts; large pages stream in chunks of 500 blocks.
- `db.query(viewId, window)` compiles filters/sorts to SQL (see 03) and returns a window of rows plus total count.
- Live queries: the DB process keeps, per subscription, the set of tables/properties a query depends on; a change touching them triggers re-query (coarse but correct; refined later if budgets require).

## Undo

One undo stack per page, owned by the **DB process**, not ProseMirror. Each command stores its inverse. The renderer's PM history plugin is disabled; Ctrl+Z sends `undo(pageId)`. Typing bursts are grouped by `groupId` (500 ms). This removes the "two unrelated undo stacks" defect found in the Lana review.

## Module layout

```
src/
  main/           app lifecycle, windows, menus, protocol, backup, updater(off)
  preload/        typed bridge
  db/             utility process entry, migrations, commands, queries, fts
  renderer/       app shell, sidebar, editor, database views, search
  shared/         command & query types (zod), keymap, ids, fractional index
tests/
  unit/  e2e/  perf/  fixtures/
scripts/          check-ids, gen-dataset, perf-report
```

## Failure handling

- DB process crash → main restarts it once, renderer shows a non-blocking banner and replays unacknowledged commands (each has `clientOpId`, idempotent).
- Startup runs `PRAGMA quick_check`; on failure offers restore from the latest automatic backup (daily, 7 kept).
- Migrations run in a transaction with `foreign_keys=OFF` + `PRAGMA foreign_key_check` before commit; a pre-migration backup is always taken.
