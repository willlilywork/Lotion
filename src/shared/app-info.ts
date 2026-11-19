/** Static facts about the application shared by every process. */
export const APP_NAME = 'Lotion';

/** File extension of the portable backup container (ADR-013). */
export const BACKUP_EXTENSION = '.lotion';

/** Default file name offered by "Export everything" (AC-IO-001). */
export const DEFAULT_BACKUP_NAME = `DataBackup${BACKUP_EXTENSION}`;

/** Text-direction modes a page can hold (FT-BIDI, ADR-009). */
export const PAGE_DIRECTION_MODES = ['inherit', 'auto', 'ltr', 'rtl'] as const;
export type PageDirectionMode = (typeof PAGE_DIRECTION_MODES)[number];

export function isPageDirectionMode(value: unknown): value is PageDirectionMode {
  return typeof value === 'string' && (PAGE_DIRECTION_MODES as readonly string[]).includes(value);
}
