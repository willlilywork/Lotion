# Checkpoint

Updated at every stopping point so work can resume cold.

## Current — CP-007 — 2026-11-19

- **State:** T0-01 DONE; T0-02…T0-07 green on Windows + Ubuntu, R tasks awaiting second review. TypeScript 6.0.3 (ADR-014a).
- **Done since:** T0-08 Spike A (LOG-0011): schema meets budgets at 1M blocks.
- **Next task:** T0-09 (rollups/formulas, `computed_value`, dirty-set recompute) and T0-10 (backup round-trip).
- **Open owner decisions:** repo visibility (public; private advised); code-signing certificate (ADR-017).
- **Resume:** README → 04-BUILD-PLAN Phase 0 → LOG last entry. Before every push: `pnpm install --frozen-lockfile && pnpm verify && pnpm perf --smoke`.

## Previous — CP-006 — 2026-11-19

- **State:** T0-01 DONE. T0-02…T0-06 green on Windows + Ubuntu (CI 44548040186, Package 44548040837), each R task awaiting its second review. TypeScript 6.0.3 adopted (ADR-014a); no open PRs. Session stopped at ~68% daily usage.
- **Next task:** T0-07 perf harness + synthetic dataset generator, then spikes T0-08…T0-10.
- **Open owner decisions:** repo visibility (public now; private advised); code-signing certificate (ADR-017).
- **Resume steps:** README → 04-BUILD-PLAN Phase 0 → LOG.md last entry. Before every push: `pnpm install --frozen-lockfile && pnpm verify`.

## Previous — CP-005 — 2026-11-19

- **State:** T0-01 DONE. T0-02, T0-03, T0-04 green on Windows + Ubuntu (latest run 44545301741), each awaiting its second review (R). TypeScript 6.0.3 adopted (ADR-014a); no open PRs. Session stopped at ~68% daily usage.
- **Next task:** T0-06 ID checker (small, independent), then T0-05 packaging.
- **Resume steps:** README → 04-BUILD-PLAN Phase 0 → LOG.md last entry.

## Previous — CP-004 — 2026-11-19

- **State:** T0-01 DONE. T0-02, T0-03 green on Windows + Ubuntu (run 44544245891), each awaiting its second review (R). Dependabot PR #2 closed; #1 (TypeScript 6.0.3) open, unevaluated.
- **Next task:** T0-04 `utilityProcess` DB host + MessagePort broker + zod-validated `ping`.
- **Resume steps:** README → 04-BUILD-PLAN Phase 0 → LOG.md last entry.

## Previous — CP-003 — 2026-11-19

- **State:** T0-01 DONE; T0-02 CI green on Windows + Ubuntu (run 44540499990), awaiting its second review (R). Dependabot active; open PRs #1 (TypeScript 6.0.3, within typescript-eslint range — evaluate) and #2 (@types/node 26, now ignored for majors — close).
- **Next task:** T0-03 Electron shell (sandboxed renderer, preload bridge, CSP, navigation lock) with an E2E launch test via Playwright `_electron`.
- **Resume steps:** README → 04-BUILD-PLAN Phase 0 → LOG.md last entry.

## Previous — CP-002 — 2026-11-19

- **State:** plan re-centred on advanced databases (ADR-010…013): full property set incl. relation/rollup/formula/unique ID, six views, linked views, templates, sub-items, calculations; formula language spec (06-FORMULA); computed-value engine; DataBackup.lotion format with cross-OS round-trip. Build plan now Phases 0–5 with spikes T0-08…T0-10.
- **Next task:** T0-01 scaffold.
- **Resume steps:** README → 04-BUILD-PLAN Phase 0 → first task not DONE in `docs/LOG.md`.

## Previous — CP-001 — 2026-11-19

- **State:** planning docs 00–05, DECISIONS (ADR-001…009), README written, including native Arabic three-level direction (FT-BIDI). No code. No CI yet.
- **Confirmed by user:** Lotion = local, offline alternative to a hosted block-and-database workspace, personal use, Windows `.exe` + `.deb`.
- **Next task:** T0-01 (scaffold), then T0-02 (CI), T0-03 (Electron shell).
- **Open questions:** none blocking T0-01.
- **Resume steps:** read README → 04-BUILD-PLAN Phase 0 → start the first task not marked DONE in `docs/LOG.md`.
