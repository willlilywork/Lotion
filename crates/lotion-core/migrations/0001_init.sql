-- Generated from the first ```sql block of docs/03-DATA-MODEL.md.
-- A unit test fails if the two drift apart; edit the doc, then regenerate.
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
