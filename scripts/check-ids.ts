// Traceability checker (05-QUALITY §Traceability, T0-06).
// Usage: node scripts/check-ids.ts [--release 0.1]
// Always fails on: duplicate AC ID, AC outside an FT section, AC area not matching its FT,
// test tag or doc reference naming an undefined AC.
// With --release R, also fails on any AC of release ≤ R that no test tags (release gate).
import { readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export interface AcDef {
  id: string;
  ft: string;
  release: string;
  line: number;
}

export interface Report {
  defs: Map<string, AcDef>;
  errors: string[];
  untested: AcDef[];
}

const AC_RE = /AC-[A-Z0-9]+-\d{3}/g;

export function parseSpec(text: string, errors: string[]): Map<string, AcDef> {
  const defs = new Map<string, AcDef>();
  let ft: string | undefined;
  text.split(/\r?\n/).forEach((raw, i) => {
    const line = i + 1;
    const heading = /^##\s+(FT-[A-Z0-9]+)\b/.exec(raw);
    if (heading) {
      ft = heading[1];
      return;
    }
    if (/^##\s/.test(raw)) {
      ft = undefined;
      return;
    }
    const row = /^\|\s*(AC-[A-Z0-9]+-\d{3})\s*\|/.exec(raw);
    if (!row?.[1]) return;
    const id = row[1];
    const cells = raw.split('|').map((c) => c.trim());
    const release = cells[cells.length - 2] ?? '';
    if (!ft) {
      errors.push(`spec:${String(line)} ${id} is not inside an FT section`);
      return;
    }
    const area = id.split('-')[1];
    if (`FT-${area ?? ''}` !== ft) errors.push(`spec:${String(line)} ${id} sits under ${ft}`);
    if (!/^\d+\.\d+\+?$/.test(release))
      errors.push(`spec:${String(line)} ${id} has no release tag`);
    const prev = defs.get(id);
    if (prev) {
      errors.push(`spec:${String(line)} ${id} duplicates line ${String(prev.line)}`);
      return;
    }
    defs.set(id, { id, ft, release, line });
  });
  return defs;
}

/** Test tags look like `@AC-ED-001` anywhere in a test file. */
export function collectTags(files: { name: string; text: string }[]): Map<string, string[]> {
  const tags = new Map<string, string[]>();
  for (const f of files) {
    for (const m of f.text.matchAll(/@(AC-[A-Z0-9]+-\d{3})/g)) {
      const id = m[1] as string;
      tags.set(id, [...(tags.get(id) ?? []), f.name]);
    }
  }
  return tags;
}

function releaseLe(a: string, b: string): boolean {
  const [am = 0, an = 0] = a.replace('+', '').split('.').map(Number);
  const [bm = 0, bn = 0] = b.split('.').map(Number);
  return am < bm || (am === bm && an <= bn);
}

export function check(input: {
  spec: string;
  docs: { name: string; text: string }[];
  tests: { name: string; text: string }[];
  release?: string | undefined;
}): Report {
  const errors: string[] = [];
  const defs = parseSpec(input.spec, errors);
  const tags = collectTags(input.tests);
  for (const [id, where] of tags) {
    if (!defs.has(id)) errors.push(`test tag @${id} (${where.join(', ')}) names no AC`);
  }
  for (const d of input.docs) {
    for (const m of new Set(d.text.match(AC_RE) ?? [])) {
      if (!defs.has(m)) errors.push(`${d.name} references undefined ${m}`);
    }
  }
  const untested = [...defs.values()].filter((d) => !tags.has(d.id));
  if (input.release) {
    for (const d of untested) {
      if (releaseLe(d.release, input.release)) {
        errors.push(`${d.id} (release ${d.release}) has no test tagged @${d.id}`);
      }
    }
  }
  return { defs, errors, untested };
}

function walk(dir: string, exts: string[]): string[] {
  let out: string[] = [];
  for (const name of readdirSync(dir)) {
    const p = path.join(dir, name);
    if (statSync(p).isDirectory()) out = out.concat(walk(p, exts));
    else if (exts.some((e) => p.endsWith(e))) out.push(p);
  }
  return out;
}

function main(): void {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const argv = process.argv.slice(2);
  const ri = argv.indexOf('--release');
  const release = ri >= 0 ? argv[ri + 1] : undefined;
  const read = (p: string) => ({ name: path.relative(root, p), text: readFileSync(p, 'utf8') });
  const specPath = path.join(root, 'docs', '01-PRODUCT-SPEC.md');
  const docs = walk(path.join(root, 'docs'), ['.md'])
    .filter((p) => p !== specPath)
    .map(read);
  const tests = walk(path.join(root, 'tests'), ['.ts']).map(read);
  const r = check({ spec: readFileSync(specPath, 'utf8'), docs, tests, release });
  const tested = r.defs.size - r.untested.length;
  console.log(`check-ids: ${String(r.defs.size)} ACs, ${String(tested)} tagged by tests`);
  for (const e of r.errors) console.error(`  ✗ ${e}`);
  if (r.errors.length > 0) process.exit(1);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) main();
