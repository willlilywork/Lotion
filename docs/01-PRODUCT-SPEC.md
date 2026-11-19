# 01 — Product Specification (0.1)

Each requirement has an acceptance ID `AC-<AREA>-<NNN>`. IDs are unique, never reused, and each maps to exactly one feature `FT-<AREA>-<NN>` and at least one automated test. `scripts/check-ids` enforces this (see 05-QUALITY).

Release tags: **0.1** = required for the first daily-driver build; **0.2+** = planned, listed for data-model foresight only.

## FT-WS — Workspace & navigation

| ID | Requirement | Oracle (pass/fail) | Rel |
|---|---|---|---|
| AC-WS-001 | First launch creates a workspace file in the OS app-data dir and opens an empty "Getting started" page written by us. | File exists; window shows page title within 2 s on reference HW. | 0.1 |
| AC-WS-002 | Sidebar shows the page tree; pages expand/collapse; state persists across restarts. | E2E: expand A, restart, A still expanded. | 0.1 |
| AC-WS-003 | Favorites section: add/remove page; order is user-draggable. | E2E. | 0.1 |
| AC-WS-004 | Back/forward history per window (Alt+Left / Alt+Right). | E2E: visit A→B→C, back twice lands on A. | 0.1 |
| AC-WS-005 | Quick find (Ctrl+P) searches titles and content, ranked, keyboard navigable. | Results within budget P-SEARCH; Enter opens. | 0.1 |
| AC-WS-006 | Trash: deleted pages go to Trash with their subtree; restore puts them back under the original parent (or root if parent gone); permanent delete after confirm. | Unit + E2E. | 0.1 |
| AC-WS-007 | Settings: theme (light/dark/system), font (default/serif/mono), default text direction for root pages (auto/LTR/RTL), spellcheck languages. | Persisted; applied without restart. | 0.1 |
| AC-WS-008 | Settings → Data shows workspace location, size, last backup, and the export/import actions of FT-IO. | E2E. | 0.1 |
| AC-WS-009 | Multiple windows open different pages; edits appear in the other window within 200 ms. | E2E two windows. | 0.2 |

## FT-ED — Block editor

Block types for 0.1: paragraph, heading 1–3, bulleted list, numbered list, to-do, toggle, quote, callout, divider, code (with language label, no execution), image (local file copied into the workspace), page link (sub-page), table of contents (0.2), simple table (0.2).

| ID | Requirement | Oracle | Rel |
|---|---|---|---|
| AC-ED-001 | Typing in a block persists within 1 s (debounced transaction). | Kill process 1.1 s after last key; reopen shows text. | 0.1 |
| AC-ED-002 | Enter splits the block at caret; new block inherits list/to-do type; Enter on an empty list item turns it into a paragraph. | Unit on editor commands. | 0.1 |
| AC-ED-003 | Backspace at start of block: if block has a non-paragraph type, convert to paragraph; else merge into previous block, moving children. | Unit. | 0.1 |
| AC-ED-004 | Tab / Shift+Tab nests/un-nests blocks (children move with parent). | Unit. | 0.1 |
| AC-ED-005 | Markdown shortcuts at line start: `#`,`##`,`###`, `-`/`*`, `1.`, `[]`, `>`, `"`, ```` ``` ````, `---`. | Unit table-driven. | 0.1 |
| AC-ED-006 | Slash menu: `/` opens a filtered list of block types and actions; arrow keys + Enter; Esc closes and leaves the `/` text. Works with Arabic query text. | E2E. | 0.1 |
| AC-ED-007 | Inline marks: bold, italic, underline, strikethrough, inline code, link, text color/highlight (fixed palette of our own). | Unit. | 0.1 |
| AC-ED-008 | `@` mention: pages (creates backlink), dates (today, tomorrow, specific date). | Unit + E2E. | 0.1 |
| AC-ED-009 | `[[` opens page-link search; selecting inserts a page mention; "Create page" option if no match. | E2E. | 0.1 |
| AC-ED-010 | Block selection: Esc selects current block; Shift+Arrow extends; Ctrl+A twice selects all blocks; Delete removes; Ctrl+C/X/V copy blocks as both internal JSON and Markdown/HTML. | E2E. | 0.1 |
| AC-ED-011 | Drag handle reorders blocks and nests by horizontal offset; also works across selection of multiple blocks. | E2E. | 0.1 |
| AC-ED-012 | Paste: Markdown text becomes blocks; HTML maps headings/lists/links/images; plain text keeps lines as paragraphs. Ctrl+Shift+V pastes as plain text. | Unit fixtures. | 0.1 |
| AC-ED-013 | Undo/redo covers text, structure, property and move operations in one stack per page, grouped by 500 ms typing bursts. | Unit: any op sequence then N undos restores initial snapshot. | 0.1 |
| AC-ED-014 | Turn-into menu converts block type preserving text and children where meaningful. | Unit. | 0.1 |
| AC-ED-015 | Duplicate block (Ctrl+D) deep-copies subtree. | Unit. | 0.1 |
| AC-ED-016 | Move block up/down (Ctrl+Shift+Up/Down). | Unit. | 0.1 |
| AC-ED-017 | Toggle blocks remember open/closed per user (stored locally, not in content). | E2E. | 0.1 |
| AC-ED-018 | Page title, optional icon (emoji) and cover (local image or solid color). | E2E. | 0.1 |
| AC-ED-019 | Backlinks panel at page bottom lists blocks mentioning the page, with context. | Unit on query + E2E. | 0.1 |
| AC-ED-020 | IME composition (Arabic, Japanese, Chinese) never triggers shortcuts or markdown rules mid-composition; AltGr characters type normally. | E2E with synthetic composition events + keymap unit test. | 0.1 |
| AC-ED-021 | Find in page (Ctrl+F) with match highlighting. | E2E. | 0.1 |
| AC-ED-022 | Synced blocks. | — | 0.3 |

## FT-BIDI — Native Arabic & bidirectional text

Lotion treats Arabic as a first-class language. Direction is resolved at three levels, most specific wins:

1. **Block override** — a block set to `ltr` or `rtl` always uses it.
2. **Page direction** — every page has a direction mode `auto | ltr | rtl | inherit`. It applies to all blocks in that page that have no override, including nested blocks at any depth and the page's title. `inherit` (the default for a new page) takes the mode of the nearest ancestor page; the root falls back to the workspace default in Settings (AC-WS-007).
3. **Auto** — when the effective mode is `auto`, each block takes its direction from its own first strong character (Unicode bidi P2/P3); a block with no strong character follows the previous sibling, else the page's UI direction.

| ID | Requirement | Oracle | Rel |
|---|---|---|---|
| AC-BIDI-001 | Auto RTL: with mode `auto`, a block whose first strong character is Arabic renders `dir="rtl"`, Latin renders `dir="ltr"`, and direction updates live while typing the first character. | Unit on resolver + E2E. | 0.1 |
| AC-BIDI-002 | Per-block override: the block menu and keyboard set a block to LTR, RTL, or back to "follow page". The override is stored on the block, persists, and is undoable. | Unit + E2E. | 0.1 |
| AC-BIDI-003 | Page-level direction: the page menu offers "Text direction: Auto / Right-to-left / Left-to-right / Same as parent page". Changing it re-renders every non-overridden block on the page (any depth) in one step, within budget P-PAGE for 2,000 blocks. | E2E on 2,000-block page. | 0.1 |
| AC-BIDI-004 | "Apply to all blocks on this page" (in the same menu) also clears every block override on the page, so the whole page follows the chosen mode. It is one undoable command and shows the number of overrides cleared. | Unit: N overrides → 0, single undo restores N. | 0.1 |
| AC-BIDI-005 | Opening any block's sub-page (zoom into a block with children, or a sub-page link) shows that page with its own effective mode; sub-pages set to "Same as parent page" follow changes to the parent page immediately. | E2E: parent RTL → child inherits; parent switched to LTR → child switches. | 0.1 |
| AC-BIDI-006 | Search normalizes Arabic (alef variants, ta marbuta/ha, ya/alef maqsura, removes tashkeel and tatweel, Arabic-Indic digits). | Unit fixtures. | 0.1 |
| AC-BIDI-007 | Mirroring: list markers, numbering, to-do boxes, toggle arrows, callout icons, drag handles, indentation guides and the block menu sit on the leading side of each block's own direction. | Visual regression on our own UI, LTR/RTL/mixed fixtures. | 0.1 |
| AC-BIDI-008 | Caret and selection: arrow keys follow visual order within mixed text; Home/End go to logical line start/end; Shift+Arrow selection across an LTR and an RTL block selects the expected logical range. | E2E. | 0.1 |
| AC-BIDI-009 | Keyboard: Ctrl+Right Shift sets the current block (or selected blocks) to RTL, Ctrl+Left Shift to LTR, on Windows and Linux. | E2E. | 0.1 |
| AC-BIDI-010 | Numbered lists in RTL blocks can use Arabic-Indic digits (page setting: Western / Arabic-Indic). | Unit. | 0.1 |
| AC-BIDI-011 | Database direction: titles and text cells use auto direction per cell; the database page's direction mode sets column order (RTL mirrors the table, board columns and calendar weekday order). | E2E. | 0.1 |
| AC-BIDI-012 | Paste and import keep direction: pasted HTML `dir` attributes become block overrides only when they differ from the page's effective mode; Markdown export writes `<div dir="rtl">` only for overridden blocks. | Unit fixtures. | 0.1 |
| AC-BIDI-013 | Sidebar and quick-find items render each title in its own auto direction, independent of UI direction. | Unit. | 0.1 |
| AC-BIDI-014 | Full Arabic UI language with mirrored application chrome. | — | 0.2 |
| AC-BIDI-015 | Arabic spellcheck available when the OS dictionary exists (Chromium spellchecker `ar`). | E2E on Windows. | 0.1 |

## FT-DB — Advanced databases (the core of Lotion)

Lotion exists to give a local, offline workspace a database layer at least as capable as the best hosted block workspaces: typed properties, relations between databases, rollups, a formula language, many view types over the same data, linked views, templates, and fast filtering at tens of thousands of rows. These requirements are **0.1** unless tagged otherwise.

**Model.** A database is a page whose child pages are its rows. Every row is a full page (it can hold any blocks). Inline and full-page databases are the same object rendered differently. A **linked view** is a view block placed in any page that points at a database elsewhere.

### Property types

| Type | Notes | Rel |
|---|---|---|
| Title | One per database; row page title | 0.1 |
| Text | Rich text (marks, mentions) | 0.1 |
| Number | Formats: plain, comma, percent, currency (user-chosen symbol), with decimals; display as number, bar, or ring | 0.1 |
| Select / Multi-select | Options with our own color palette; reorder, rename, merge options | 0.1 |
| Status | Options grouped into To-do / In progress / Complete | 0.1 |
| Date | Start, optional end, optional time and time zone; reminder flag (local notification) | 0.1 |
| Checkbox, URL, Email, Phone | | 0.1 |
| Files & media | Local files copied into the workspace assets | 0.1 |
| Person | From a local "People" list (no accounts) | 0.1 |
| Relation | Link rows to rows of the same or another database; one-way or two-way (synced property on the other side); optional limit to 1 row | 0.1 |
| Rollup | Aggregate a property across related rows (see functions below) | 0.1 |
| Formula | Expression over properties, relations and rollups (language in 06-FORMULA) | 0.1 |
| Unique ID | Auto-increment with optional prefix (e.g. `TASK-42`), never reused | 0.1 |
| Created time / Last edited time | System | 0.1 |
| Button | Runs a local action list: set properties, add a row to a database, open a page | 0.2 |

### Requirements

| ID | Requirement | Oracle | Rel |
|---|---|---|---|
| AC-DB-001 | Create a database as a full page or inline (`/database`), empty or from CSV. | E2E. | 0.1 |
| AC-DB-002 | Add, rename, reorder, hide, duplicate, delete properties; change type with defined, previewed, undoable conversions (03-DATA-MODEL). | Unit per conversion pair. | 0.1 |
| AC-DB-003 | Relation: create one-way or two-way; editing either side updates the other in the same transaction; deleting a row removes it from all relations; self-relations supported (parent/child, dependencies). | Unit: invariants after random edits (property-based). | 0.1 |
| AC-DB-004 | Rollup over a relation with functions: show original, count all, count values, count unique, count empty/not empty, percent empty/not empty, sum, average, median, min, max, range, earliest/latest date, date range, checked/unchecked counts and percents. | Unit vs reference evaluator. | 0.1 |
| AC-DB-005 | Formulas: typed language (06-FORMULA) with live error messages, autocomplete of properties and functions, and results usable in filters, sorts, groups and other formulas/rollups. | Unit: function table + golden tests. | 0.1 |
| AC-DB-006 | Computed values (formula, rollup, two-way relation) are consistent after every command: changing a row recomputes all dependents across databases before the command returns. Cycles are rejected at definition time. | Property-based: random edits → recompute-from-scratch equals incremental. | 0.1 |
| AC-DB-007 | Views: table, board, list, gallery, calendar, timeline. Each database has any number of named views with independent filters, sorts, grouping, visible properties, property order and widths. | E2E per view. | 0.1 |
| AC-DB-008 | Table view: inline editing, keyboard navigation (arrows, Enter, Esc, Tab), multi-cell selection, copy/paste of cell ranges to/from spreadsheet TSV, column resize, freeze first column, wrap cells toggle, add row at end, open row as side peek / center peek / full page. | E2E. | 0.1 |
| AC-DB-009 | Board view grouped by select, multi-select, status, person, checkbox, relation, or date (by day/week/month); drag cards between groups updates the value; manual card order persists per view; card preview (cover, properties). | E2E. | 0.1 |
| AC-DB-010 | Gallery view with card preview from page cover, first image, or file property; card size S/M/L. | E2E. | 0.1 |
| AC-DB-011 | Calendar view by any date property or date formula; month/week; drag to reschedule; multi-day spans. | E2E. | 0.1 |
| AC-DB-012 | Timeline view by start/end dates; zoom day/week/month/quarter/year; drag/resize bars; optional table pane; dependency arrows from a self-relation ("blocked by"). | E2E. | 0.1 |
| AC-DB-013 | Filters: per-type operators (text: is, is not, contains, does not contain, starts with, ends with, is empty/not empty; number: =, ≠, >, <, ≥, ≤; date: is, before, after, on or before, on or after, between, relative (today, past/next N days/weeks/months, this week/month/year); select/status/person: is, is not, contains any, contains all; checkbox; relation: contains, does not contain, is empty; rollup and formula by their result type). Filter groups with AND/OR nested up to 3 levels. | Unit: SQL compiler vs reference evaluator (property-based). | 0.1 |
| AC-DB-014 | Sorts: multiple keys, any property including formula/rollup; stable; locale-aware collation for Arabic and English; manual order when no sort. | Unit. | 0.1 |
| AC-DB-015 | Group by (table, list, gallery, timeline) and sub-group (board); hide empty groups; collapse groups; per-group counts. | E2E. | 0.1 |
| AC-DB-016 | Calculations row per column: count, count values, unique, empty, not empty, percent empty/not empty, sum, average, median, min, max, range, earliest/latest, checked/unchecked; computed over the filtered view, per group too. | Unit vs reference. | 0.1 |
| AC-DB-017 | Linked views: insert a view of an existing database in any page (`/linked view`), with its own filters/sorts; edits write through to the source database. | E2E. | 0.1 |
| AC-DB-018 | Row templates: multiple templates per database (properties preset + page content); a default template; templates may include a nested linked view filtered to "this row". | E2E. | 0.1 |
| AC-DB-019 | Sub-items: a self-relation shown as nested rows (expand/collapse) in table and list views, filter applies to parents or children per setting. | E2E. | 0.1 |
| AC-DB-020 | Filter by "this row" and relative "today" inside templates and linked views. | Unit. | 0.1 |
| AC-DB-021 | Database lock: prevents schema and view edits while allowing row edits. | E2E. | 0.1 |
| AC-DB-022 | Performance: table view of 10,000 rows with 3 relations, 3 rollups, 5 formulas filters+sorts within P-DBVIEW; editing a cell that feeds 1,000 dependents recomputes within P-RECOMP. | Perf test. | 0.1 |
| AC-DB-023 | Database search box per view (text match over visible text-like properties). | E2E. | 0.1 |
| AC-DB-024 | Duplicate database (with or without rows); move a database between pages; turn a table block into a database. | E2E. | 0.1 |
| AC-DB-025 | Page history for row pages and database schema (restore a version within 30 days, from change_log snapshots). | E2E. | 0.2 |
| AC-DB-026 | Chart view (bar, line, donut) over a view's groups. | — | 0.2 |
| AC-DB-027 | Local automations: when a property changes or a row is added, run an action list (set properties, add row, add to relation). | — | 0.2 |
| AC-DB-028 | Form view: a local form that adds rows. | — | 0.3 |

## FT-IO — Portable backup, import and export

**DataBackup.lotion** is the portable file that carries the whole workspace from one machine to another (Windows ↔ Ubuntu). Format in 03-DATA-MODEL §Backup format.

| ID | Requirement | Oracle | Rel |
|---|---|---|---|
| AC-IO-001 | "Export everything" writes one `.lotion` file (default name `DataBackup.lotion`) containing all pages, blocks, databases, properties, views, templates, relations, formulas, assets, favorites and settings; the app keeps working while it runs, and the snapshot is transactionally consistent. | Integration: export while a writer runs; restored DB equals snapshot. | 0.1 |
| AC-IO-002 | "Import backup" on another machine (any supported OS) restores the workspace exactly: every row of every table, every asset byte (SHA-256), every link, view and template. | Cross-OS CI job: export on Windows, import on Ubuntu and vice versa, then compare canonical dumps. | 0.1 |
| AC-IO-003 | Import validates before touching anything: manifest version, per-entry SHA-256, SQLite `integrity_check`, schema version (older → migrated forward; newer → refused with a clear message). | Unit with corrupted/truncated/tampered fixtures. | 0.1 |
| AC-IO-004 | Import modes: "Replace this workspace" (current one is backed up first automatically) or "Open as a new workspace". | E2E. | 0.1 |
| AC-IO-005 | Optional passphrase encryption of the backup (AES-256-GCM, key from Argon2id). | Unit: wrong passphrase fails, tamper fails. | 0.1 |
| AC-IO-006 | Scheduled automatic backups to a user-chosen folder (daily, keep N). | E2E with fake clock. | 0.1 |
| AC-IO-007 | Export page (optionally with sub-pages) to Markdown + assets (zip); databases export as CSV plus a row page per Markdown file. | Round-trip fixture. | 0.1 |
| AC-IO-008 | Import Markdown/CSV folders or zip, preserving hierarchy; CSV becomes a database with type inference and preview. | Fixture corpus. | 0.1 |
| AC-IO-009 | Import the user's own Markdown & CSV export zip from another workspace app, reconstructing nested pages, databases, and relations from page-link columns. | Fixture corpus (synthetic, authored by us). | 0.1 |
| AC-IO-010 | Export a page to PDF. | Text extractable. | 0.1 |
| AC-IO-011 | Merge a backup into an existing workspace (conflict-free by UUID, newer `updated_at` wins, report of conflicts). | — | 0.2 |

## FT-SR — Search

| ID | Requirement | Oracle | Rel |
|---|---|---|---|
| AC-SR-001 | Full-text index covers page titles, block text, and text-like property values; updated in the same transaction as the edit. | Unit: edit then search in same tick finds it. | 0.1 |
| AC-SR-002 | Prefix matching, Arabic normalization (AC-BIDI-006), case-folding, diacritic folding for Latin. | Unit. | 0.1 |
| AC-SR-003 | Filters in quick find: only titles, within a page subtree, edited in last N days. | Unit. | 0.1 |

## FT-KB — Keymap (Windows & Linux)

`Ctrl` below; macOS is not a 0.1 target. Rules: never bind `Ctrl+Alt+<printable>` (AltGr on Windows/European layouts); never bind `Ctrl+Alt+Arrow` (OS/graphics-driver reserved); never bind `Super`-based combos; every binding is checked against `event.isComposing === false`; one action per chord per context. `src/shared/keymap.ts` is the single source and a unit test asserts no chord appears twice in the same context.

| Action | Chord | Context |
|---|---|---|
| Bold / Italic / Underline | Ctrl+B / Ctrl+I / Ctrl+U | text |
| Strikethrough | Ctrl+Shift+S | text |
| Inline code | Ctrl+E | text |
| Link | Ctrl+K | text |
| Quick find | Ctrl+P | global |
| New page | Ctrl+N | global |
| Toggle sidebar | Ctrl+\\ | global |
| Undo / Redo | Ctrl+Z / Ctrl+Shift+Z, Ctrl+Y | global |
| Duplicate block | Ctrl+D | block |
| Move block | Ctrl+Shift+Up / Down | block |
| Indent / Outdent | Tab / Shift+Tab | block |
| Turn into H1/H2/H3 | Ctrl+Shift+1 / 2 / 3 | block |
| Turn into to-do / bullet / number | Ctrl+Shift+4 / 5 / 6 | block |
| Toggle to-do checked | Ctrl+Enter | block |
| Select block | Esc | text→block |
| Back / Forward | Alt+Left / Alt+Right | global |
| Find in page | Ctrl+F | global |
| Paste as plain text | Ctrl+Shift+V | text |
| Block direction RTL / LTR | Ctrl+RightShift / Ctrl+LeftShift | block (Windows and Linux) |
| Page direction menu | Ctrl+Shift+D | page |
| Settings | Ctrl+, | global |

AC-KB-001 (0.1): keymap uniqueness test passes and every chord above is covered by an E2E smoke test.

## FT-A11Y — Accessibility

| ID | Requirement | Oracle | Rel |
|---|---|---|---|
| AC-A11Y-001 | Sidebar, editor, quick find, settings and database views have no serious/critical axe-core violations in LTR and RTL. | axe-core in E2E reports zero serious/critical. | 0.1 |
| AC-A11Y-002 | Every action in FT-KB is reachable with keyboard only; focus is always visible. | E2E keyboard-only paths. | 0.1 |
