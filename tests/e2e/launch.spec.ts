import {
  _electron as electron,
  expect,
  test,
  type ElectronApplication,
  type Page,
} from '@playwright/test';
import path from 'node:path';

let app: ElectronApplication;
let page: Page;

test.beforeAll(async () => {
  app = await electron.launch({
    args: [path.join(process.cwd(), 'out', 'main', 'main.cjs')],
    env: { ...process.env, ELECTRON_ENABLE_LOGGING: '1' },
  });
  page = await app.firstWindow();
  await page.waitForSelector('body[data-ready="true"]');
});

test.afterAll(async () => {
  // On headless Linux runners Electron can hang on quit (no session D-Bus). Bound the
  // graceful close, then kill, so teardown never masks a real result.
  const proc = app.process();
  const closed = await Promise.race([
    app.close().then(() => true),
    new Promise<false>((r) =>
      setTimeout(() => {
        r(false);
      }, 10_000),
    ),
  ]);
  if (!closed) {
    console.warn('electron did not quit within 10 s; killing');
    proc.kill('SIGKILL');
  }
});

test('launch: window opens and the bridge answers', async () => {
  await expect(page).toHaveTitle('Lotion');
  await expect(page.locator('#app-status')).toContainText('Lotion');
});

test('launch: renderer has no Node access', async () => {
  const globals = await page.evaluate(() => ({
    require: typeof (globalThis as Record<string, unknown>)['require'],
    process: typeof (globalThis as Record<string, unknown>)['process'],
    bridgeKeys: Object.keys(window.lotion),
  }));
  expect(globals).toEqual({ require: 'undefined', process: 'undefined', bridgeKeys: ['appInfo'] });
});

test('launch: CSP blocks inline script and string-eval', async () => {
  // DevTools evaluation bypasses CSP, so the probe goes through the page itself.
  const result = await page.evaluate(async () => {
    const w = window as unknown as Record<string, unknown>;
    const violations: string[] = [];
    document.addEventListener('securitypolicyviolation', (e) =>
      violations.push(e.violatedDirective),
    );
    const s = document.createElement('script');
    s.textContent = 'window.__inlineRan = true';
    document.body.append(s);
    setTimeout('window.__timeoutRan = true' as unknown as () => void, 0);
    await new Promise((r) => setTimeout(r, 200));
    return { inline: w['__inlineRan'] === true, timeout: w['__timeoutRan'] === true, violations };
  });
  expect(result.inline).toBe(false);
  expect(result.timeout).toBe(false);
  expect(result.violations.length).toBeGreaterThanOrEqual(2);
});

test('launch: navigation away from the app is blocked', async () => {
  const before = page.url();
  await page.evaluate(() => {
    window.location.href = 'https://example.com/';
  });
  await page.waitForTimeout(500);
  expect(page.url()).toBe(before);
});

test('launch: window.open creates no window', async () => {
  const opened = await page.evaluate(() => window.open('https://example.com/') === null);
  expect(opened).toBe(true);
  expect(app.windows()).toHaveLength(1);
});

test('launch: sandbox and isolation are on', async () => {
  const prefs = await app.evaluate(({ BrowserWindow }) => {
    const wc = BrowserWindow.getAllWindows()[0]?.webContents;
    const p = (
      wc as unknown as { getLastWebPreferences(): Record<string, unknown> } | undefined
    )?.getLastWebPreferences();
    return p
      ? {
          sandbox: p['sandbox'],
          contextIsolation: p['contextIsolation'],
          nodeIntegration: p['nodeIntegration'],
        }
      : null;
  });
  expect(prefs).toEqual({ sandbox: true, contextIsolation: true, nodeIntegration: false });
});
