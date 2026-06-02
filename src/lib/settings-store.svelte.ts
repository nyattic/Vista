import type { Language } from './types';

export type Theme = 'system' | 'dark' | 'light';

const TILE_SIZES = [
  { value: 150, label: 'Compact' },
  { value: 180, label: 'Comfortable' },
  { value: 220, label: 'Spacious' }
] as const;

function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw === null ? fallback : (JSON.parse(raw) as T);
  } catch {
    return fallback;
  }
}

class SettingsStore {
  theme = $state<Theme>(load<Theme>('vista.theme', 'system'));
  tileMin = $state<number>(load<number>('vista.tileMin', 180));
  language = $state<Language>(load<Language>('vista.language', 'english'));
  blacklist = $state<string[]>(load<string[]>('vista.blacklist', []));
  downloadDir = $state<string>(load<string>('vista.downloadDir', ''));
  readonly tileSizes = TILE_SIZES;

  setTheme(t: Theme) {
    this.theme = t;
    localStorage.setItem('vista.theme', JSON.stringify(t));
  }

  setTileMin(n: number) {
    this.tileMin = n;
    localStorage.setItem('vista.tileMin', JSON.stringify(n));
  }

  setLanguage(l: Language) {
    this.language = l;
    localStorage.setItem('vista.language', JSON.stringify(l));
  }

  setDownloadDir(dir: string) {
    this.downloadDir = dir;
    localStorage.setItem('vista.downloadDir', JSON.stringify(dir));
  }

  private persistBlacklist() {
    localStorage.setItem('vista.blacklist', JSON.stringify(this.blacklist));
  }

  addBlacklist(tag: string) {
    const t = tag.trim().toLowerCase();
    if (!t || this.blacklist.includes(t)) return;
    this.blacklist = [...this.blacklist, t];
    this.persistBlacklist();
  }

  removeBlacklist(tag: string) {
    this.blacklist = this.blacklist.filter((x) => x !== tag);
    this.persistBlacklist();
  }
}

export const settingsStore = new SettingsStore();
