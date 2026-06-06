import { fetchGalleries, searchGalleries, listFavorites, listHistory, listDownloads } from './api';
import { settingsStore } from './settings-store.svelte';
import { friendlyError } from './errors';
import { parseTag } from './format';
import {
  DEFAULT_PAGE_SIZE,
  MAX_PAGE_SIZE,
  MIN_PAGE_SIZE,
  type Gallery,
  type GalleryPage,
  type GalleryType,
  type SortOrder
} from './types';

export type View = 'browse' | 'favorites' | 'history' | 'downloads';

// Normalize a tag/term to a canonical form so blacklist entries match tags
// regardless of case or underscore-vs-space spelling.
function normalizeTerm(s: string): string {
  return s.toLowerCase().replace(/_/g, ' ').trim();
}

// Whole-token match: a gallery is hidden only when one of its tags (or its
// label without namespace) or a credited name equals a blacklist term exactly.
// Substring matching is deliberately avoided so "anal" can't hide "analog".
function isBlacklisted(g: Gallery, terms: Set<string>): boolean {
  if (!terms.size) return false;
  for (const t of g.tags) {
    if (terms.has(normalizeTerm(t)) || terms.has(normalizeTerm(parseTag(t).label))) return true;
  }
  for (const f of g.artists) if (terms.has(normalizeTerm(f))) return true;
  for (const f of g.groups) if (terms.has(normalizeTerm(f))) return true;
  for (const f of g.series) if (terms.has(normalizeTerm(f))) return true;
  for (const f of g.characters) if (terms.has(normalizeTerm(f))) return true;
  return false;
}

class GalleryStore {
  items = $state<Gallery[]>([]);
  view = $state<View>('browse');
  gtype = $state<GalleryType>('');
  sort = $state<SortOrder>('latest');
  query = $state('');
  activeQuery = $state('');
  page = $state(1);
  pageSize = $state(DEFAULT_PAGE_SIZE);
  totalPages = $state(1);
  total = $state(0);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedId = $state<number | null>(null);
  // Bumped to ask the grid to move DOM focus onto the current selection. Only
  // raised for keyboard-initiated page flips, never for plain loads.
  focusRequest = $state(0);

  // Memoized filtered list. As a $derived it recomputes only when items or the
  // blacklist change, instead of on every read like the previous getter.
  visible: Gallery[] = $derived.by(() => {
    const terms = new Set(settingsStore.blacklist.map(normalizeTerm).filter(Boolean));
    if (!terms.size) return this.items;
    return this.items.filter((g) => !isBlacklisted(g, terms));
  });

  private token = 0;
  private localItems: Gallery[] = [];
  private loadTimer: ReturnType<typeof setTimeout> | undefined;
  private pendingSelect: 'first' | 'last' = 'first';
  private pendingFocus = false;

  get searching(): boolean {
    return this.activeQuery.length > 0;
  }

  get selected(): Gallery | null {
    return this.items.find((g) => g.id === this.selectedId) ?? null;
  }

  private async fetchPage(p: number): Promise<GalleryPage> {
    if (this.view === 'favorites' || this.view === 'history' || this.view === 'downloads') {
      if (p <= 1 || this.localItems.length === 0) {
        if (this.view === 'favorites') {
          this.localItems = await listFavorites();
        } else if (this.view === 'history') {
          this.localItems = await listHistory();
        } else {
          const records = await listDownloads();
          this.localItems = records.map((r) => ({
            ...r.gallery,
            downloadFolder: r.folder,
            downloadFailedPages: r.failedPages
          }));
        }
      }
      const total = this.localItems.length;
      const totalPages = Math.max(1, Math.ceil(total / this.pageSize));
      const page = Math.max(1, Math.min(totalPages, p));
      const start = (page - 1) * this.pageSize;
      const items = this.localItems.slice(start, start + this.pageSize);
      return { items, total, totalPages, page };
    }
    return this.activeQuery
      ? searchGalleries(this.activeQuery, p, this.sort, settingsStore.language, this.pageSize)
      : fetchGalleries(p, this.gtype, this.sort, settingsStore.language, this.pageSize);
  }

  setView(v: View) {
    this.view = v;
    this.query = '';
    this.activeQuery = '';
    this.localItems = [];
    this.load(1);
  }

  // `soft` keeps the current grid visible (no loading spinner) while the next
  // page loads in the background — used for resize-driven page-size changes so
  // the grid doesn't blank and flash on every window resize.
  load(p: number, opts?: { select?: 'first' | 'last'; focus?: boolean; soft?: boolean }) {
    this.page = p;
    this.pendingSelect = opts?.select ?? 'first';
    this.pendingFocus = opts?.focus ?? false;
    if (!opts?.soft) this.loading = true;
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
      if (vis.length) {
        const target = this.pendingSelect === 'last' ? vis[vis.length - 1] : vis[0];
        this.selectedId = target.id;
        if (this.pendingFocus) this.focusRequest++;
      } else {
        this.selectedId = null;
      }
    } catch (e) {
      if (t !== this.token) return;
      this.error = friendlyError(e);
      this.items = [];
      this.total = 0;
      this.totalPages = 1;
      this.selectedId = null;
    } finally {
      if (t === this.token) {
        this.loading = false;
        this.pendingFocus = false;
      }
    }
  }

  goToPage(p: number) {
    if (!Number.isFinite(p)) return;
    const clamped = Math.max(1, Math.min(this.totalPages, Math.floor(p)));
    if (clamped === this.page && !this.error) return;
    this.load(clamped);
  }

  setPageSize(size: number) {
    if (!Number.isFinite(size)) return;
    const next = Math.max(MIN_PAGE_SIZE, Math.min(MAX_PAGE_SIZE, Math.floor(size)));
    if (next === this.pageSize) return;
    const firstIndex = (this.page - 1) * this.pageSize;
    this.pageSize = next;
    const nextPage = Math.floor(firstIndex / next) + 1;
    this.load(nextPage, { soft: true });
  }

  // `edge` is set when the flip is triggered by arrow-key navigation past the
  // edge of the grid, so the landing card is selected and focused.
  next(edge = false) {
    if (this.page < this.totalPages) this.load(this.page + 1, { select: 'first', focus: edge });
  }

  prev(edge = false) {
    if (this.page > 1) this.load(this.page - 1, { select: edge ? 'last' : 'first', focus: edge });
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
