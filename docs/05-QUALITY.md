# 05 — Quality

## Performance budgets

Measured on CI runners (2-4 vCPU) against the reference dataset from `scripts/gen-dataset.ts` (50k pages, 1M blocks, 20 databases × 10k rows with relations between 6 of them, 40 rollups, 60 formulas, ~30% Arabic text). Reported as p50/p95 over 30 runs in `tests/perf/report.json`; regressions > 20% fail CI.

| ID | Operation | p95 budget |
|---|---|---|
| P-START | Cold start to editable last page | 1500 ms |
| P-PAGE | Open page with 2,000 blocks (first paint of viewport) | 150 ms |
| P-KEY | Keystroke to paint | 16 ms |
| P-SAVE | Command round-trip (renderer→DB→ack) | 10 ms |
| P-SEARCH | Quick find over full dataset, first 20 results | 100 ms |
| P-DBVIEW | Table view, 10k rows with 3 relations, 3 rollups, 5 formulas; nested filter + 2 sorts + group; first window | 150 ms |
| P-RECOMP | Edit one cell feeding 1,000 dependent computed values across 2 databases | 50 ms |
| P-REDEF | Change a formula on a 10k-row database (full column recompute) | 2 s |
| P-BACKUP | Export DataBackup.lotion for reference dataset (excluding assets) | 20 s |
| P-RESTORE | Import + verify the same backup | 30 s |
| P-SCROLL | Scrolling 10k-row table | ≥ 55 fps |
| P-MEM | Resident memory after opening 20 pages | < 600 MB |

## Test pyramid

- **Unit (Vitest):** commands, undo, fractional index, filter compiler, normalizer, converters, keymap. Property-based tests (fast-check) for undo reversibility, ordering, and filter compiler vs reference evaluator.
- **E2E (Playwright `_electron`):** every AC marked E2E; tagged `@AC-XX-NNN` in the test title so `check-ids` can map them.
- **Visual regression:** our own UI only (LTR and RTL fixtures), Playwright screenshots stored in-repo — screenshots of Lotion itself, never of third-party products.
- **Perf:** nightly full run; smoke run on every PR.
- **Accessibility:** axe-core on main screens (AC-A11Y-001, 0.1); every interactive control has a role and label; focus visible; full keyboard paths covered by E2E.

## Traceability

- `docs/01-PRODUCT-SPEC.md` is the only place AC IDs are defined.
- `pnpm check-ids` (every CI run) fails on: duplicate ID, AC outside its own FT section, AC without a release tag, test tag naming no AC, any doc referencing an undefined AC.
- `pnpm check-ids --release 0.1` additionally fails on any AC of that release without a test tagged `@AC-…`. It is part of gate G4 (release), not of every push, because ACs gain tests phase by phase.
- `docs/LOG.md` is append-only: `LOG-NNNN | date (UTC) | task | commit | CI run URL | result`.
- CI artifacts kept 90 days; release evidence attached to the GitHub Release (permanent).

## CI and supply chain

- All third-party actions pinned to full commit SHA, with the tag in a comment; Dependabot updates them weekly.
- Default `permissions: contents: read`; no workflow pushes to `main`.
- Lockfile committed; `pnpm install --frozen-lockfile` in CI.
- `pnpm audit --prod` and license check (allow MIT/ISC/BSD/Apache-2.0/OFL/MPL-2.0) run on PRs.
- Electron fuses: `RunAsNode` off, `EnableNodeOptionsEnvironmentVariable` off, `OnlyLoadAppFromAsar` on, `EnableEmbeddedAsarIntegrityValidation` on.

## Security review checklist (tasks marked R)

Context isolation, sandbox, CSP, no `remote`, IPC schema validation, asset protocol cannot read arbitrary paths, path traversal tests on import, zip-slip tests on import, no `shell.openExternal` without scheme allow-list (`http`, `https`, `mailto`).
