import { listen } from '@tauri-apps/api/event';
import { downloadGallery, defaultDownloadDir } from './api';
import { settingsStore } from './settings-store.svelte';

export interface DownloadState {
  done: number;
  total: number;
  finished: boolean;
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
      this.jobs = {
        ...this.jobs,
        [id]: { done, total, finished: false, folder: prev?.folder }
      };
    });

    listen<DoneEvent>('download-done', (e) => {
      const { id, folder } = e.payload;
      const prev = this.jobs[id] ?? { done: 0, total: 0 };
      this.jobs = { ...this.jobs, [id]: { ...prev, finished: true, folder } };
    });
  }

  get(id: number): DownloadState | undefined {
    return this.jobs[id];
  }

  async start(id: number) {
    if (this.jobs[id] && !this.jobs[id].finished && !this.jobs[id].error) return;

    let dir = settingsStore.downloadDir;
    if (!dir) {
      dir = (await defaultDownloadDir()) ?? '';
      if (dir) settingsStore.setDownloadDir(dir);
    }
    if (!dir) {
      this.jobs = { ...this.jobs, [id]: { done: 0, total: 0, finished: true, error: 'no download folder' } };
      return;
    }

    this.jobs = { ...this.jobs, [id]: { done: 0, total: 0, finished: false } };
    try {
      await downloadGallery(id, dir);
    } catch (e) {
      const prev = this.jobs[id] ?? { done: 0, total: 0 };
      this.jobs = { ...this.jobs, [id]: { ...prev, finished: true, error: String(e) } };
    }
  }
}

export const downloadStore = new DownloadStore();
