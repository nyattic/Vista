export interface GalleryFile {
  name: string;
  hash: string;
  width: number;
  height: number;
  haswebp: number;
  hasavif: number;
  hasavifsmalltn: number | null;
  localPath?: string;
}

export interface Gallery {
  id: number;
  title: string;
  type: string;
  language: string | null;
  artists: string[];
  groups: string[];
  series: string[];
  characters: string[];
  tags: string[];
  date: string;
  files: GalleryFile[];
  pageCount: number;
  downloadFolder?: string;
  downloadFailedPages?: number[];
}

export interface DownloadRecord {
  gallery: Gallery;
  folder: string;
  failedPages: number[];
}

export interface DownloadResult {
  id: number;
  gallery: Gallery;
  folder: string;
  done: number;
  total: number;
  failed: number;
  failedPages: number[];
  skipped: number;
}

export interface GalleryPage {
  items: Gallery[];
  total: number;
  totalPages: number;
  page: number;
}

export type GalleryType = '' | 'doujinshi' | 'manga' | 'artistcg' | 'gamecg' | 'anime';

export const GALLERY_TYPES: { value: GalleryType; label: string }[] = [
  { value: '', label: 'All' },
  { value: 'doujinshi', label: 'Doujinshi' },
  { value: 'manga', label: 'Manga' },
  { value: 'artistcg', label: 'Artist CG' },
  { value: 'gamecg', label: 'Game CG' },
  { value: 'anime', label: 'Anime' }
];

export type SortOrder = 'latest' | 'today' | 'week' | 'month' | 'year';

export const SORT_ORDERS: { value: SortOrder; label: string }[] = [
  { value: 'latest', label: 'Latest' },
  { value: 'today', label: 'Day' },
  { value: 'week', label: 'Week' },
  { value: 'month', label: 'Month' },
  { value: 'year', label: 'Year' }
];

export type Language = '' | 'korean' | 'english' | 'japanese' | 'chinese';

export const LANGUAGES: { value: Language; label: string }[] = [
  { value: '', label: 'All' },
  { value: 'korean', label: 'Korean' },
  { value: 'english', label: 'English' },
  { value: 'japanese', label: 'Japanese' },
  { value: 'chinese', label: 'Chinese' }
];

export const DEFAULT_PAGE_SIZE = 20;
export const MIN_PAGE_SIZE = 12;
export const MAX_PAGE_SIZE = 80;
