# 00 — Charter

## Purpose

A personal, offline block-based workspace that covers the everyday writing-and-organizing workflow of a hosted block editor, running entirely on one machine.

## Users

One person on one machine (Windows 11 or Ubuntu 22.04+/Debian 12+). Arabic and English content, mixed in the same page.

## Clean-room policy (binding)

Lotion is built from general, publicly known editor conventions and our own design. It is not a copy of any product.

- **Allowed sources:** public help-center articles and public documentation describing *what* a feature does; general knowledge of block editors, outliners and databases; open-source libraries under compatible licenses.
- **Forbidden:** decompiling, de-minifying, or inspecting the network traffic, bundles, or storage of any proprietary app; using any proprietary service or its API to derive behavior for this project; copying code, icons, illustrations, fonts, color palettes, product copy, templates, or screenshots from any proprietary product; using any third-party product name or logo in the UI, the installer, or package metadata.
- **Visual identity:** our own. Default fonts are system fonts or OFL-licensed fonts (e.g. Inter, IBM Plex Sans Arabic). Icons from an open set (e.g. Lucide, ISC).
- **Repository hygiene:** no third-party screenshots or captured data are committed. Reference notes cite public URLs only.
- **Interoperability:** import of Markdown/CSV/HTML exports the user already owns is allowed (format interop, not behavior cloning).

This policy is a guardrail, not legal advice. Anything that would be distributed beyond personal use needs counsel review first.

## Product principles

1. **Local and durable.** Every keystroke is persisted within 1 s; crash loses at most the last 1 s; one-file backup.
2. **Fast at real size.** Budgets in 05-QUALITY are measured at 50k pages / 1M blocks.
3. **Bidirectional text is first-class.** Each block resolves its own direction; Arabic shaping, IME and AltGr work everywhere text is typed.
4. **Keyboard-complete.** Every action reachable without a mouse, with no shortcut collisions per OS.
5. **Small verified steps.** Each task ends green on CI on Windows and Ubuntu with a recorded log entry.

## In scope for 0.1 ("daily driver")

Pages and nested pages, sidebar tree, core block types, slash menu, inline formatting, links and page mentions, backlinks, drag to reorder, undo/redo, full-text search, databases (table, board, list, calendar views) with properties, filters and sorts, Trash, Markdown import/export, light/dark theme, RTL/LTR.

## Out of scope for 0.1

Real-time collaboration, sync, sharing/publishing, comments, AI features, web clipper, mobile, formulas beyond a small arithmetic set, relations/rollups (planned 0.2), gallery/timeline views (0.2), synced blocks (0.3), plugins.

## Definition of done for 0.1

All acceptance IDs tagged `0.1` in 01-PRODUCT-SPEC pass in CI on both OSes; performance budgets pass on the reference dataset; installers build, install, launch, and uninstall cleanly on clean VMs in CI.
