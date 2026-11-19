import { contextBridge, ipcRenderer } from 'electron';
import {
  IPC_APP_INFO,
  IPC_DB_CONNECT,
  IPC_DB_PORT,
  type AppInfo,
  type DbOutcome,
  type LotionBridge,
} from '../shared/ipc.js';
import type { Response } from '../shared/commands/protocol.js';

// The DB port stays inside the preload; the page only gets a request function.
let port: MessagePort | undefined;
const waiting: ((p: MessagePort) => void)[] = [];
const pending = new Map<number, (outcome: DbOutcome) => void>();
let nextId = 1;

ipcRenderer.on(IPC_DB_PORT, (event) => {
  const p = event.ports[0];
  if (!p) return;
  port = p;
  p.onmessage = (e: MessageEvent<Response>) => {
    const res = e.data;
    const resolve = pending.get(res.id);
    if (!resolve) return;
    pending.delete(res.id);
    resolve(res.ok ? { ok: true, result: res.result } : { ok: false, error: res.error });
  };
  p.start();
  for (const w of waiting.splice(0)) w(p);
});

// Ask only after the listener exists, so the port can never arrive unheard.
ipcRenderer.send(IPC_DB_CONNECT);

function getPort(): Promise<MessagePort> {
  return port ? Promise.resolve(port) : new Promise((r) => waiting.push(r));
}

const bridge: LotionBridge = {
  appInfo: () => ipcRenderer.invoke(IPC_APP_INFO) as Promise<AppInfo>,
  db: async (cmd, args) => {
    const p = await getPort();
    const id = nextId++;
    return new Promise<DbOutcome>((resolve) => {
      pending.set(id, resolve);
      p.postMessage({ id, cmd, args });
    });
  },
};

contextBridge.exposeInMainWorld('lotion', Object.freeze(bridge));
