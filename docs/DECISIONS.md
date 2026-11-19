# Decisions

Format: `ADR-NNN — title — status — date`. Context, decision, consequences. Lessons from the Lana design review (2026-11-17) are cited where they shaped a decision.

## ADR-001 — Clean-room, independent identity — Accepted — 2026-11-19
Context: hosted editors' terms forbid reverse engineering and use of the service to build a competing product. Decision: 00-CHARTER clean-room policy; no visual-parity goal; behavior specified from general conventions and public docs. Consequence: no screenshot "evidence packs"; acceptance oracles are our own.

## ADR-002 — Block tree with fractional ordering — Accepted — 2026-11-19
Context: Lana review found integer midpoint ordering exhausts. Decision: base-62 fractional keys + compaction. Consequence: unique `(parent_id, sort_key)` index; property tests required.

## ADR-003 — DB in `utilityProcess` — Accepted — 2026-11-19
Context: Lana review flagged DB worker inside main. Decision: `utilityProcess` + direct MessagePort to renderers. Consequence: restart/replay logic (T1-08).

## ADR-004 — Single undo stack per page in DB process — Accepted — 2026-11-19
Context: Lana review found two unrelated undo stacks. Decision: PM history disabled; inverses stored with commands. Consequence: every command must define an inverse; property test enforces it.

## ADR-005 — Durable change log from day one — Accepted — 2026-11-19
Decision: `change_log` written in the same transaction as every command. Consequence: enables live queries, crash replay, and a future sync without schema break.

## ADR-006 — Risk-tiered review and phase gates — Accepted — 2026-11-19
Context: Lana's single lock gated 200+ tasks and required review on every task. Decision: gates per phase; second review only for `R` tasks. Consequence: usable slice at G2.

## ADR-007 — Measure scale before freezing schema — Accepted — 2026-11-19
Context: Lana had no evidence beyond 120 rows. Decision: T0-08 spike at 1M blocks before G1. Consequence: schema in 03 is a draft until T0-08 numbers are logged.

## ADR-008 — Stack — Proposed — 2026-11-19
Electron (exact version pinned at T0-01 to the current stable), React 19, ProseMirror, better-sqlite3, Vite, Vitest, Playwright, electron-builder, pnpm. Accepted when T0-03…T0-05 are green.

## ADR-009 — Three-level text direction — Accepted — 2026-11-19
Context: native Arabic support is a core requirement, beyond what common block editors offer. Decision: block override → page mode (`auto|ltr|rtl|inherit`, inherited by sub-pages) → auto by first strong character; resolution computed in the renderer, not stored per block. Consequence: switching a page with thousands of blocks costs one write; "apply to all" clears overrides in one undoable transaction.

## ADR-010 — Databases are the core; engine before UI — Accepted — 2026-11-19
Context: the user's stated reason for Lotion is a database layer equal to the most advanced hosted workspaces, which local-first apps lack. Decision: relations, rollups, formulas, six view types, linked views and templates are 0.1 scope; the engine is built and proven headless (Phase 3) in parallel with the editor. Consequence: 0.1 is larger; Phase 0 gains spike T0-09.

## ADR-011 — Materialized computed values with synchronous propagation — Accepted (pending T0-09 numbers) — 2026-11-19
Decision: formula/rollup results stored in `computed_value`, recomputed via a property dependency DAG inside the writing transaction; time-dependent formulas evaluated at query time. Consequence: filters/sorts on computed properties use indexes; writes pay for propagation (budget P-RECOMP). Fallback if T0-09 fails: deferred propagation in a background transaction with a "computing…" cell state.

## ADR-012 — Own formula language — Accepted — 2026-11-19
Decision: Lotion Formula (06-FORMULA), spreadsheet-style, typed, sandboxed (no JS eval). Consequence: golden tests per function; no claim of compatibility with any other product's formulas.

## ADR-013 — Portable backup container `.lotion` — Accepted — 2026-11-19
Decision: ZIP with manifest + `VACUUM INTO` snapshot + content-addressed assets, per-entry SHA-256, optional AES-256-GCM/Argon2id. Consequence: cross-OS round-trip CI job is a release gate.
