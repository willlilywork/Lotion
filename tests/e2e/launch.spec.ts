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
  await app.close();
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

test('launch: CSP blocks eval', async () => {
  const blocked = await page.evaluate(() => {
    try {
      (0, eval)('1');
      return false;
    } catch {
      return true;
    }
  });
  expect(blocked).toBe(true);
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
