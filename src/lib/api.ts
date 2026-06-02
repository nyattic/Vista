import { invoke } from '@tauri-apps/api/core';
import type { Gallery, GalleryPage, GalleryType, Language, SortOrder } from './types';

export function fetchGalleries(
  page: number,
  gtype: GalleryType,
  sort: SortOrder,
  language: Language
): Promise<GalleryPage> {
  return invoke('fetch_galleries', { page, gtype, sort, language });
}

export function fetchGallery(id: number): Promise<Gallery> {
  return invoke('fetch_gallery', { id });
}

export function searchGalleries(
  query: string,
  page: number,
  language: Language
): Promise<GalleryPage> {
  return invoke('search_galleries', { query, page, language });
}

export interface Suggestion {
  label: string;
  value: string;
  count: number;
  namespace: string;
}

export function tagSuggestions(query: string): Promise<Suggestion[]> {
  return invoke('tag_suggestions', { query });
}

export function downloadGallery(id: number, dir: string): Promise<string> {
  return invoke('download_gallery', { id, dir });
}

export function cancelDownload(id: number): Promise<void> {
  return invoke('cancel_download', { id });
}

export function defaultDownloadDir(): Promise<string | null> {
  return invoke('default_download_dir');
}

export function clearImageCache(): Promise<void> {
  return invoke('clear_image_cache');
}

export function imageCacheSize(): Promise<number> {
  return invoke('image_cache_size');
}

export function setCacheLimit(bytes: number): Promise<void> {
  return invoke('set_cache_limit', { bytes });
}

export interface ProgressRow {
  id: number;
  page: number;
  total: number;
}

export function toggleFavorite(gallery: Gallery): Promise<boolean> {
  return invoke('toggle_favorite', { gallery });
}

export function removeFavorite(id: number): Promise<void> {
  return invoke('remove_favorite', { id });
}

export function favoriteIds(): Promise<number[]> {
  return invoke('favorite_ids');
}

export function listFavorites(): Promise<Gallery[]> {
  return invoke('list_favorites');
}

export function recordView(gallery: Gallery): Promise<void> {
  return invoke('record_view', { gallery });
}

export function listHistory(): Promise<Gallery[]> {
  return invoke('list_history');
}

export function removeHistory(id: number): Promise<void> {
  return invoke('remove_history', { id });
}

export function clearHistory(): Promise<void> {
  return invoke('clear_history');
}

export function setProgress(id: number, page: number, total: number): Promise<void> {
  return invoke('set_progress', { id, page, total });
}

export function allProgress(): Promise<ProgressRow[]> {
  return invoke('all_progress');
}

const isWindows =
  typeof navigator !== 'undefined' && navigator.userAgent.includes('Windows');

export function imageSrc(hash: string, thumbnail = false): string {
  const path = thumbnail ? `${hash}?thumb=1` : hash;
  return isWindows ? `http://vimg.localhost/${path}` : `vimg://localhost/${path}`;
}
