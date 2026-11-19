import { app, BrowserWindow, ipcMain } from 'electron';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { APP_NAME } from '../shared/app-info.js';
import { IPC_APP_INFO, type AppInfo } from '../shared/ipc.js';
import { installSecurity } from './security.js';

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

  void app.whenReady().then(createWindow);

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
  void win.loadURL(rendererUrl);
}
