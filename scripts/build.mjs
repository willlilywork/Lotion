// Bundles main, preload and renderer into out/. Renderer assets are copied as-is.
import { build } from 'esbuild';
import { cp, mkdir, rm } from 'node:fs/promises';

const common = { bundle: true, sourcemap: true, target: 'node22', logLevel: 'warning' };

await rm('out', { recursive: true, force: true });
await Promise.all([
  build({
    ...common,
    entryPoints: ['src/main/main.ts'],
    outfile: 'out/main/main.cjs',
    platform: 'node',
    format: 'cjs',
    external: ['electron'],
  }),
  build({
    ...common,
    entryPoints: ['src/db/host.ts'],
    outfile: 'out/db/host.cjs',
    platform: 'node',
    format: 'cjs',
    external: ['electron', 'better-sqlite3'],
  }),
  build({
    ...common,
    entryPoints: ['src/preload/preload.ts'],
    outfile: 'out/preload/preload.cjs',
    platform: 'node',
    format: 'cjs',
    external: ['electron'],
  }),
  build({
    ...common,
    entryPoints: ['src/renderer/renderer.ts'],
    outfile: 'out/renderer/renderer.js',
    platform: 'browser',
    format: 'iife',
    target: 'chrome140',
  }),
]);
await mkdir('out/renderer', { recursive: true });
await cp('src/renderer/index.html', 'out/renderer/index.html');
await cp('src/renderer/styles.css', 'out/renderer/styles.css');
