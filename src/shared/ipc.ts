/** IPC channel names shared by main and preload. Only these channels exist. */
export const IPC_APP_INFO = 'lotion:app-info';
/** Preload → main: asks for a DB port. Main → preload: carries it. */
export const IPC_DB_CONNECT = 'lotion:db-connect';
export const IPC_DB_PORT = 'lotion:db-port';

export interface AppInfo {
  name: string;
  version: string;
  platform: string;
}

/** The API the preload exposes to the renderer as `window.lotion`. */
export interface LotionBridge {
  appInfo(): Promise<AppInfo>;
  /**
   * Sends one command to the DB process. Resolves with the outcome rather than rejecting,
   * because Error objects lose their fields when they cross the context bridge.
   */
  db(cmd: string, args: unknown): Promise<DbOutcome>;
}

export type DbOutcome =
  { ok: true; result: unknown } | { ok: false; error: { code: string; message: string } };
