# 04 — Build Plan

## Rules

- Each task card names: **paths** it creates/changes, the **acceptance IDs** it closes, the exact **verify command**, and a **size** (S ≤ ½ day, M ≤ 1 day). Nothing larger than M; split instead.
- A task is `DONE` when its verify command passes on CI (windows-latest + ubuntu-latest) and a log entry `LOG-NNNN` is appended to `docs/LOG.md` with commit SHA and CI run URL.
- Review is **risk-tiered**: tasks marked `R` (security boundary, data loss, migrations, packaging) get a second review pass (a separate reviewer session) before `DONE`; others are self-verified by CI. No single gate blocks the whole plan.
- A vertical slice ships first: by the end of Phase 2 the app is usable for real notes.

## Phase 0 — Foundations (gate G0: CI green, packaging proven)

| ID | Task | Paths | Verify | Size |
|---|---|---|---|---|
| T0-01 | Repo scaffold: pnpm, TS strict, ESLint, Prettier, Vitest, `.editorconfig`, LICENSE (private, all rights reserved) | `package.json`, `tsconfig*.json`, `eslint.config.js` | `pnpm lint && pnpm test` | S |
| T0-02 | CI: matrix win/ubuntu, Node LTS, pnpm cache, actions pinned by SHA, `permissions: contents: read`, artifacts retention 90 days | `.github/workflows/ci.yml` | CI green | S R |
| T0-03 | Electron shell: main, sandboxed renderer, preload bridge, CSP, navigation lock | `src/main/*`, `src/preload/*`, `src/renderer/index.html` | `pnpm e2e -g "launch"` | M R |
| T0-04 | `utilityProcess` DB host + MessagePort broker + zod-validated `ping` command | `src/db/host.ts`, `src/main/db-broker.ts`, `src/shared/commands/ping.ts` | `pnpm e2e -g "db ping"` | M R |
| T0-05 | Packaging: NSIS `.exe` and `.deb`; CI installs, launches (smoke), uninstalls on runners | `electron-builder.yml`, `.github/workflows/package.yml` | package job green | M R |
| T0-06 | ID/traceability checker: unique AC IDs, each mapped to a FT and ≥1 test tag `@AC-…` | `scripts/check-ids.ts` | `pnpm check-ids` | S |
| T0-07 | Perf harness + synthetic dataset generator (50k pages, 1M blocks, 20 DBs × 10k rows, 30% Arabic) | `scripts/gen-dataset.ts`, `tests/perf/*` | `pnpm perf --smoke` | M |
| T0-08 | Spike: SQLite schema at 1M blocks — page load, sibling insert, FTS query, DB view window; record numbers | `tests/perf/schema.spec.ts`, `docs/LOG.md` | numbers within 05 budgets or ADR raised | M |

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

## Phase 3 — Databases (G3)

T3-01 property CRUD + conversions (AC-DB-002) · T3-02 filter/sort compiler + reference evaluator (AC-DB-007/008) · T3-03 table view virtualized (AC-DB-003/010) · T3-04 board (AC-DB-004) · T3-05 list (AC-DB-005) · T3-06 calendar (AC-DB-006) · T3-07 multiple views (AC-DB-009) · T3-10 database direction and mirrored views (AC-BIDI-011) · T3-08 inline DB block (AC-DB-001) · T3-09 CSV import/export (AC-DB-011). Cards to be expanded to full form at G2.

## Phase 4 — I/O and release 0.1 (G4)

T4-01 Markdown export (AC-IO-001) · T4-02 Markdown/CSV import (AC-IO-002) · T4-03 PDF export (AC-IO-003) · T4-04 perf pass on reference dataset · T4-05 release checklist, clean-VM install tests, `0.1.0` tag.

## Gates

| Gate | Opens | Condition |
|---|---|---|
| G0 | Phase 1 | T0-01…T0-08 DONE; T0-08 numbers recorded |
| G1 | Phase 2 & 3 (in parallel) | Phase 1 DONE; migration fixture committed |
| G2 | daily use | Phase 2 DONE |
| G3 | Phase 4 | Phase 3 DONE |
| G4 | release | all `0.1` AC pass; perf budgets pass |
