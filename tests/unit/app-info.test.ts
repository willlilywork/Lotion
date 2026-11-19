import { describe, expect, it } from 'vitest';
import {
  DEFAULT_BACKUP_NAME,
  PAGE_DIRECTION_MODES,
  isPageDirectionMode,
} from '../../src/shared/app-info.js';

describe('app-info', () => {
  it('names the default backup DataBackup.lotion', () => {
    expect(DEFAULT_BACKUP_NAME).toBe('DataBackup.lotion');
  });

  it('accepts exactly the four page direction modes', () => {
    expect(PAGE_DIRECTION_MODES).toEqual(['inherit', 'auto', 'ltr', 'rtl']);
    for (const mode of PAGE_DIRECTION_MODES) expect(isPageDirectionMode(mode)).toBe(true);
    for (const bad of ['', 'RTL', 'left', null, undefined, 1]) {
      expect(isPageDirectionMode(bad)).toBe(false);
    }
  });
});
