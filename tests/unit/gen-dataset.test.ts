import { describe, expect, it } from 'vitest';
import Database from 'better-sqlite3';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { generate, key, SCALES } from '../../scripts/gen-dataset.ts';

describe('gen-dataset (T0-07)', () => {
  it('keys sort in insertion order', () => {
    const ks = Array.from({ length: 5000 }, (_, i) => key(i));
    expect([...ks].sort()).toEqual(ks);
  });
  it('is deterministic and hits the requested counts', () => {
    const dir = mkdtempSync(join(tmpdir(), 'gen-'));
    try {
      const small = { ...SCALES.smoke, pages: 50, blocks: 1500, rowsPerDb: 100 };
      const dump = (f: string) => {
        generate(f, small, 7);
        const db = new Database(f, { readonly: true });
        const total = db.prepare('SELECT count(*) FROM block').pluck().get();
        const ar = db
          .prepare(`SELECT count(*) FROM block WHERE plain_text GLOB '*[ء-ي]*'`)
          .pluck()
          .get() as number;
        const h = db
          .prepare(
            `SELECT group_concat(id||sort_key||plain_text, '|') FROM (SELECT * FROM block ORDER BY id)`,
          )
          .pluck()
          .get();
        db.close();
        return { total, ar, h };
      };
      const a = dump(join(dir, 'a.sqlite'));
      const b = dump(join(dir, 'b.sqlite'));
      expect(a.total).toBe(1500);
      expect(a.h).toBe(b.h);
      expect(a.ar / 1500).toBeGreaterThan(0.2);
      expect(a.ar / 1500).toBeLessThan(0.4);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
