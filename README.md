# Lotion

Lotion is a local-first, offline desktop workspace for personal use, built around advanced databases (relations, rollups, formulas, six view types, linked views, templates) plus a block editor and a portable full backup (`DataBackup.lotion`) that moves everything to another machine. Data lives in one SQLite file on the user's machine. No accounts, no cloud, no telemetry.

Targets: Windows (`.exe`, NSIS installer) and Debian/Ubuntu (`.deb`).

## Status

Planning (Phase 0). No application code yet. Start with:

1. [docs/00-CHARTER.md](docs/00-CHARTER.md): scope, clean-room rules, non-goals
2. [docs/01-PRODUCT-SPEC.md](docs/01-PRODUCT-SPEC.md): what the 0.1 product does, with acceptance IDs
3. [docs/02-ARCHITECTURE.md](docs/02-ARCHITECTURE.md): processes, modules, IPC
4. [docs/03-DATA-MODEL.md](docs/03-DATA-MODEL.md): schema, ordering, undo, change log
5. [docs/04-BUILD-PLAN.md](docs/04-BUILD-PLAN.md): phases, task cards, gates
6. [docs/05-QUALITY.md](docs/05-QUALITY.md): budgets, tests, CI policy
7. [docs/06-FORMULA.md](docs/06-FORMULA.md): the formula language
8. [docs/DECISIONS.md](docs/DECISIONS.md): decision records
9. [docs/CHECKPOINT.md](docs/CHECKPOINT.md): where work stopped and what comes next
