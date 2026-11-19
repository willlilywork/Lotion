import { app, session, shell, type WebContents } from 'electron';

/** Content-Security-Policy applied to every response the renderer loads (05-ARCH §Security). */
export const CSP = [
  "default-src 'none'",
  "script-src 'self'",
  "style-src 'self'",
  "img-src 'self' data: blob:",
  "font-src 'self'",
  "connect-src 'none'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
  "frame-ancestors 'none'",
].join('; ');

/** True when `target` is the app's own renderer document (same file URL, any hash). */
export function isAppUrl(target: string, appUrl: string): boolean {
  try {
    const a = new URL(target);
    const b = new URL(appUrl);
    return a.protocol === 'file:' && a.pathname === b.pathname;
  } catch {
    return false;
  }
}

/** Lock down sessions and every web contents the app creates. Call before any window opens. */
export function installSecurity(appUrl: () => string): void {
  const ses = session.defaultSession;
  ses.setPermissionRequestHandler((_wc, _perm, callback) => {
    callback(false);
  });
  ses.setPermissionCheckHandler(() => false);
  ses.webRequest.onHeadersReceived((details, callback) => {
    callback({
      responseHeaders: { ...details.responseHeaders, 'Content-Security-Policy': [CSP] },
    });
  });

  app.on('web-contents-created', (_event, contents: WebContents) => {
    contents.on('will-navigate', (event, url) => {
      if (!isAppUrl(url, appUrl())) event.preventDefault();
    });
    contents.on('will-redirect', (event, url) => {
      if (!isAppUrl(url, appUrl())) event.preventDefault();
    });
    contents.on('will-attach-webview', (event) => {
      event.preventDefault();
    });
    contents.setWindowOpenHandler(({ url }) => {
      // External links open in the system browser; nothing opens inside the app.
      if (/^https?:\/\//i.test(url)) void shell.openExternal(url);
      return { action: 'deny' };
    });
  });
}
