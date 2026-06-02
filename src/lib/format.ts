import type { Gallery } from './types';

export type TagKind = 'male' | 'female' | 'general';

export interface ParsedTag {
  kind: TagKind;
  label: string;
  raw: string;
}

export function parseTag(raw: string): ParsedTag {
  if (raw.startsWith('female:')) return { kind: 'female', label: raw.slice(7), raw };
  if (raw.startsWith('male:')) return { kind: 'male', label: raw.slice(5), raw };
  return { kind: 'general', label: raw, raw };
}

export function tagToQuery(raw: string): string {
  if (raw.startsWith('female:') || raw.startsWith('male:')) return raw.replace(/ /g, '_');
  return `tag:${raw.replace(/ /g, '_')}`;
}

export function compact(n: number): string {
  if (n < 1000) return String(n);
  if (n < 1_000_000) return `${(n / 1000).toFixed(n < 10_000 ? 1 : 0)}k`;
  return `${(n / 1_000_000).toFixed(1)}m`;
}

export function formatDate(date: string): string {
  if (!date) return '';
  return date.slice(0, 10);
}

export function primaryArtist(gallery: Gallery): string {
  if (gallery.artists.length) return gallery.artists.join(', ');
  if (gallery.groups.length) return gallery.groups.join(', ');
  return '';
}

export function galleryLabel(gallery: Gallery): string {
  return gallery.title || `#${gallery.id}`;
}
