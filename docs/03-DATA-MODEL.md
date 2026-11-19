# 03 — Data Model

SQLite, `PRAGMA journal_mode=WAL; synchronous=NORMAL; foreign_keys=ON`. IDs are UUIDv7 (time-ordered, generated in the DB process). Times are Unix ms integers.

## Core idea

Everything visible is a **block**. A page is a block of type `page`. A database is a block of type `database` whose child pages are its rows. Hierarchy is a single `parent_id` tree; ordering among siblings uses fractional keys.

## Tables

```sql
CREATE TABLE block (
  id          TEXT PRIMARY KEY,
  type        TEXT NOT NULL,           -- page|database|paragraph|heading1..3|bulleted|numbered|todo|toggle|quote|callout|divider|code|image|page_link
  parent_id   TEXT REFERENCES block(id) ON DELETE RESTRICT,
  page_id     TEXT NOT NULL,           -- nearest ancestor page (self for pages); denormalized for fast page loads
  sort_key    TEXT NOT NULL,           -- fractional index, base-62, see below
  content     TEXT NOT NULL DEFAULT '{}', -- JSON: {text:[runs], checked, language, icon, ...} per type
  plain_text  TEXT NOT NULL DEFAULT '',   -- derived, for FTS & backlinks context
  direction   TEXT CHECK (direction IN ('ltr','rtl')),   -- per-block override; NULL = follow page
  page_dir    TEXT CHECK (page_dir IN ('inherit','auto','ltr','rtl')), -- only on type=page|database; NULL treated as 'inherit'
  trashed_at  INTEGER,                  -- set on the trashed root only
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL,
  version     INTEGER NOT NULL DEFAULT 1
);
CREATE UNIQUE INDEX block_sibling_order ON block(parent_id, sort_key);
CREATE INDEX block_page ON block(page_id);

CREATE TABLE property (               -- schema of a database
  id TEXT PRIMARY KEY,
  database_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  type TEXT NOT NULL,                 -- title|text|number|select|multi_select|status|date|checkbox|url|email|phone|files|person|relation|rollup|formula|unique_id|created_time|edited_time
  config TEXT NOT NULL DEFAULT '{}',  -- options, number format, status groups; relation: {target_db, paired_property_id|null, limit}; rollup: {relation_prop, target_prop, fn}; formula: {source, ast_hash, result_type, time_dependent}; unique_id: {prefix, next}
  sort_key TEXT NOT NULL
);

CREATE TABLE prop_value (             -- one row per (row page, property); typed columns for indexable sort/filter
  row_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  property_id TEXT NOT NULL REFERENCES property(id) ON DELETE CASCADE,
  v_text TEXT, v_num REAL, v_date_start INTEGER, v_date_end INTEGER, v_bool INTEGER,
  PRIMARY KEY (row_id, property_id)
);
CREATE TABLE prop_option_value (      -- select / multi_select / status
  row_id TEXT NOT NULL, property_id TEXT NOT NULL, option_id TEXT NOT NULL,
  PRIMARY KEY (row_id, property_id, option_id),
  FOREIGN KEY (row_id, property_id) REFERENCES prop_value(row_id, property_id) ON DELETE CASCADE
);
CREATE INDEX pv_num  ON prop_value(property_id, v_num);
CREATE INDEX pv_text ON prop_value(property_id, v_text);
CREATE INDEX pv_date ON prop_value(property_id, v_date_start);

CREATE TABLE relation_edge (          -- one row per link; two-way relations write both directions in one transaction
  property_id TEXT NOT NULL REFERENCES property(id) ON DELETE CASCADE,
  from_row TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  to_row   TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  sort_key TEXT NOT NULL,             -- user order of related rows in the cell
  PRIMARY KEY (property_id, from_row, to_row)
);
CREATE INDEX rel_reverse ON relation_edge(property_id, to_row);

CREATE TABLE computed_value (         -- materialized formula & rollup results (non-time-dependent)
  row_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  property_id TEXT NOT NULL REFERENCES property(id) ON DELETE CASCADE,
  v_type TEXT NOT NULL,               -- number|text|boolean|date|list|error|empty
  v_text TEXT, v_num REAL, v_date_start INTEGER, v_date_end INTEGER, v_bool INTEGER, v_json TEXT,
  PRIMARY KEY (row_id, property_id)
);
CREATE INDEX cv_num  ON computed_value(property_id, v_num);
CREATE INDEX cv_text ON computed_value(property_id, v_text);
CREATE INDEX cv_date ON computed_value(property_id, v_date_start);

CREATE TABLE prop_dependency (        -- edges of the property dependency graph (DAG, cycles rejected)
  property_id TEXT NOT NULL REFERENCES property(id) ON DELETE CASCADE,   -- the computed property
  depends_on  TEXT NOT NULL REFERENCES property(id) ON DELETE CASCADE,
  via_relation TEXT REFERENCES property(id) ON DELETE CASCADE,            -- NULL = same row
  PRIMARY KEY (property_id, depends_on, via_relation)
);

CREATE TABLE person (id TEXT PRIMARY KEY, name TEXT NOT NULL, email TEXT, color TEXT);
CREATE TABLE row_template (
  id TEXT PRIMARY KEY, database_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  name TEXT NOT NULL, template_page_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE, -- hidden page holding preset content & values
  is_default INTEGER NOT NULL DEFAULT 0, sort_key TEXT NOT NULL
);

CREATE TABLE view (
  id TEXT PRIMARY KEY,
  database_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  host_block_id TEXT REFERENCES block(id) ON DELETE CASCADE, -- set for linked views placed in another page
  name TEXT NOT NULL, kind TEXT NOT NULL,      -- table|board|list|gallery|calendar|timeline
  config TEXT NOT NULL,                        -- JSON: filter AST, sorts, group_by, sub_group_by, visible props, widths, calcs, date props, card preview, locked
  sort_key TEXT NOT NULL
);
CREATE TABLE view_card_order (                -- manual order inside a board column / list
  view_id TEXT NOT NULL REFERENCES view(id) ON DELETE CASCADE,
  group_key TEXT NOT NULL, row_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  sort_key TEXT NOT NULL, PRIMARY KEY (view_id, group_key, row_id)
);

CREATE TABLE link (                   -- mentions & page links; source of backlinks
  source_block_id TEXT NOT NULL REFERENCES block(id) ON DELETE CASCADE,
  target_page_id  TEXT NOT NULL,      -- no FK: target may be trashed/deleted; UI shows "deleted page"
  kind TEXT NOT NULL,                 -- mention|page_link|url_internal
  PRIMARY KEY (source_block_id, target_page_id, kind)
);
CREATE INDEX link_target ON link(target_page_id);

CREATE TABLE unique_id_counter (property_id TEXT PRIMARY KEY REFERENCES property(id) ON DELETE CASCADE, next INTEGER NOT NULL);
CREATE TABLE asset (sha256 TEXT PRIMARY KEY, mime TEXT NOT NULL, bytes INTEGER NOT NULL, created_at INTEGER NOT NULL);
CREATE TABLE favorite (page_id TEXT PRIMARY KEY REFERENCES block(id) ON DELETE CASCADE, sort_key TEXT NOT NULL);

CREATE TABLE change_log (             -- durable, append-only; drives live queries, crash replay, future sync
  seq INTEGER PRIMARY KEY AUTOINCREMENT,
  at INTEGER NOT NULL, client_op_id TEXT, command TEXT NOT NULL, payload TEXT NOT NULL
);
CREATE TABLE undo_entry (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  page_id TEXT NOT NULL, group_id TEXT NOT NULL, stack TEXT NOT NULL CHECK (stack IN ('undo','redo')),
  inverse TEXT NOT NULL, at INTEGER NOT NULL
);
CREATE TABLE ui_state (key TEXT PRIMARY KEY, value TEXT NOT NULL); -- expanded tree nodes, toggle open state, window bounds

CREATE VIRTUAL TABLE fts USING fts5(block_id UNINDEXED, page_id UNINDEXED, text, tokenize='unicode61 remove_diacritics 2');
```

## Text direction resolution (FT-BIDI)

```
effective(block) =
  block.direction                      if not NULL
  else mode = pageMode(block.page_id)
       mode == 'ltr' | 'rtl'  -> mode
       mode == 'auto'         -> firstStrong(block.plain_text) ?? effective(prevSibling) ?? uiDir
pageMode(p) = p.page_dir in ('auto','ltr','rtl') ? p.page_dir
            : p.parent page exists ? pageMode(parent page) : settings.defaultDirection
```

- Resolution runs in the renderer; `pageMode` is computed once per page load and cached, so switching a page's mode is an O(visible blocks) re-render, not a write per block.
- `page.setDirection(pageId, mode)` writes one row. `page.applyDirectionToAll(pageId, mode)` also sets `direction = NULL` on every block with `page_id = pageId` in one transaction; its undo inverse lists the cleared `(id, direction)` pairs.
- Sub-pages with `page_dir` NULL/`inherit` change with their parent without any write (AC-BIDI-005).
- `CREATE INDEX block_dir_override ON block(page_id) WHERE direction IS NOT NULL;` keeps "apply to all" fast.

## Computed values

- On any command that touches a property value, relation edge, row creation/deletion or property definition, the DB process collects the **dirty set** `(row, property)` and walks `prop_dependency` (following `relation_edge` in reverse for `via_relation`) in topological order, recomputing into `computed_value` inside the same transaction (AC-DB-006).
- Defining a formula/rollup that would create a cycle is rejected with the cycle path.
- A property definition change (formula edited, rollup function changed) recomputes that column for all rows in batches of 2,000 inside one transaction; the UI shows progress for > 10k rows.
- Time-dependent formulas are not materialized; filters/sorts on them are evaluated in the DB process with the compiled closure (registered as a SQLite user function) — slower, documented in the UI.
- `scripts/verify-computed` recomputes everything from scratch and diffs against `computed_value`; run in CI after property-based tests and at startup in debug builds.

## Backup format (`.lotion`)

A ZIP container (store method for the DB, deflate for JSON):

```
manifest.json      {format:"lotion-backup", format_version:1, app_version, schema_version,
                    created_at, source_os, encrypted:false|true, kdf:{argon2id params, salt},
                    entries:[{path, bytes, sha256}], counts:{pages, blocks, databases, rows, assets}}
workspace.sqlite   produced by `VACUUM INTO` on a read transaction (consistent snapshot, compacted)
assets/<sha256>.<ext>
```

- Encrypted backups wrap each entry with AES-256-GCM (per-entry nonce; key from Argon2id over the passphrase); `manifest.json` stays readable except `counts`.
- Paths inside are POSIX, no absolute paths, no `..`; import rejects anything else (zip-slip).
- Import writes to a temp directory, verifies hashes and `integrity_check`, migrates forward if older, then atomically swaps the workspace directory (after backing up the current one).
- `ui_state` is excluded by default (machine-specific window bounds), included on request.

## Rich text in `content.text`

Array of runs: `{t:"string", m:[["b"],["i"],["code"],["link","https://…"],["mention",{"page":"<id>"}],["date",{"start":…}],["color","red"]]}`. Marks are a closed set validated by zod. Mentions are also written to `link` in the same transaction.

## Ordering: fractional indexing

`sort_key` is a variable-length base-62 string (algorithm: "fractional-indexing" by Rocicorp, MIT, or an equivalent own implementation). Insert between `a` and `b` is always possible without renumbering; keys grow only logarithmically under repeated same-spot inserts. A background compaction rebalances a sibling list when any key exceeds 64 chars. This removes the integer-midpoint exhaustion found in the Lana review.

## Trash and deletion

`page.trash` sets `trashed_at` on the root only; queries exclude any block whose ancestor chain contains a trashed page via the denormalized `page_id` plus a `trashed_page` set kept in memory (rebuilt at startup). Permanent delete deletes children bottom-up in one transaction (FK is `RESTRICT` on `parent_id` on purpose, to catch accidental orphaning).

## Arabic search normalization

Before indexing and before querying: NFKC; remove tashkeel U+064B–U+065F, U+0670 and tatweel U+0640; map أ إ آ ٱ → ا; ى → ي; ة → ه; ؤ → و; ئ → ي; Arabic-Indic digits → ASCII. Stored only in FTS, never in `content`.

## Property type conversions (AC-DB-002)

| From \ To | text | number | select | multi | date | checkbox |
|---|---|---|---|---|---|---|
| text | — | parse or empty | option per distinct value | split on `,` | parse ISO/locale or empty | "true/yes/1/✓" |
| number | toString | — | option per value | same | empty | ≠0 |
| select | name | parse | — | single | parse | name non-empty |
| multi | joined `, ` | parse first | first | — | empty | any |
| date | ISO | epoch days | ISO option | ISO | — | non-empty |
| checkbox | "Yes"/"" | 1/0 | Yes/No options | same | empty | — |

Conversions are previewed and undoable (old values kept in the undo inverse).

## Filter AST → SQL

```ts
type Filter = {and: Filter[]} | {or: Filter[]} | {prop: string, op: Op, value?: unknown};
```

Compiled to parameterized SQL over `prop_value`/`prop_option_value` with `EXISTS` sub-queries; never string-interpolated. One nesting level in the UI; the compiler supports any depth. Unit tests run each operator against a fixture DB and compare to an in-memory reference evaluator (property-based tests with fast-check).

## Migrations

`src/db/migrations/NNNN_name.sql`, forward-only, recorded in `PRAGMA user_version`. Each migration has a fixture DB from the previous version and a test that migrates it and runs `foreign_key_check` and `integrity_check`.
