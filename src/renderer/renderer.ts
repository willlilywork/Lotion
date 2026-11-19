import type { LotionBridge } from '../shared/ipc.js';

declare global {
  interface Window {
    lotion: LotionBridge;
  }
}

async function boot(): Promise<void> {
  const info = await window.lotion.appInfo();
  const status = document.getElementById('app-status');
  if (status) status.textContent = `${info.name} ${info.version} · ${info.platform}`;
  document.body.dataset['ready'] = 'true';
}

void boot();
