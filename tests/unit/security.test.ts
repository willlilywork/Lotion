import { describe, expect, it, vi } from 'vitest';

vi.mock('electron', () => ({ app: {}, session: {}, shell: {} }));
const { CSP, isAppUrl } = await import('../../src/main/security.js');

const APP = 'file:///C:/Program%20Files/Lotion/resources/app/out/renderer/index.html';

describe('isAppUrl', () => {
  it('accepts the app document, with or without a hash', () => {
    expect(isAppUrl(APP, APP)).toBe(true);
    expect(isAppUrl(`${APP}#/page/1`, APP)).toBe(true);
  });
  it('rejects other files, web URLs and garbage', () => {
    expect(isAppUrl('file:///C:/Windows/system32/calc.exe', APP)).toBe(false);
    expect(isAppUrl('https://example.com/', APP)).toBe(false);
    expect(isAppUrl('not a url', APP)).toBe(false);
  });
});

describe('CSP', () => {
  it('forbids eval, inline script and network by default', () => {
    expect(CSP).toContain("default-src 'none'");
    expect(CSP).toContain("script-src 'self'");
    expect(CSP).not.toContain('unsafe-eval');
    expect(CSP).not.toContain('unsafe-inline');
    expect(CSP).toContain("connect-src 'none'");
  });
});
