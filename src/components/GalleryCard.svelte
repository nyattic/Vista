<script lang="ts">
  import { imageSrc } from '$lib/api';
  import { parseTag, primaryArtist } from '$lib/format';
  import { libraryStore } from '$lib/library-store.svelte';
  import type { Gallery } from '$lib/types';
  import Icon from './Icon.svelte';
  import Spinner from './Spinner.svelte';

  let {
    gallery,
    selected = false,
    index,
    tabindex = -1,
    onselect,
    onopen,
    onkeynav
  }: {
    gallery: Gallery;
    selected?: boolean;
    index: number;
    tabindex?: number;
    onselect: (id: number) => void;
    onopen: (g: Gallery) => void;
    onkeynav: (e: KeyboardEvent, index: number, g: Gallery) => void;
  } = $props();

  let loaded = $state(false);
  let failed = $state(false);

  const cover = $derived(gallery.files[0]?.hash ?? null);
  const artist = $derived(primaryArtist(gallery));
  const chips = $derived(gallery.tags.slice(0, 3).map(parseTag));
  const fav = $derived(libraryStore.isFavorite(gallery.id));
  const pct = $derived(libraryStore.percent(gallery.id));
</script>

<button
  type="button"
  role="gridcell"
  {tabindex}
  data-card={gallery.id}
  class="group flex flex-col overflow-hidden rounded-[4px] border text-left transition-colors duration-150 {selected
    ? 'border-room-accent bg-room-panel-hi'
    : 'border-room-line bg-room-panel hover:border-room-line-strong'}"
  onclick={() => onselect(gallery.id)}
  ondblclick={() => onopen(gallery)}
  onkeydown={(e) => onkeynav(e, index, gallery)}
>
  <div class="relative aspect-[3/4] overflow-hidden bg-room-floor">
    {#if cover && !failed}
      <img
        class="absolute inset-0 h-full w-full object-cover transition-opacity duration-300 {loaded
          ? 'opacity-100'
          : 'opacity-0'}"
        src={imageSrc(cover, true)}
        alt={gallery.title}
        draggable="false"
        loading="lazy"
        onload={() => (loaded = true)}
        onerror={() => (failed = true)}
      />
    {/if}
    {#if !loaded && !failed}
      <span class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
        <Spinner class="size-4" />
      </span>
    {:else if failed}
      <span
        class="absolute inset-0 grid place-items-center font-mono text-[9.5px] uppercase tracking-[0.2em] text-room-text-low"
      >
        no preview
      </span>
    {/if}

    <span
      class="absolute left-1.5 top-1.5 rounded-[3px] bg-black/65 px-1.5 py-0.5 font-mono text-[9px] uppercase tracking-wider text-white backdrop-blur-sm"
    >
      {gallery.type || 'all'}
    </span>
    <span
      class="absolute bottom-1.5 right-1.5 rounded-[3px] bg-black/65 px-1.5 py-0.5 font-mono text-[9px] tabular-nums text-white backdrop-blur-sm"
    >
      {gallery.pageCount}p
    </span>
    {#if fav}
      <span
        class="absolute right-1.5 top-1.5 text-room-fav drop-shadow-[0_1px_2px_rgba(0,0,0,0.8)]"
      >
        <Icon name="heart" class="size-3.5" filled />
      </span>
    {/if}
    {#if pct > 0}
      <span class="absolute inset-x-0 bottom-0 h-[3px] bg-black/40">
        <span class="block h-full bg-room-accent" style="width: {pct}%"></span>
      </span>
    {/if}
  </div>

  <div class="flex min-h-0 flex-col gap-1 px-2.5 py-2">
    <div class="line-clamp-2 text-[12px] leading-snug text-room-text">{gallery.title}</div>
    {#if artist}
      <div class="truncate text-[11px] text-room-text-mid">{artist}</div>
    {/if}
    <div class="flex flex-wrap items-center gap-1 overflow-hidden">
      {#if gallery.language}
        <span class="font-mono text-[9.5px] uppercase tracking-wide text-room-accent"
          >{gallery.language}</span
        >
      {/if}
      {#each chips as chip, i (i)}
        <span
          class="truncate rounded-[3px] px-1 text-[9.5px] {chip.kind === 'male'
            ? 'text-room-male'
            : chip.kind === 'female'
              ? 'text-room-female'
              : 'text-room-text-low'}"
        >
          {chip.label}
        </span>
      {/each}
    </div>
  </div>
</button>
