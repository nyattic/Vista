import type { Gallery } from './types';

class ReaderStore {
  gallery = $state<Gallery | null>(null);
  startPage = $state(0);

  get isOpen(): boolean {
    return this.gallery !== null;
  }

  open(gallery: Gallery, page = 0) {
    this.startPage = page;
    this.gallery = gallery;
  }

  close() {
    this.gallery = null;
  }
}

export const readerStore = new ReaderStore();
