// T0-07 perf harness. `pnpm perf --smoke` generates a small dataset and runs every probe once
// (proves the harness works; numbers are not judged). `pnpm perf` runs the full reference
// dataset and judges against docs/05-QUALITY.md budgets. Spikes T0-08..T0-10 add probes here.
import Database from 'better-sqlite3';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { generate, SCALES } from './gen-dataset.ts';

type Probe = { id: string; budgetMs: number | null; run: (db: Database.Database) => unknown };

function time<T>(fn: () => T, reps: number): { p50: number; max: number; out: T } {
  const xs: number[] = [];
  let out!: T;
  for (let i = 0; i < reps; i++) {
    const t = performance.now();
    out = fn();
    xs.push(performance.now() - t);
  }
  xs.sort((a, b) => a - b);
  return { p50: xs[Math.floor(xs.length / 2)] ?? NaN, max: xs[xs.length - 1] ?? NaN, out };
}

// Setup lookups are cached so they never count against a probe's time.
const memo = new WeakMap<Database.Database, unknown>();
function largestPage(db: Database.Database): unknown {
  if (!memo.has(db))
    memo.set(
      db,
      db
        .prepare(
          `SELECT page_id FROM block WHERE type!='page' GROUP BY page_id ORDER BY count(*) DESC LIMIT 1`,
        )
        .pluck()
        .get(),
    );
  return memo.get(db);
}

export const probes: Probe[] = [
  {
    id: 'page-load (largest page, all blocks ordered)',
    budgetMs: null, // P-PAGE is a paint budget; DB share recorded in T0-08
    run: (db) => {
      const pg = largestPage(db);
      return db
        .prepare(
          'SELECT id,parent_id,type,sort_key,content FROM block WHERE page_id=? ORDER BY parent_id,sort_key',
        )
        .all(pg).length;
    },
  },
  {
    id: 'P-SEARCH (FTS, Arabic term, first 20)',
    budgetMs: 100,
    run: (db) =>
      db.prepare(`SELECT block_id FROM fts WHERE fts MATCH ? LIMIT 20`).all('ميزانية').length,
  },
  {
    id: 'db-view (filter+sort, first 50)',
    budgetMs: null, // P-DBVIEW proper (rollups, formulas, group) is T0-09
    run: (db) => {
      const dbId = db
        .prepare(`SELECT id FROM block WHERE type='database' LIMIT 1 OFFSET 1`)
        .pluck()
        .get();
      return db
        .prepare(
          `SELECT b.id, n.v_num FROM block b
           JOIN property pn ON pn.database_id=b.parent_id AND pn.type='number'
           JOIN prop_value n ON n.row_id=b.id AND n.property_id=pn.id
           JOIN property pc ON pc.database_id=b.parent_id AND pc.type='checkbox'
           JOIN prop_value c ON c.row_id=b.id AND c.property_id=pc.id
           WHERE b.parent_id=? AND c.v_bool=1 AND n.v_num > 100
           ORDER BY n.v_num DESC LIMIT 50`,
        )
        .all(dbId).length;
    },
  },
];

function main() {
  const smoke = process.argv.includes('--smoke');
  const dir = mkdtempSync(join(tmpdir(), 'lotion-perf-'));
  const file = join(dir, 'ref.sqlite');
  try {
    const gen = generate(file, smoke ? SCALES.smoke : SCALES.full);
    console.log(
      `dataset ${smoke ? 'smoke' : 'full'}: ${JSON.stringify(gen.counts)} in ${String(gen.ms)} ms`,
    );
    const db = new Database(file, { readonly: true });
    let failed = 0;
    for (const p of probes) {
      p.run(db); // warm-up + setup, not timed
      const r = time(() => p.run(db), smoke ? 3 : 15);
      const over = !smoke && p.budgetMs !== null && r.p50 > p.budgetMs;
      if (over) failed++;
      console.log(
        `${over ? 'OVER' : 'ok  '} ${p.id}: p50 ${r.p50.toFixed(1)} ms, max ${r.max.toFixed(1)} ms` +
          `${p.budgetMs ? ` (budget ${String(p.budgetMs)})` : ''} -> ${String(r.out)}`,
      );
      if (smoke && (typeof r.out !== 'number' || r.out === 0)) {
        console.error(`probe returned nothing: ${p.id}`);
        failed++;
      }
    }
    db.close();
    process.exitCode = failed ? 1 : 0;
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}
main();
