import { fetchGalleries, searchGalleries, listFavorites, listHistory } from './api';
import { settingsStore } from './settings-store.svelte';
import { parseTag } from './format';
import { PAGE_SIZE, type Gallery, type GalleryPage, type GalleryType, type SortOrder } from './types';

export type View = 'browse' | 'favorites' | 'history';

function isBlacklisted(g: Gallery, blacklist: string[]): boolean {
  if (!blacklist.length) return false;
  const tagHit = g.tags.some((t) => {
    const raw = t.toLowerCase();
    const label = parseTag(t).label.toLowerCase();
    return blacklist.some((b) => raw.includes(b) || label.includes(b));
  });
  if (tagHit) return true;
  const fields = [...g.artists, ...g.groups, ...g.series, ...g.characters];
  return fields.some((f) => {
    const v = f.toLowerCase();
    return blacklist.some((b) => v.includes(b));
  });
}

class GalleryStore {
  items = $state<Gallery[]>([]);
  view = $state<View>('browse');
  gtype = $state<GalleryType>('');
  sort = $state<SortOrder>('latest');
  query = $state('');
  activeQuery = $state('');
  page = $state(1);
  totalPages = $state(1);
  total = $state(0);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedId = $state<number | null>(null);

  private token = 0;
  private localItems: Gallery[] = [];
  private loadTimer: ReturnType<typeof setTimeout> | undefined;

  get searching(): boolean {
    return this.activeQuery.length > 0;
  }

  get visible(): Gallery[] {
    return this.items.filter((g) => !isBlacklisted(g, settingsStore.blacklist));
  }

  get selected(): Gallery | null {
    return this.items.find((g) => g.id === this.selectedId) ?? null;
  }

  private async fetchPage(p: number): Promise<GalleryPage> {
    if (this.view === 'favorites' || this.view === 'history') {
      if (p <= 1 || this.localItems.length === 0) {
        this.localItems = this.view === 'favorites' ? await listFavorites() : await listHistory();
      }
      const total = this.localItems.length;
      const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));
      const page = Math.max(1, Math.min(totalPages, p));
      const start = (page - 1) * PAGE_SIZE;
      const items = this.localItems.slice(start, start + PAGE_SIZE);
      return { items, total, totalPages, page };
    }
    return this.activeQuery
      ? searchGalleries(this.activeQuery, p, settingsStore.language)
      : fetchGalleries(p, this.gtype, this.sort, settingsStore.language);
  }

  setView(v: View) {
    this.view = v;
    this.query = '';
    this.activeQuery = '';
    this.localItems = [];
    this.load(1);
  }

  load(p: number) {
    this.page = p;
    this.loading = true;
    this.error = null;
    clearTimeout(this.loadTimer);
    this.loadTimer = setTimeout(() => void this.runLoad(p), 130);
  }

  private async runLoad(p: number) {
    const t = ++this.token;
    try {
      const res = await this.fetchPage(p);
      if (t !== this.token) return;
      this.items = res.items;
      this.page = res.page;
      this.total = res.total;
      this.totalPages = res.totalPages;
      const vis = this.visible;
      this.selectedId = vis.length ? vis[0].id : null;
    } catch (e) {
      if (t !== this.token) return;
      this.error = String(e);
      this.items = [];
      this.total = 0;
      this.totalPages = 1;
      this.selectedId = null;
    } finally {
      if (t === this.token) this.loading = false;
    }
  }

  goToPage(p: number) {
    if (!Number.isFinite(p)) return;
    const clamped = Math.max(1, Math.min(this.totalPages, Math.floor(p)));
    if (clamped === this.page && !this.error) return;
    this.load(clamped);
  }

  next() {
    if (this.page < this.totalPages) this.load(this.page + 1);
  }

  prev() {
    if (this.page > 1) this.load(this.page - 1);
  }

  setType(t: GalleryType) {
    this.view = 'browse';
    this.gtype = t;
    this.query = '';
    this.activeQuery = '';
    this.load(1);
  }

  setSort(s: SortOrder) {
    this.view = 'browse';
    this.sort = s;
    this.query = '';
    this.activeQuery = '';
    this.load(1);
  }

  submitSearch() {
    this.view = 'browse';
    this.activeQuery = this.query.trim();
    this.load(1);
  }

  applyQuery(q: string) {
    this.view = 'browse';
    this.query = q;
    this.activeQuery = q.trim();
    this.load(1);
  }

  clearSearch() {
    this.query = '';
    this.activeQuery = '';
    this.load(1);
  }

  select(id: number | null) {
    this.selectedId = id;
  }

  moveSelection(delta: number): number | null {
    const vis = this.visible;
    if (!vis.length) return null;
    const idx = vis.findIndex((g) => g.id === this.selectedId);
    const next = Math.max(0, Math.min(vis.length - 1, (idx < 0 ? 0 : idx) + delta));
    const id = vis[next].id;
    this.selectedId = id;
    return id;
  }
}

export const galleryStore = new GalleryStore();
