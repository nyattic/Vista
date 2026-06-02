import * as api from './api';
import type { Gallery } from './types';

export interface PageProgress {
  page: number;
  total: number;
}

class LibraryStore {
  favorites = $state<Set<number>>(new Set());
  downloads = $state<Set<number>>(new Set());
  progress = $state<Record<number, PageProgress>>({});

  async init() {
    try {
      this.favorites = new Set(await api.favoriteIds());
    } catch {
      /* ignore */
    }
    try {
      const rows = await api.allProgress();
      const map: Record<number, PageProgress> = {};
      for (const r of rows) map[r.id] = { page: r.page, total: r.total };
      this.progress = map;
    } catch {
      /* ignore */
    }
    try {
      this.downloads = new Set(await api.downloadIds());
    } catch {
      /* ignore */
    }
  }

  isFavorite(id: number): boolean {
    return this.favorites.has(id);
  }

  isDownloaded(id: number): boolean {
    return this.downloads.has(id);
  }

  markDownloaded(id: number) {
    this.downloads = new Set([...this.downloads, id]);
  }

  markNotDownloaded(id: number) {
    const next = new Set(this.downloads);
    next.delete(id);
    this.downloads = next;
  }

  async toggleFavorite(gallery: Gallery): Promise<boolean> {
    let fav = this.isFavorite(gallery.id);
    try {
      fav = await api.toggleFavorite(gallery);
    } catch {
      return fav;
    }
    const next = new Set(this.favorites);
    if (fav) next.add(gallery.id);
    else next.delete(gallery.id);
    this.favorites = next;
    return fav;
  }

  progressOf(id: number): PageProgress | undefined {
    return this.progress[id];
  }

  percent(id: number): number {
    const p = this.progress[id];
    if (!p || p.total <= 0) return 0;
    return Math.min(100, Math.round((p.page / p.total) * 100));
  }

  async record(gallery: Gallery) {
    try {
      await api.recordView(gallery);
    } catch {
      /* ignore */
    }
  }

  async saveProgress(id: number, page: number, total: number) {
    this.progress = { ...this.progress, [id]: { page, total } };
    try {
      await api.setProgress(id, page, total);
    } catch {
      /* ignore */
    }
  }
}

export const libraryStore = new LibraryStore();
