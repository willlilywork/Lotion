/** IPC channel names shared by main and preload. Only these channels exist. */
export const IPC_APP_INFO = 'lotion:app-info';

export interface AppInfo {
  name: string;
  version: string;
  platform: string;
}

/** The API the preload exposes to the renderer as `window.lotion`. */
export interface LotionBridge {
  appInfo(): Promise<AppInfo>;
}
