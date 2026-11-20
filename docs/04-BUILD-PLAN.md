# 04 — Build Plan

## Rules

- Each task card names: **paths** it creates/changes, the **acceptance IDs** it closes, the exact **verify command**, and a **size** (S ≤ ½ day, M ≤ 1 day). Nothing larger than M; split instead.
- A task is `DONE` when its verify command passes on CI (windows-latest + ubuntu-latest) and a log entry `LOG-NNNN` is appended to `docs/LOG.md` with commit SHA and CI run URL.
- Review is **risk-tiered**: tasks marked `R` (security boundary, data loss, migrations, packaging) get a second review pass (a separate reviewer session) before `DONE`; others are self-verified by CI. No single gate blocks the whole plan.
- Databases are the product's reason to exist, so the database engine (Phase 3) starts right after storage (G1), in parallel with the editor, and its scale is proven in Phase 0 (T0-09) before anything is frozen.

## Phase 0R — Foundations on Rust (ADR-018; gate G0 moves here)

Phase 0 below was built on Electron + TypeScript and proved CI, packaging, the schema at 1M blocks (T0-08) and `VACUUM INTO` (T0-10 partial). Phase 0R rebuilds the foundations on a Rust core with a Tauri 2 shell; the UI stays web technology inside the system webview (Arabic shaping, bidi, IME per ADR-009). Spike numbers and budgets carry over. Gate G0 = T0-R1…T0-R8 DONE.

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T0-R1 | Re-plan: this table, 02-ARCHITECTURE for Rust/Tauri, ADR-019 stack | `docs/*` | review | S |
| T0-R2 | Cargo workspace + `lotion-core` crate: rusqlite (bundled, FTS5), schema v1 generated from 03 and checked against it, `ping`, UUIDv7 ids; CI job `rust` (fmt, clippy `-D warnings`, test) on win/ubuntu | `Cargo.toml`, `crates/lotion-core/*`, `.github/workflows/ci.yml` | `cargo test` + CI green | S |
| T0-R3 | Tauri 2 shell: window, locked capabilities (only Lotion commands), CSP, navigation lock, `ping` through `invoke`; launch E2E (WebDriver via `tauri-driver`) | `crates/lotion-app/*`, `ui/*` | e2e "launch", "db ping" | M R |
| T0-R4 | Core runs off the UI thread: one DB worker thread owns the connection, commands over a channel, typed (serde) command/response enum with validation | `crates/lotion-core/src/host.rs` | unit + e2e "db ping" | M R |
| T0-R5 | Packaging with the Tauri bundler: NSIS `.exe` + `.deb`; CI installs, launches (smoke reaches SQLite), uninstalls | `crates/lotion-app/tauri.conf.json`, `.github/workflows/package.yml` | package job green | M R |
| T0-R6 | Port dataset generator + perf harness to Rust (`lotion-perf` binary, smoke/full); re-record Spike A numbers | `crates/lotion-perf/*` | `cargo run -p lotion-perf -- --smoke` | M — DONE (LOG-0019) |
| T0-R7 | Spike B on Rust (was T0-09): computed values at 10k rows, dirty-set recompute of 1,000 dependents | `crates/lotion-perf/src/db_engine.rs` | P-DBVIEW, P-RECOMP | M |
| T0-R8 | Spike C on Rust (rest of T0-10): ZIP + manifest backup/restore, Windows→Ubuntu round-trip in CI, canonical dump equality; then remove the Electron scaffold and pnpm toolchain except `check-ids` | `crates/lotion-core/src/backup.rs`, CI | P-BACKUP, P-RESTORE | M R |

## Phase 0 — Foundations on Electron (superseded by Phase 0R; kept as record)

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T0-01 | Repo scaffold: pnpm, TS strict, ESLint, Prettier, Vitest, `.editorconfig`, LICENSE (private, all rights reserved) | `package.json`, `tsconfig*.json`, `eslint.config.js` | `pnpm lint && pnpm test` | S |
| T0-02 | CI: matrix win/ubuntu, Node LTS, pnpm cache, actions pinned by SHA, `permissions: contents: read`, artifacts retention 90 days | `.github/workflows/ci.yml` | CI green | S R |
| T0-03 | Electron shell: main, sandboxed renderer, preload bridge, CSP, navigation lock | `src/main/*`, `src/preload/*`, `src/renderer/index.html` | `pnpm e2e -g "launch"` | M R |
| T0-04 | `utilityProcess` DB host + MessagePort broker + zod-validated `ping` command | `src/db/host.ts`, `src/main/db-broker.ts`, `src/shared/commands/ping.ts` | `pnpm e2e -g "db ping"` | M R |
| T0-05 | Packaging: NSIS `.exe` and `.deb`; CI installs, launches (smoke), uninstalls on runners | `electron-builder.yml`, `.github/workflows/package.yml` | package job green | M R |
| T0-06 | ID/traceability checker: unique AC IDs, each mapped to a FT and ≥1 test tag `@AC-…` | `scripts/check-ids.ts` | `pnpm check-ids` | S |
| T0-07 | Perf harness + synthetic dataset generator (50k pages, 1M blocks, 20 DBs × 10k rows, 30% Arabic) | `scripts/gen-dataset.ts`, `tests/perf/*` | `pnpm perf --smoke` | M |
| T0-08 | Spike A: schema at 1M blocks — page load, sibling insert, FTS query; record numbers | `tests/perf/schema.spec.ts`, `docs/LOG.md` | numbers within 05 budgets or ADR raised | M |
| T0-09 | Spike B: database engine at scale — 10k-row view with relations/rollups/formulas (materialized `computed_value`), nested filter + sort + group SQL, dirty-set recompute of 1,000 dependents | `tests/perf/db-engine.spec.ts` | P-DBVIEW, P-RECOMP met or ADR raised | M |
| T0-10 | Spike C: `VACUUM INTO` + ZIP backup/restore of reference dataset, Windows→Ubuntu round-trip in CI | `tests/perf/backup.spec.ts`, `.github/workflows/ci.yml` | P-BACKUP, P-RESTORE, canonical dump equal | S |

## Phase 1 — Storage core (G1: schema frozen v1)

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T1-01 | Migration runner + `0001_init.sql` (03-DATA-MODEL) + fixture test | `src/db/migrations/*` | `pnpm test db/migrate` | M R |
| T1-02 | Fractional index lib + property tests (10⁶ random inserts, max key length) | `src/shared/fracindex.ts` | `pnpm test fracindex` | S |
| T1-03 | Commands: block insert/update/move/delete with change_log + undo inverse | `src/db/commands/block.ts` | `pnpm test commands/block` | M |
| T1-04 | Undo/redo engine with grouping; property test: random op sequence fully reversible | `src/db/undo.ts` | `pnpm test undo` | M |
| T1-05 | FTS + Arabic normalizer | `src/db/fts.ts`, `src/shared/normalize-ar.ts` | `pnpm test fts` (AC-SR-001/002, AC-BIDI-006) | M |
| T1-06 | Trash, restore, permanent delete | `src/db/commands/trash.ts` | AC-WS-006 unit | S |
| T1-07 | Backup/restore + daily auto-backup + integrity check at startup | `src/main/backup.ts` | AC-WS-008 | M R |
| T1-08 | Crash replay: idempotent `clientOpId`, DB-process restart | `src/main/db-broker.ts` | e2e kill test, AC-ED-001 | M R |

## Phase 2 — Editor vertical slice (G2: usable daily for notes)

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T2-01 | App shell: sidebar tree (virtualized), page routing, history | `src/renderer/shell/*` | AC-WS-002/004 | M |
| T2-02 | PM schema for 0.1 block types + node views | `src/renderer/editor/schema.ts` | unit | M |
| T2-03 | PM→commands mapping + coalescing | `src/renderer/editor/pm-to-commands.ts` | AC-ED-001 | M |
| T2-04 | Enter/Backspace/Tab semantics | `src/renderer/editor/commands/*` | AC-ED-002/003/004 | M |
| T2-05 | Markdown input rules + IME guard | `src/renderer/editor/inputrules.ts` | AC-ED-005, AC-ED-020 | S |
| T2-06 | Keymap single source + uniqueness test | `src/shared/keymap.ts` | AC-KB-001 | S |
| T2-07 | Inline marks + toolbar on selection | `src/renderer/editor/marks.ts` | AC-ED-007 | M |
| T2-08 | Slash menu | `src/renderer/editor/slash/*` | AC-ED-006 | M |
| T2-09a | Direction resolver (block override → page mode → auto) + unit tests | `src/shared/direction.ts` | AC-BIDI-001, 005 | S |
| T2-09b | Per-block override UI + Ctrl+Shift keys + commands | `src/renderer/editor/bidi.ts`, `src/db/commands/direction.ts` | AC-BIDI-002, 009 | M |
| T2-09c | Page direction menu, inheritance, "apply to all" | `src/renderer/page/direction-menu.tsx` | AC-BIDI-003, 004, 005 | M |
| T2-09d | Mirrored block chrome + caret/selection E2E + Arabic-Indic numbering | `src/renderer/editor/nodeviews/*` | AC-BIDI-007, 008, 010 | M |
| T2-09e | Sidebar/quick-find auto direction, paste/export direction, spellcheck | `src/renderer/shell/*`, `src/renderer/editor/clipboard.ts` | AC-BIDI-012, 013, 015 | S |
| T2-10 | Block selection, drag handle, move/duplicate | `src/renderer/editor/blocksel/*` | AC-ED-010/011/015/016 | M |
| T2-11 | Paste/copy (MD/HTML/plain) | `src/renderer/editor/clipboard.ts` | AC-ED-012 | M |
| T2-12 | Mentions, `[[`, backlinks panel | `src/renderer/editor/mention/*` | AC-ED-008/009/019 | M |
| T2-13 | Quick find | `src/renderer/search/*` | AC-WS-005, AC-SR-003 | M |
| T2-14 | Title, icon, cover, images via asset protocol | `src/renderer/page/*`, `src/main/asset-protocol.ts` | AC-ED-018 | M R |
| T2-15 | Settings, theme, favorites, Trash UI, find in page | `src/renderer/settings/*` | AC-WS-003/006/007, AC-ED-021 | M |

## Phase 3 — Database engine (no UI; starts at G1, runs in parallel with Phase 2)

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T3-01 | Property CRUD + type conversions with undo | `src/db/commands/property.ts` | AC-DB-002 | M |
| T3-02 | Values for all simple types incl. files, person, unique ID | `src/db/commands/value.ts` | unit | M |
| T3-03 | Relations one-/two-way, self-relations, limit, delete cascade invariants (property-based) | `src/db/commands/relation.ts` | AC-DB-003 | M R |
| T3-04 | Formula lexer + Pratt parser + AST + fuzz tests | `src/db/compute/formula/parse.ts` | `pnpm test formula/parse` | M |
| T3-05 | Formula type checker + error messages | `src/db/compute/formula/types.ts` | negative-case suite | M |
| T3-06 | Formula functions (logic, math, text, date, list, page) + golden files | `src/db/compute/formula/fns/*` | AC-DB-005 goldens | M |
| T3-07 | Rollup aggregators | `src/db/compute/rollup.ts` | AC-DB-004 | S |
| T3-08 | Dependency graph, cycle detection, dirty-set propagation, `verify-computed` | `src/db/compute/graph.ts`, `propagate.ts`, `scripts/verify-computed.ts` | AC-DB-006 (property-based) | M R |
| T3-09 | Filter AST → SQL compiler (all operators, 3-level groups, relative dates) + reference evaluator | `src/db/query/filter.ts` | AC-DB-013 (property-based) | M |
| T3-10 | Sorts with ICU collation (ar, en), groups, sub-groups, calculations | `src/db/query/sort-group-calc.ts` | AC-DB-014/015/016 | M |
| T3-11 | View config model, linked views, "this row" filters, lock | `src/db/commands/view.ts` | AC-DB-017/020/021 | M |
| T3-12 | Row templates, sub-items | `src/db/commands/template.ts` | AC-DB-018/019 | M |
| T3-13 | Live query subscriptions with dependency-based invalidation | `src/db/query/live.ts` | unit + AC-DB-022 smoke | M |

## Phase 4 — Database UI (G3 needs G2 + Phase 3)

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T4-01 | Database page + inline block + view tabs + property menus | `src/renderer/db/*` | AC-DB-001, 024 | M |
| T4-02 | Table view: virtualized grid, cell editors per type, keyboard, ranges, TSV copy/paste, peek modes | `src/renderer/db/table/*` | AC-DB-008 | M (×2 cards) |
| T4-03 | Filter/sort/group builder UI | `src/renderer/db/query-ui/*` | AC-DB-013/014/015 E2E | M |
| T4-04 | Formula editor: autocomplete, inline errors, bidi tokens | `src/renderer/db/formula-editor/*` | AC-DB-005 E2E | M |
| T4-05 | Board view | `src/renderer/db/board/*` | AC-DB-009 | M |
| T4-06 | List + gallery views | `src/renderer/db/list/*`, `gallery/*` | AC-DB-007, 010 | M |
| T4-07 | Calendar view | `src/renderer/db/calendar/*` | AC-DB-011 | M |
| T4-08 | Timeline view with dependencies | `src/renderer/db/timeline/*` | AC-DB-012 | M (×2 cards) |
| T4-09 | Calculations row, database search, sub-items UI | `src/renderer/db/table/*` | AC-DB-016/019/023 | M |
| T4-10 | Templates UI, linked view insertion | `src/renderer/db/templates/*` | AC-DB-017/018 | M |
| T4-11 | Database direction & mirrored views | `src/renderer/db/*` | AC-BIDI-011 | S |
| T4-12 | Perf pass on reference dataset | `tests/perf/*` | AC-DB-022, P-DBVIEW, P-RECOMP | M |

## Phase 5 — Portability and release 0.1 (G4)

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T5-01 | DataBackup.lotion export (manifest, `VACUUM INTO`, assets, hashes) | `src/db/backup/export.ts` | AC-IO-001 | M R |
| T5-02 | Import: validate, migrate forward, atomic swap, new-workspace mode | `src/db/backup/import.ts` | AC-IO-002/003/004 | M R |
| T5-03 | Encryption (AES-256-GCM, Argon2id) | `src/db/backup/crypto.ts` | AC-IO-005 | S R |
| T5-04 | Scheduled backups | `src/main/backup-scheduler.ts` | AC-IO-006 | S |
| T5-05 | Cross-OS round-trip CI job (export Windows → import Ubuntu, and reverse) | `.github/workflows/portability.yml` | AC-IO-002 | S R |
| T5-06 | Markdown/CSV export & import, other-app export import | `src/db/io/*` | AC-IO-007/008/009 | M (×2 cards) |
| T5-07 | PDF export | `src/main/pdf.ts` | AC-IO-010 | S |
| T5-08 | Release checklist, clean-VM install tests, `0.1.0` tag with evidence attached | `docs/RELEASE.md` | all `0.1` AC green | M |

## Gates

| Gate | Opens | Condition |
|---|---|---|
| G0 | Phase 1 | T0-R1…T0-R8 DONE (Phase 0R); spike numbers recorded |
| G1 | Phase 2 & Phase 3 (in parallel) | Phase 1 DONE; migration fixture committed |
| G2 | daily note-taking | Phase 2 DONE |
| G3 | Phase 4 | G2 + Phase 3 DONE |
| G4 | Phase 5 completion & release | Phase 4 DONE; all `0.1` AC pass; perf budgets pass |
