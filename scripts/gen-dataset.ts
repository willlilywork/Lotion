// T0-07: deterministic synthetic dataset generator for perf spikes.
// Schema is taken verbatim from the first ```sql block of docs/03-DATA-MODEL.md
// so spikes always measure the documented schema.
//
// Usage: node scripts/gen-dataset.ts <out.sqlite> [--scale smoke|full] [--seed N]
import Database from 'better-sqlite3';
import { readFileSync, rmSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

export type Scale = {
  /** free-standing pages (database rows are counted separately) */
  pages: number;
  /** total blocks including pages, databases and rows */
  blocks: number;
  dbs: number;
  rowsPerDb: number;
  arabicShare: number;
};

export const SCALES: Record<'smoke' | 'full', Scale> = {
  smoke: { pages: 500, blocks: 10_000, dbs: 2, rowsPerDb: 500, arabicShare: 0.3 },
  full: { pages: 50_000, blocks: 1_000_000, dbs: 20, rowsPerDb: 10_000, arabicShare: 0.3 },
};

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

export function schemaSql(): string {
  const md = readFileSync(join(root, 'docs/03-DATA-MODEL.md'), 'utf8');
  const m = /```sql\n([\s\S]*?)```/.exec(md);
  if (!m) throw new Error('no sql block in 03-DATA-MODEL.md');
  return m[1] ?? '';
}

// mulberry32: small, fast, deterministic
function rng(seed: number) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function at<T>(xs: readonly T[], i: number): T {
  const v = xs[i];
  if (v === undefined) throw new Error(`index ${String(i)} out of range`);
  return v;
}

const B62 = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
/** Evenly spaced fractional keys for generated siblings (real inserts use T1-02). */
export function key(i: number): string {
  let s = '';
  let n = i + 1;
  while (n > 0) {
    s = B62.charAt(n % 62) + s;
    n = Math.floor(n / 62);
  }
  return 'a' + s.padStart(4, '0');
}

const EN =
  'project meeting notes design review budget roadmap invoice travel garden recipe research draft summary decision follow up client launch'.split(
    ' ',
  );
const AR =
  'مشروع اجتماع ملاحظات تصميم مراجعة ميزانية خطة فاتورة سفر حديقة وصفة بحث مسودة ملخص قرار متابعة عميل إطلاق'.split(
    ' ',
  );

export function generate(
  out: string,
  scale: Scale,
  seed = 1,
): { ms: number; counts: Record<string, number> } {
  const t0 = performance.now();
  if (existsSync(out)) rmSync(out);
  const db = new Database(out);
  db.pragma('journal_mode = WAL');
  db.pragma('synchronous = OFF'); // generation only
  db.pragma('foreign_keys = OFF'); // bulk load; checked at the end
  db.exec(schemaSql());
  const r = rng(seed);
  const pick = <T>(xs: readonly T[]): T => at(xs, Math.floor(r() * xs.length));
  let idn = 0;
  const id = () => (++idn).toString(36).padStart(8, '0');
  const now = 1_763_000_000_000;
  const text = () => {
    const ar = r() < scale.arabicShare;
    const words = ar ? AR : EN;
    const n = 3 + Math.floor(r() * 12);
    return Array.from({ length: n }, () => pick(words)).join(' ');
  };

  const insBlock = db.prepare(
    `INSERT INTO block (id,type,parent_id,page_id,sort_key,content,plain_text,direction,page_dir,created_at,updated_at)
     VALUES (?,?,?,?,?,?,?,?,?,?,?)`,
  );
  const insFts = db.prepare('INSERT INTO fts (block_id,page_id,text) VALUES (?,?,?)');
  const insProp = db.prepare(
    'INSERT INTO property (id,database_id,name,type,config,sort_key) VALUES (?,?,?,?,?,?)',
  );
  const insPv = db.prepare(
    'INSERT INTO prop_value (row_id,property_id,v_text,v_num,v_date_start,v_bool) VALUES (?,?,?,?,?,?)',
  );
  const insOpt = db.prepare(
    'INSERT INTO prop_option_value (row_id,property_id,option_id) VALUES (?,?,?)',
  );
  const insEdge = db.prepare(
    'INSERT OR IGNORE INTO relation_edge (property_id,from_row,to_row,sort_key) VALUES (?,?,?,?)',
  );

  const childCount = new Map<string | null, number>();
  const block = (
    type: string,
    parent: string | null,
    pageId: string | null,
    _sk: string,
    t: string,
    pageDir: string | null,
  ) => {
    const bid = id();
    const n = childCount.get(parent) ?? 0;
    childCount.set(parent, n + 1);
    const sk = key(n); // unique per parent, document order
    const pid = pageId ?? bid;
    insBlock.run(
      bid,
      type,
      parent,
      pid,
      sk,
      JSON.stringify({ text: [{ t }] }),
      t,
      null,
      pageDir,
      now,
      now,
    );
    if (t) insFts.run(bid, pid, t);
    return bid;
  };

  const counts = { pages: 0, blocks: 0, dbs: 0, rows: 0, edges: 0 };
  const dbRows = scale.dbs * scale.rowsPerDb;
  const freePages = scale.pages; // pages outside databases; DB rows are pages too and come on top
  const contentTypes = [
    'paragraph',
    'paragraph',
    'paragraph',
    'bulleted',
    'todo',
    'heading2',
    'numbered',
    'quote',
  ];

  db.transaction(() => {
    // page tree: roots of ~20 children each, depth grows naturally
    const pages: string[] = [];
    for (let i = 0; i < freePages; i++) {
      const parent = i < 20 ? null : pick(pages);
      const ar = r() < scale.arabicShare;
      const p = block('page', parent, null, key(i), text(), ar ? 'auto' : null);
      pages.push(p);
    }
    counts.pages = freePages;
    // databases with rows, properties, values, relations to the previous db
    const dbIds: { id: string; rows: string[]; rel?: string }[] = [];
    for (let d = 0; d < scale.dbs; d++) {
      const dbId = block(
        'database',
        at(pages, d % pages.length),
        null,
        key(1_000_000 + d),
        `DB ${String(d)}`,
        null,
      );
      const props = {
        title: id(),
        num: id(),
        date: id(),
        done: id(),
        status: id(),
        rel: d > 0 ? id() : null,
      };
      insProp.run(props.title, dbId, 'Name', 'title', '{}', key(0));
      insProp.run(props.num, dbId, 'Amount', 'number', '{}', key(1));
      insProp.run(props.date, dbId, 'Due', 'date', '{}', key(2));
      insProp.run(props.done, dbId, 'Done', 'checkbox', '{}', key(3));
      insProp.run(
        props.status,
        dbId,
        'Status',
        'status',
        '{"options":["todo","doing","done"]}',
        key(4),
      );
      if (props.rel)
        insProp.run(
          props.rel,
          dbId,
          'Link',
          'relation',
          JSON.stringify({ target_db: at(dbIds, d - 1).id }),
          key(5),
        );
      const rows: string[] = [];
      for (let i = 0; i < scale.rowsPerDb; i++) {
        const t = text();
        const row = block('page', dbId, null, key(i), t, null);
        rows.push(row);
        insPv.run(row, props.title, t, null, null, null);
        insPv.run(row, props.num, null, Math.round(r() * 100_000) / 100, null, null);
        insPv.run(row, props.date, null, null, now + Math.floor(r() * 365) * 86_400_000, null);
        insPv.run(row, props.done, null, null, null, r() < 0.5 ? 1 : 0);
        insPv.run(row, props.status, null, null, null, null);
        insOpt.run(row, props.status, pick(['todo', 'doing', 'done']));
        if (props.rel) {
          insPv.run(row, props.rel, null, null, null, null);
          const prev = at(dbIds, d - 1).rows;
          const k = 1 + Math.floor(r() * 3);
          for (let j = 0; j < k; j++) {
            insEdge.run(props.rel, row, pick(prev), key(j));
            counts.edges++;
          }
        }
      }
      dbIds.push({ id: dbId, rows });
      counts.rows += rows.length;
    }
    counts.dbs = scale.dbs;
    // content blocks: spread over free pages, some nested one level
    const allPages = pages;
    const remaining = scale.blocks - freePages - scale.dbs - dbRows;
    const perPage = new Map<string, number>();
    const lastChild = new Map<string, string>();
    for (let i = 0; i < remaining; i++) {
      const pg = at(allPages, Math.floor(Math.pow(r(), 2) * allPages.length)); // skewed: some big pages
      const n = perPage.get(pg) ?? 0;
      perPage.set(pg, n + 1);
      const nest = r() < 0.15 && lastChild.has(pg);
      const parent = (nest ? lastChild.get(pg) : undefined) ?? pg;
      const b = block(pick(contentTypes), parent, pg, key(n), text(), null);
      if (!nest) lastChild.set(pg, b);
    }
    counts.blocks = scale.blocks;
  })();
  db.pragma('foreign_keys = ON');
  const fk = db.pragma('foreign_key_check') as unknown[];
  if (fk.length) throw new Error(`foreign_key_check: ${String(fk.length)} violations`);
  db.exec('ANALYZE');
  db.close();
  return { ms: Math.round(performance.now() - t0), counts };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const args = process.argv.slice(2);
  const out = args[0];
  if (!out) {
    console.error('usage: gen-dataset <out.sqlite> [--scale smoke|full] [--seed N]');
    process.exit(2);
  }
  const scaleName = (args[args.indexOf('--scale') + 1] ?? 'smoke') as 'smoke' | 'full';
  const seed = args.includes('--seed') ? Number(args[args.indexOf('--seed') + 1]) : 1;
  const res = generate(out, SCALES[args.includes('--scale') ? scaleName : 'smoke'], seed);
  console.log(JSON.stringify(res));
}
