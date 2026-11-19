// Entry point of the DB utilityProcess. Owns SQLite; talks to renderers over MessagePorts
// handed over by main (02-ARCHITECTURE, ADR-003). Main never sees DB traffic.
import Database from 'better-sqlite3';
import { dispatch, type DbContext } from './dispatch.js';

const probe = new Database(':memory:');
const version = (probe.prepare('select sqlite_version() as v').get() as { v: string }).v;

const ctx: DbContext = { sqliteVersion: () => version, pid: process.pid };

interface PortLike {
  on(event: 'message', listener: (e: { data: unknown }) => void): void;
  postMessage(message: unknown): void;
  start(): void;
}

process.parentPort.on('message', (e) => {
  const msg = e.data as { type?: unknown };
  if (msg.type !== 'connect') return;
  const port = e.ports[0] as unknown as PortLike | undefined;
  if (!port) return;
  port.on('message', (m) => {
    port.postMessage(dispatch(m.data, ctx));
  });
  port.start();
});

process.parentPort.postMessage({ type: 'ready', sqliteVersion: version });
