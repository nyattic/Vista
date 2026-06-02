import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'ready'
  | 'uptodate'
  | 'error';

class UpdateStore {
  status = $state<UpdateStatus>('idle');
  currentVersion = $state('');
  newVersion = $state('');
  notes = $state('');
  downloaded = $state(0);
  contentLength = $state(0);
  error = $state('');
  private update: Update | null = null;

  get percent(): number {
    if (this.contentLength <= 0) return 0;
    return Math.min(100, Math.round((this.downloaded / this.contentLength) * 100));
  }

  async init() {
    try {
      this.currentVersion = await getVersion();
    } catch {
      /* ignore */
    }
    if (import.meta.env.PROD) this.check(true);
  }

  async check(silent = false) {
    if (this.status === 'checking' || this.status === 'downloading') return;
    this.status = 'checking';
    this.error = '';
    try {
      const update = await check();
      if (update) {
        this.update = update;
        this.newVersion = update.version;
        this.notes = update.body ?? '';
        this.status = 'available';
      } else {
        this.status = silent ? 'idle' : 'uptodate';
      }
    } catch (e) {
      this.status = silent ? 'idle' : 'error';
      this.error = String(e);
    }
  }

  async install() {
    if (!this.update) return;
    this.status = 'downloading';
    this.downloaded = 0;
    this.contentLength = 0;
    try {
      await this.update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          this.contentLength = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          this.downloaded += event.data.chunkLength;
        } else if (event.event === 'Finished') {
          this.status = 'ready';
        }
      });
      this.status = 'ready';
      await relaunch();
    } catch (e) {
      this.status = 'error';
      this.error = String(e);
    }
  }

  dismiss() {
    if (this.status === 'available' || this.status === 'uptodate' || this.status === 'error') {
      this.status = 'idle';
    }
  }
}

export const updateStore = new UpdateStore();
