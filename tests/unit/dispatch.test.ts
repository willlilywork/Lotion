import { describe, expect, it } from 'vitest';
import { dispatch } from '../../src/db/dispatch.js';

const ctx = { sqliteVersion: () => '3.54.1', pid: 4242 };

describe('dispatch', () => {
  it('answers ping with the nonce, sqlite version and pid', () => {
    expect(dispatch({ id: 1, cmd: 'ping', args: { nonce: 'n' } }, ctx)).toEqual({
      id: 1,
      ok: true,
      result: { pong: 'n', sqliteVersion: '3.54.1', pid: 4242 },
    });
  });

  it('rejects malformed envelopes, keeping the id when there is one', () => {
    expect(dispatch('nope', ctx)).toMatchObject({
      id: -1,
      ok: false,
      error: { code: 'INVALID_REQUEST' },
    });
    expect(dispatch({ id: 7, cmd: 'ping', args: {}, extra: true }, ctx)).toMatchObject({
      id: 7,
      ok: false,
      error: { code: 'INVALID_REQUEST' },
    });
  });

  it('rejects bad args and unknown or inherited command names', () => {
    expect(dispatch({ id: 2, cmd: 'ping', args: { nonce: 5 } }, ctx)).toMatchObject({
      error: { code: 'INVALID_ARGS' },
    });
    for (const cmd of ['nope', 'toString', '__proto__', 'constructor']) {
      expect(dispatch({ id: 3, cmd, args: {} }, ctx)).toMatchObject({
        error: { code: 'UNKNOWN_COMMAND' },
      });
    }
  });
});
