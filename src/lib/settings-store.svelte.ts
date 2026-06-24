import { setCacheLimit } from './api';
import { LANGUAGES, type Language } from './types';

export type Theme = 'system' | 'dark' | 'light';
export type ReadingMode = 'continuous' | 'paged' | 'spread';
export type ReadingDirection = 'ltr' | 'rtl';

const TILE_SIZES = [
  { value: 150, label: 'Compact' },
  { value: 180, label: 'Comfortable' },
  { value: 220, label: 'Spacious' }
] as const;

const GRID_SCALE_OPTIONS = [
  { value: 100, label: '100%' },
  { value: 125, label: '125%' },
  { value: 150, label: '150%' },
  { value: 175, label: '175%' },
  { value: 200, label: '200%' }
] as const;

const CACHE_LIMITS = [
  { value: 512, label: '512 MB' },
  { value: 1024, label: '1 GB' },
  { value: 2048, label: '2 GB' },
  { value: 5120, label: '5 GB' },
  { value: 0, label: 'Unlimited' }
] as const;

function load<T>(key: string, fallback: T, validate: (v: unknown) => v is T): T {
  try {
    const raw = localStorage.getItem(key);
    if (raw === null) return fallback;
    const parsed: unknown = JSON.parse(raw);
    return validate(parsed) ? parsed : fallback;
  } catch {
    return fallback;
  }
}

function clampNumber(value: number, min: number, max: number, fallback: number): number {
  if (!Number.isFinite(value)) return fallback;
  return Math.max(min, Math.min(max, Math.round(value)));
}

function persist<T>(key: string, value: T) {
  localStorage.setItem(key, JSON.stringify(value));
}

const isTheme = (v: unknown): v is Theme => v === 'system' || v === 'dark' || v === 'light';
const isLanguage = (v: unknown): v is Language => LANGUAGES.some((l) => l.value === v);
const isTileMin = (v: unknown): v is number =>
  typeof v === 'number' && Number.isFinite(v) && v >= 120 && v <= 400;
const isGridScalePct = (v: unknown): v is number =>
  typeof v === 'number' && Number.isFinite(v) && v >= 100 && v <= 200;
const isCacheLimit = (v: unknown): v is number =>
  typeof v === 'number' && Number.isFinite(v) && v >= 0 && v <= 20_480;
const isStringArray = (v: unknown): v is string[] =>
  Array.isArray(v) &&
  v.length <= 200 &&
  v.every((x) => typeof x === 'string' && x.length <= 200);
const isString = (v: unknown): v is string => typeof v === 'string' && v.length <= 4096;
const isReadingMode = (v: unknown): v is ReadingMode =>
  v === 'continuous' || v === 'paged' || v === 'spread';
const isReadingDirection = (v: unknown): v is ReadingDirection => v === 'ltr' || v === 'rtl';

class SettingsStore {
  theme = $state<Theme>(load<Theme>('vista.theme', 'system', isTheme));
  tileMin = $state<number>(load<number>('vista.tileMin', 180, isTileMin));
  gridScalePct = $state<number>(load<number>('vista.gridScalePct', 100, isGridScalePct));
  language = $state<Language>(load<Language>('vista.language', 'english', isLanguage));
  blacklist = $state<string[]>(load<string[]>('vista.blacklist', [], isStringArray));
  downloadDir = $state<string>(load<string>('vista.downloadDir', '', isString));
  cacheLimitMb = $state<number>(load<number>('vista.cacheLimitMb', 2048, isCacheLimit));
  readingMode = $state<ReadingMode>(
    load<ReadingMode>('vista.readingMode', 'continuous', isReadingMode)
  );
  readingDirection = $state<ReadingDirection>(
    load<ReadingDirection>('vista.readingDirection', 'ltr', isReadingDirection)
  );
  readonly tileSizes = TILE_SIZES;
  readonly gridScaleOptions = GRID_SCALE_OPTIONS;
  readonly cacheLimits = CACHE_LIMITS;

  get effectiveTileMin() {
    return Math.round((this.tileMin * this.gridScalePct) / 100);
  }

  init() {
    void setCacheLimit(this.cacheLimitMb * 1024 * 1024).catch(() => {});
  }

  setTheme(t: Theme) {
    if (!isTheme(t)) return;
    this.theme = t;
    persist('vista.theme', t);
  }

  setTileMin(n: number) {
    const next = clampNumber(n, 120, 400, this.tileMin);
    this.tileMin = next;
    persist('vista.tileMin', next);
  }

  setGridScalePct(n: number) {
    const scale = clampNumber(n, 100, 200, this.gridScalePct);
    this.gridScalePct = scale;
    persist('vista.gridScalePct', scale);
  }

  setLanguage(l: Language) {
    if (!isLanguage(l)) return;
    this.language = l;
    persist('vista.language', l);
  }

  setDownloadDir(dir: string) {
    if (!isString(dir) || /[\u0000-\u001f\u007f]/.test(dir)) return;
    const next = dir.trim();
    this.downloadDir = next;
    persist('vista.downloadDir', next);
  }

  setCacheLimitMb(mb: number) {
    const next = clampNumber(mb, 0, 20_480, this.cacheLimitMb);
    this.cacheLimitMb = next;
    persist('vista.cacheLimitMb', next);
    void setCacheLimit(next * 1024 * 1024).catch(() => {});
  }

  setReadingMode(m: ReadingMode) {
    if (!isReadingMode(m)) return;
    this.readingMode = m;
    persist('vista.readingMode', m);
  }

  setReadingDirection(d: ReadingDirection) {
    if (!isReadingDirection(d)) return;
    this.readingDirection = d;
    persist('vista.readingDirection', d);
  }

  private persistBlacklist() {
    persist('vista.blacklist', this.blacklist);
  }

  addBlacklist(tag: string) {
    const t = tag.trim().toLowerCase();
    if (!t || t.length > 200 || this.blacklist.includes(t) || this.blacklist.length >= 200) return;
    this.blacklist = [...this.blacklist, t];
    this.persistBlacklist();
  }

  removeBlacklist(tag: string) {
    this.blacklist = this.blacklist.filter((x) => x !== tag);
    this.persistBlacklist();
  }
}

export const settingsStore = new SettingsStore();
