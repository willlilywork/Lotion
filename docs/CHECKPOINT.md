# Checkpoint

Updated at every stopping point so work can resume cold.

## Current — CP-014 — 2026-11-20

- **State:** Phase 0R: T0-R1, R2, R4, R5, R6, R7 done; T0-R8 first half done (Spike C backup/restore, LOG-0021; cross-OS `backup-roundtrip` CI job added, first run to be confirmed). T0-R3 done except WebDriver E2E.
- **Next task:** confirm CI for the Spike C commit (rust, backup-roundtrip, verify, Package on both OSes); then T0-R8 second half: remove the Electron scaffold and pnpm toolchain except `check-ids`; then T0-R3 E2E via tauri-driver → G0.
- **Stopped because:** daily Hark usage near the 96% limit.
- **Open owner decisions:** license (MIT or AGPL-3.0); code-signing certificate (ADR-017).
- **Resume:** as CP-012. Sandbox /tmp is ~6 GB: delete `/tmp/lotion-target/debug` before a full perf run.

## Previous — CP-013 — 2026-11-20

- **State:** Phase 0R: T0-R1, T0-R2, T0-R4, T0-R5, T0-R6, T0-R7 done (Spike B: P-DBVIEW 63 ms, P-RECOMP 27 ms, LOG-0020). T0-R3 done except WebDriver E2E. Remaining for G0: T0-R3 E2E, T0-R8.
- **Next task:** T0-R8 Spike C (ZIP + manifest backup/restore, Windows→Ubuntu CI round-trip, canonical dump equality), then remove the Electron scaffold; T0-R3 E2E via tauri-driver.
- **Open owner decisions:** license (MIT or AGPL-3.0); code-signing certificate (ADR-017).
- **Resume:** as CP-012.

## Previous — CP-012 — 2026-11-20

- **State:** Phase 0R: T0-R1, T0-R2, T0-R4, T0-R5, T0-R6 done. T0-R3 done except WebDriver E2E. Spike A re-recorded on Rust (LOG-0019): all budgets met.
- **Next task:** T0-R7 Spike B (computed values at 10k rows, dirty-set recompute of 1,000 dependents) in `crates/lotion-perf/src/db_engine.rs`; then T0-R3 E2E; T0-R8 Spike C + Electron removal.
- **Open owner decisions:** license (MIT or AGPL-3.0); code-signing certificate (ADR-017).
- **Resume:** as CP-010. `cargo run -p lotion-perf --release -- --smoke` (full run without `--smoke`, ~1 min).

## Previous — CP-011 — 2026-11-20

- **State:** Phase 0R: T0-R1, T0-R2, T0-R4, T0-R5 done (CI 44830637760 + Package 44830637869 green on Windows + Ubuntu at eabbd8b). T0-R3 done except WebDriver E2E via tauri-driver.
- **Next task:** T0-R6 port dataset generator + perf harness to Rust (`lotion-perf`), then T0-R3 E2E, T0-R7 Spike B, T0-R8 Spike C + Electron removal.
- **Open owner decisions:** license (MIT or AGPL-3.0); code-signing certificate (ADR-017).
- **Resume:** as CP-010.

## Previous — CP-010 — 2026-11-20

- **State:** Phase 0R: T0-R1, T0-R2 done; T0-R3 Tauri shell done except WebDriver E2E (smoke green in CI on Windows + Ubuntu, e64c12b); T0-R4 DB worker + single typed `request` command done locally.
- **Next task:** T0-R5 packaging with the Tauri bundler (NSIS + `.deb`, install/launch/uninstall in CI), then T0-R3 E2E, T0-R6 perf port.
- **Open owner decisions:** license (MIT or AGPL-3.0); code-signing certificate (ADR-017).
- **Resume:** as CP-009. Sandbox note: Rust lives in `/workspace/.rust` (set RUSTUP_HOME, CARGO_HOME); system packages (gcc, webkit2gtk dev, xvfb) must be reinstalled with apt in a fresh sandbox.

## Previous — CP-009 — 2026-11-20

- **State:** ADR-019 (Tauri 2 + rusqlite) accepted; Phase 0R planned in 04 (T0-R1…T0-R8); 02-ARCHITECTURE rewritten. T0-R1 done; T0-R2 `lotion-core` crate (schema v1 generated from 03 and tested against it, ping, UUIDv7, FTS5 Arabic) with CI job `rust`.
- **Next task:** T0-R3 Tauri 2 shell + launch E2E.
- **Open owner decisions:** license (MIT or AGPL-3.0); code-signing certificate (ADR-017).
- **Resume:** README → 00-CHARTER → DECISIONS ADR-018/019 → 04 Phase 0R → LOG last entry. Before every push: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`, plus `pnpm install --frozen-lockfile && pnpm verify && pnpm perf --smoke` until T0-R8.

## Previous — CP-008 — 2026-11-20

- **State:** direction changed (ADR-018): Notion's interface and shortcuts, open source, local, Rust. Charter, README, DECISIONS, 02 and 04 updated. Electron scaffold T0-01…T0-08 remains on main as a reference; spike numbers carry over.
- **Next task:** T0-R1 re-plan Phase 0 onto Rust, after the owner confirms the framework (Tauri 2 proposed) and the license (MIT or AGPL-3.0).
- **Open owner decisions:** framework; license; repo visibility (moot once open source); code-signing certificate (ADR-017).
- **Resume:** README → 00-CHARTER → DECISIONS ADR-018 → 04-BUILD-PLAN note → LOG last entry. Before every push: `pnpm install --frozen-lockfile && pnpm verify && pnpm perf --smoke` (until the Rust toolchain replaces it).

## Previous — CP-007 — 2026-11-19

- **State:** T0-01 DONE; T0-02…T0-07 green on Windows + Ubuntu, R tasks awaiting second review. TypeScript 6.0.3 (ADR-014a).
- **Done since:** T0-08 Spike A (LOG-0011): schema meets budgets at 1M blocks. T0-10 partial (LOG-0012): VACUUM INTO 3.6 s; remaining: ZIP + manifest, cross-OS CI round-trip, canonical dump equality.
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
