import { RequestEnvelope, type Response } from '../shared/commands/protocol.js';
import { PingArgs, type PingResult } from '../shared/commands/ping.js';

export interface DbContext {
  sqliteVersion(): string;
  pid: number;
}

type Handler = (args: unknown, ctx: DbContext) => unknown;

const handlers: Record<string, Handler> = {
  ping: (raw, ctx): PingResult => {
    const { nonce } = PingArgs.parse(raw);
    return { pong: nonce, sqliteVersion: ctx.sqliteVersion(), pid: ctx.pid };
  },
};

/** Validates one inbound message and runs its command. Never throws. */
export function dispatch(message: unknown, ctx: DbContext): Response {
  const env = RequestEnvelope.safeParse(message);
  if (!env.success) {
    const id =
      typeof message === 'object' &&
      message !== null &&
      typeof (message as { id?: unknown }).id === 'number'
        ? (message as { id: number }).id
        : -1;
    return { id, ok: false, error: { code: 'INVALID_REQUEST', message: env.error.message } };
  }
  const { id, cmd, args } = env.data;
  const handler = Object.hasOwn(handlers, cmd) ? handlers[cmd] : undefined;
  if (!handler) return { id, ok: false, error: { code: 'UNKNOWN_COMMAND', message: cmd } };
  try {
    return { id, ok: true, result: handler(args, ctx) };
  } catch (err) {
    if (err instanceof Error && err.name === 'ZodError') {
      return { id, ok: false, error: { code: 'INVALID_ARGS', message: err.message } };
    }
    return { id, ok: false, error: { code: 'INTERNAL', message: String(err) } };
  }
}
