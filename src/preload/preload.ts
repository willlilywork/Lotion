import { contextBridge, ipcRenderer } from 'electron';
import { IPC_APP_INFO, type AppInfo, type LotionBridge } from '../shared/ipc.js';

const bridge: LotionBridge = {
  appInfo: () => ipcRenderer.invoke(IPC_APP_INFO) as Promise<AppInfo>,
};

contextBridge.exposeInMainWorld('lotion', Object.freeze(bridge));
