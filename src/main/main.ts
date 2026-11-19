import { app, BrowserWindow, ipcMain } from 'electron';
import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { APP_NAME } from '../shared/app-info.js';
import { IPC_APP_INFO, IPC_DB_CONNECT, type AppInfo } from '../shared/ipc.js';
import { hardenDefaultSession, installSecurity } from './security.js';
import { DbBroker } from './db-broker.js';

const db = new DbBroker(path.join(__dirname, '..', 'db', 'host.cjs'));

const rendererFile = path.join(__dirname, '..', 'renderer', 'index.html');
const rendererUrl = pathToFileURL(rendererFile).toString();

app.enableSandbox();
app.setName(APP_NAME);

if (!app.requestSingleInstanceLock()) {
  app.quit();
} else {
  app.on('second-instance', () => {
    const win = BrowserWindow.getAllWindows()[0];
    if (win) {
      if (win.isMinimized()) win.restore();
      win.focus();
    }
  });

  installSecurity(() => rendererUrl);

  ipcMain.handle(IPC_APP_INFO, (event): AppInfo => {
    if (event.senderFrame?.url !== rendererUrl) throw new Error('untrusted sender');
    return { name: APP_NAME, version: app.getVersion(), platform: process.platform };
  });

  // Each document load asks once; a reload gets a fresh port and the old one dies with its page.
  ipcMain.on(IPC_DB_CONNECT, (event) => {
    if (event.senderFrame?.url !== rendererUrl) return;
    db.connect(event.sender);
  });

  void app.whenReady().then(() => {
    hardenDefaultSession();
    db.start();
    createWindow();
  });

  app.on('will-quit', () => {
    db.stop();
    if (process.env['LOTION_SMOKE'] === '1') console.log('LOTION_SMOKE will-quit');
  });

  app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit();
  });
}

function createWindow(): void {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    show: false,
    title: APP_NAME,
    webPreferences: {
      preload: path.join(__dirname, '..', 'preload', 'preload.cjs'),
      sandbox: true,
      contextIsolation: true,
      nodeIntegration: false,
      nodeIntegrationInWorker: false,
      webSecurity: true,
      allowRunningInsecureContent: false,
      spellcheck: false,
    },
  });
  win.once('ready-to-show', () => {
    win.show();
  });
  // Launch smoke (CI and installed-app checks): wait for the renderer to reach the DB, then
  // quit with exit code 0 on success and 2 on failure. Result also goes to LOTION_SMOKE_OUT.
  if (process.env['LOTION_SMOKE'] === '1') {
    win.webContents.once('did-finish-load', () => {
      void runSmoke(win);
    });
  }
  void win.loadURL(rendererUrl);
}

async function runSmoke(win: BrowserWindow): Promise<void> {
  let status = 'timeout';
  for (let i = 0; i < 100 && status === 'timeout'; i++) {
    const db = (await win.webContents.executeJavaScript(
      'document.body.dataset.db ?? ""',
    )) as string;
    if (db === 'ok' || db === 'error') status = db;
    else await new Promise((r) => setTimeout(r, 100));
  }
  const line = `LOTION_SMOKE ready db=${status}`;
  console.log(line);
  const out = process.env['LOTION_SMOKE_OUT'];
  if (out) writeFileSync(out, `${line}\n`);
  app.exit(status === 'ok' ? 0 : 2);
}
