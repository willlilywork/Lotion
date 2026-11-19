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
