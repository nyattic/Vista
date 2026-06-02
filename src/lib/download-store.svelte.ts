import { listen, type UnlistenFn } from '@tauri-apps/api/event';
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

class DownloadStore {
  jobs = $state<Record<number, DownloadState>>({});
  private started = false;
  private unlisteners: UnlistenFn[] = [];
  // Ids the user asked to cancel. The awaited `downloadGallery` promise is the
  // single source of truth for a job's terminal state; this set tells us whether
  // that resolution should be read as "paused" (user cancelled) or "finished".
  private cancelRequested = new Set<number>();

  async init() {
    if (this.started) return;
    this.started = true;
    try {
      // Only progress is event-driven. Completion/cancellation is decided by the
      // `start()` promise so the two can never race to write the terminal state.
      const off = await listen<ProgressEvent>('download-progress', (e) => {
        const { id, done, total } = e.payload;
        const prev = this.jobs[id];
        // Ignore strays for unknown or already-finalized jobs.
        if (!prev || prev.finished) return;
        this.patch(id, { ...prev, done, total });
      });
      this.unlisteners.push(off);
    } catch (e) {
      // Registration failed — allow a later retry rather than silently wedging.
      this.started = false;
      console.error('failed to register download listeners', e);
    }
  }

  destroy() {
    for (const off of this.unlisteners) off();
    this.unlisteners = [];
    this.started = false;
  }

  private patch(id: number, next: DownloadState) {
    this.jobs = { ...this.jobs, [id]: next };
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
      this.patch(id, {
        ...(cur ?? { id, title, done: 0, total: 0 }),
        id,
        title,
        finished: true,
        running: false,
        paused: false,
        error: 'already downloaded'
      });
      return;
    }

    let dir = settingsStore.downloadDir;
    if (!dir) {
      dir = (await defaultDownloadDir()) ?? '';
      if (dir) settingsStore.setDownloadDir(dir);
    }
    if (!dir) {
      this.patch(id, {
        id,
        title,
        done: 0,
        total: 0,
        finished: true,
        running: false,
        paused: false,
        error: 'no download folder'
      });
      return;
    }

    // Clear any cancel intent left over from a previous (paused) run of this id.
    this.cancelRequested.delete(id);
    this.patch(id, {
      id,
      title,
      done: cur?.done ?? 0,
      total: cur?.total ?? 0,
      finished: false,
      running: true,
      paused: false,
      failedPages: retryPages.length ? retryPages : cur?.failedPages,
      error: undefined
    });
    try {
      const result = await downloadGallery(id, dir, retryPages.length ? retryPages : undefined);
      const prev = this.jobs[id] ?? { id, title, done: 0, total: result.total };
      // `delete` returns true iff a cancel was requested for this run.
      const cancelled = this.cancelRequested.delete(id);
      if (cancelled) {
        // The backend honored the cancel mid-flight: this is a pause, not a
        // completion, so the library is left un-marked and the job stays resumable.
        this.patch(id, {
          ...prev,
          id,
          title,
          finished: false,
          running: false,
          paused: true,
          folder: result.folder,
          failed: result.failed,
          failedPages: result.failedPages,
          skipped: result.skipped,
          total: result.total,
          done: result.done,
          error: undefined
        });
      } else {
        libraryStore.markDownloaded(id);
        this.patch(id, {
          ...prev,
          id,
          title,
          finished: true,
          running: false,
          paused: false,
          folder: result.folder,
          failed: result.failed,
          failedPages: result.failedPages,
          skipped: result.skipped,
          total: result.total,
          done: result.done,
          error: undefined
        });
      }
    } catch (e) {
      this.cancelRequested.delete(id);
      const prev = this.jobs[id] ?? { id, title, done: 0, total: 0 };
      this.patch(id, {
        ...prev,
        id,
        title,
        finished: true,
        running: false,
        paused: false,
        error: String(e)
      });
    }
  }

  async cancel(id: number) {
    const prev = this.jobs[id];
    if (!prev || !prev.running) return;
    // Record intent before signaling the backend; the resolving `start()` promise
    // reads this flag to settle the job as paused rather than finished/errored.
    this.cancelRequested.add(id);
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
