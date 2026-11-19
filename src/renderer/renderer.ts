import type { LotionBridge } from '../shared/ipc.js';

declare global {
  interface Window {
    lotion: LotionBridge;
  }
}

async function boot(): Promise<void> {
  const info = await window.lotion.appInfo();
  const status = document.getElementById('app-status');
  const ping = await window.lotion.db('ping', { nonce: 'boot' });
  const db = ping.ok
    ? `SQLite ${(ping.result as { sqliteVersion: string }).sqliteVersion}`
    : 'DB error';
  if (status) status.textContent = `${info.name} ${info.version} · ${info.platform} · ${db}`;
  document.body.dataset['db'] = ping.ok ? 'ok' : 'error';
  document.body.dataset['ready'] = 'true';
}

void boot();
