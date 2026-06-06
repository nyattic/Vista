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
    this.theme = t;
    localStorage.setItem('vista.theme', JSON.stringify(t));
  }

  setTileMin(n: number) {
    this.tileMin = n;
    localStorage.setItem('vista.tileMin', JSON.stringify(n));
  }

  setGridScalePct(n: number) {
    const scale = Math.max(100, Math.min(200, Math.round(n)));
    this.gridScalePct = scale;
    localStorage.setItem('vista.gridScalePct', JSON.stringify(scale));
  }

  setLanguage(l: Language) {
    this.language = l;
    localStorage.setItem('vista.language', JSON.stringify(l));
  }

  setDownloadDir(dir: string) {
    this.downloadDir = dir;
    localStorage.setItem('vista.downloadDir', JSON.stringify(dir));
  }

  setCacheLimitMb(mb: number) {
    this.cacheLimitMb = mb;
    localStorage.setItem('vista.cacheLimitMb', JSON.stringify(mb));
    void setCacheLimit(mb * 1024 * 1024).catch(() => {});
  }

  setReadingMode(m: ReadingMode) {
    this.readingMode = m;
    localStorage.setItem('vista.readingMode', JSON.stringify(m));
  }

  setReadingDirection(d: ReadingDirection) {
    this.readingDirection = d;
    localStorage.setItem('vista.readingDirection', JSON.stringify(d));
  }

  private persistBlacklist() {
    localStorage.setItem('vista.blacklist', JSON.stringify(this.blacklist));
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
