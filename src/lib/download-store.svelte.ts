import { listen } from '@tauri-apps/api/event';
import { downloadGallery, cancelDownload, defaultDownloadDir } from './api';
import { libraryStore } from './library-store.svelte';
import { settingsStore } from './settings-store.svelte';

export interface DownloadState {
  id: number;
  title: string;
  done: number;
  total: number;
  finished: boolean;
  running: boolean;
  paused: boolean;
  failed?: number;
  failedPages?: number[];
  skipped?: number;
  folder?: string;
  error?: string;
}

interface ProgressEvent {
  id: number;
  done: number;
  total: number;
}

interface DoneEvent {
  id: number;
  folder: string;
  total: number;
  failed: number;
  failedPages: number[];
  skipped: number;
}

interface CancelledEvent {
  id: number;
  done: number;
  total: number;
}

class DownloadStore {
  jobs = $state<Record<number, DownloadState>>({});
  private started = false;

  init() {
    if (this.started) return;
    this.started = true;

    listen<ProgressEvent>('download-progress', (e) => {
      const { id, done, total } = e.payload;
      const prev = this.jobs[id];
      if (!prev) return;
      this.jobs = {
        ...this.jobs,
        [id]: { ...prev, done, total, finished: false, running: true, paused: false }
      };
    });

    listen<DoneEvent>('download-done', (e) => {
      const { id, folder, total, failed, failedPages, skipped } = e.payload;
      const prev = this.jobs[id];
      if (!prev) return;
      libraryStore.markDownloaded(id);
      this.jobs = {
        ...this.jobs,
        [id]: {
          ...prev,
          finished: true,
          running: false,
          paused: false,
          folder,
          failed,
          failedPages,
          skipped,
          total: total ?? prev.total
        }
      };
    });

    listen<CancelledEvent>('download-cancelled', (e) => {
      const { id, done, total } = e.payload;
      const prev = this.jobs[id];
      if (!prev) return;
      this.jobs = {
        ...this.jobs,
        [id]: { ...prev, done, total, finished: false, running: false, paused: true }
      };
    });
  }

  get(id: number): DownloadState | undefined {
    return this.jobs[id];
  }

  get list(): DownloadState[] {
    return Object.values(this.jobs);
  }

  get activeCount(): number {
    return this.list.filter((j) => j.running).length;
  }

  async start(id: number, title: string, pages?: number[]) {
    const cur = this.jobs[id];
    if (cur?.running) return;
    const retryPages = pages?.filter((p) => Number.isFinite(p) && p > 0) ?? [];

    if (retryPages.length === 0 && libraryStore.isDownloaded(id)) {
      this.jobs = {
        ...this.jobs,
        [id]: {
          ...(cur ?? { id, title, done: 0, total: 0 }),
          id,
          title,
          finished: true,
          running: false,
          paused: false,
          error: 'already downloaded'
        }
      };
      return;
    }

    let dir = settingsStore.downloadDir;
    if (!dir) {
      dir = (await defaultDownloadDir()) ?? '';
      if (dir) settingsStore.setDownloadDir(dir);
    }
    if (!dir) {
      this.jobs = {
        ...this.jobs,
        [id]: {
          id,
          title,
          done: 0,
          total: 0,
          finished: true,
          running: false,
          paused: false,
          error: 'no download folder'
        }
      };
      return;
    }

    this.jobs = {
      ...this.jobs,
      [id]: {
        id,
        title,
        done: cur?.done ?? 0,
        total: cur?.total ?? 0,
        finished: false,
        running: true,
        paused: false,
        failedPages: retryPages.length ? retryPages : cur?.failedPages,
        error: undefined
      }
    };
    try {
      const result = await downloadGallery(id, dir, retryPages.length ? retryPages : undefined);
      libraryStore.markDownloaded(id);
      const prev = this.jobs[id];
      this.jobs = {
        ...this.jobs,
        [id]: {
          ...(prev ?? { id, title, done: 0, total: result.total }),
          finished: true,
          running: false,
          paused: false,
          folder: result.folder,
          failed: result.failed,
          failedPages: result.failedPages,
          skipped: result.skipped,
          total: result.total,
          done: result.done
        }
      };
    } catch (e) {
      const prev = this.jobs[id];
      this.jobs = {
        ...this.jobs,
        [id]: {
          ...(prev ?? { id, title, done: 0, total: 0 }),
          finished: true,
          running: false,
          paused: false,
          error: String(e)
        }
      };
    }
  }

  async cancel(id: number) {
    const prev = this.jobs[id];
    if (!prev || !prev.running) return;
    await cancelDownload(id).catch(() => {});
  }

  remove(id: number) {
    const { [id]: _, ...rest } = this.jobs;
    this.jobs = rest;
  }

  clearFinished() {
    const rest: Record<number, DownloadState> = {};
    for (const job of this.list) {
      if (job.running || job.paused) rest[job.id] = job;
    }
    this.jobs = rest;
  }
}

export const downloadStore = new DownloadStore();
