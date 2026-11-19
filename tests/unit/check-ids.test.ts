import { describe, expect, it } from 'vitest';
import { check } from '../../scripts/check-ids.js';

// Tags are built at runtime so this file does not tag real ACs itself.
const tag = (id: string) => `@${id}`;

const spec = `
## FT-ED — Editor
| ID | Req | Oracle | Rel |
|---|---|---|---|
| AC-ED-001 | a | b | 0.1 |
| AC-ED-002 | a | b | 0.2 |
## FT-DB — Databases
| AC-DB-001 | a | b | 0.1 |
`;

describe('check-ids', () => {
  it('passes a clean spec and counts untested ACs without failing', () => {
    const r = check({ spec, docs: [], tests: [] });
    expect(r.errors).toEqual([]);
    expect(r.defs.size).toBe(3);
    expect(r.untested).toHaveLength(3);
  });

  it('fails on duplicates, wrong section, and missing release', () => {
    const bad = `${spec}| AC-DB-001 | dup | x | 0.1 |\n| AC-ED-003 | wrong | x | 0.1 |\n| AC-DB-002 | none | x | soon |\n`;
    const r = check({ spec: bad, docs: [], tests: [] });
    expect(r.errors.join('\n')).toMatch(/AC-DB-001 duplicates/);
    expect(r.errors.join('\n')).toMatch(/AC-ED-003 sits under FT-DB/);
    expect(r.errors.join('\n')).toMatch(/AC-DB-002 has no release tag/);
  });

  it('fails on tags and doc references to undefined ACs', () => {
    const r = check({
      spec,
      docs: [{ name: 'docs/x.md', text: 'see AC-DB-009' }],
      tests: [{ name: 't.ts', text: `test('x ${tag('AC-ZZ-001')}', () => {})` }],
    });
    expect(r.errors).toContain('docs/x.md references undefined AC-DB-009');
    expect(r.errors.join('\n')).toMatch(/AC-ZZ-001 \(t\.ts\) names no AC/);
  });

  it('enforces tests only up to the gated release', () => {
    const tests = [{ name: 't.ts', text: `test('ok ${tag('AC-ED-001')}', () => {})` }];
    const r = check({ spec, docs: [], tests, release: '0.1' });
    expect(r.errors).toEqual([`AC-DB-001 (release 0.1) has no test tagged ${tag('AC-DB-001')}`]);
  });
});
