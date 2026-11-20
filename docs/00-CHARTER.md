# 00 — Charter

## Purpose

**Lotion is to Notion what PhotoCraft is to Photoshop** (owner, 2026-11-20; ADR-018): it reproduces Notion's interface and keyboard shortcuts so a Notion user is at home immediately. The differences are that Lotion is **open source**, **runs locally** (no account, no cloud, no telemetry) and is **built in Rust**.

A personal, offline workspace whose reason to exist is an **advanced database layer** — relations, rollups, formulas, many views over the same data, linked views and templates — of the kind most productivity apps (including local-first ones) do not offer, combined with a block editor and a **portable full backup** (`DataBackup.lotion`) that moves the entire workspace to another machine.

## Users

One person on one machine (Windows 11 or Ubuntu 22.04+/Debian 12+). Arabic and English content, mixed in the same page.

## Interface parity and independence (binding; replaces the clean-room policy of ADR-001)

- **Goal:** the layout, interactions and keyboard shortcuts match Notion's, so muscle memory transfers (sidebar, page header, slash menu, block handles, database views, command palette, shortcuts).
- **Allowed sources:** Notion's public help center and published shortcut lists, public product pages and videos, and using the product as an ordinary user; general knowledge of block editors and databases; open-source libraries under compatible licenses.
- **Forbidden:** decompiling, de-minifying or inspecting the network traffic, bundles or storage of Notion or any proprietary app; copying their code, icons, illustrations, fonts, product copy, templates or screenshots into the repository; the name "Notion", its logo, or anything implying affiliation in the UI, installer or package metadata.
- **Own identity:** name and logo are Lotion's. Fonts are system or OFL-licensed (e.g. Inter, IBM Plex Sans Arabic); icons from an open set (e.g. Lucide, ISC). Layout and colors may be close to Notion's; artwork is our own.
- **Interoperability:** importing Markdown/CSV/HTML exports the user already owns is allowed.

This is a guardrail, not legal advice. Before the first public release, have the parity rules reviewed by counsel.

## Product principles

1. **Local and durable.** Every keystroke is persisted within 1 s; crash loses at most the last 1 s; one-file backup.
2. **Fast at real size.** Budgets in 05-QUALITY are measured at 50k pages / 1M blocks.
3. **Bidirectional text is first-class.** Each block resolves its own direction; Arabic shaping, IME and AltGr work everywhere text is typed.
4. **Keyboard-complete, Notion shortcuts.** Every action reachable without a mouse, using Notion's published shortcuts (Ctrl on Windows/Linux where Notion uses Cmd), with no collisions per OS.
5. **Small verified steps.** Each task ends green on CI on Windows and Ubuntu with a recorded log entry.

## In scope for 0.1 ("daily driver")

**Databases first:** all property types incl. relation, rollup, formula, unique ID, files, person; table, board, list, gallery, calendar and timeline views; nested filter groups, multi-sorts, grouping and sub-grouping, calculations; linked views; row templates; sub-items and dependencies.
**Data portability:** DataBackup.lotion export/import across Windows and Ubuntu, optional encryption, scheduled backups; Markdown/CSV import and export.
**Editor & workspace:** pages and nested pages, sidebar, core block types, slash menu, inline formatting, mentions and backlinks, drag to reorder, undo/redo, full-text search, Trash, themes, native Arabic with three-level direction.

## Out of scope for 0.1

Real-time collaboration, sync, sharing/publishing, comments, AI features, web clipper, mobile, buttons, chart view, automations and page history (0.2), form view and synced blocks (0.3), plugins.

## Definition of done for 0.1

All acceptance IDs tagged `0.1` in 01-PRODUCT-SPEC pass in CI on both OSes; performance budgets pass on the reference dataset; installers build, install, launch, and uninstall cleanly on clean VMs in CI.
