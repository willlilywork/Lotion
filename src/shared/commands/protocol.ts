import { z } from 'zod';

/** Every request from a renderer to the DB process carries a correlation id. */
export const RequestEnvelope = z
  .object({
    id: z.number().int().nonnegative(),
    cmd: z.string().min(1).max(64),
    args: z.unknown(),
  })
  .strict();
export type RequestEnvelope = z.infer<typeof RequestEnvelope>;

export type ErrorCode = 'INVALID_REQUEST' | 'UNKNOWN_COMMAND' | 'INVALID_ARGS' | 'INTERNAL';

export type Response =
  | { id: number; ok: true; result: unknown }
  | { id: number; ok: false; error: { code: ErrorCode; message: string } };
