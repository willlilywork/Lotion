import { z } from 'zod';

export const PingArgs = z.object({ nonce: z.string().min(1).max(64) }).strict();
export type PingArgs = z.infer<typeof PingArgs>;

export interface PingResult {
  pong: string;
  /** SQLite library version loaded in the DB process; proves the native module works. */
  sqliteVersion: string;
  /** PID of the DB process; differs from main and renderer. */
  pid: number;
}
