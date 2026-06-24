<script lang="ts">
  import { onMount } from 'svelte';
  import { tick, untrack } from 'svelte';
  import { galleryStore } from '$lib/gallery-store.svelte';
  import { readerStore } from '$lib/reader-store.svelte';
  import { settingsStore } from '$lib/settings-store.svelte';
  import { MAX_PAGE_SIZE, MIN_PAGE_SIZE } from '$lib/types';
  import type { Gallery } from '$lib/types';
  import GalleryCard from './GalleryCard.svelte';
  import Spinner from './Spinner.svelte';

  let container = $state<HTMLDivElement | null>(null);
  const GRID_PADDING = 24;
  const GRID_GAP = 10;
  const CARD_TEXT_HEIGHT = 86;

  function columns(): number {
    if (!container) return 1;
    const width = container.clientWidth - GRID_PADDING;
    return Math.max(1, Math.floor((width + GRID_GAP) / (settingsStore.effectiveTileMin + GRID_GAP)));
  }

  function pageSizeForContainer(): number {
    if (!container) return galleryStore.pageSize;
    const cols = columns();
    const tileWidth =
      (container.clientWidth - GRID_PADDING - GRID_GAP * Math.max(0, cols - 1)) / cols;
    const measuredCard = container.querySelector<HTMLButtonElement>('[data-card]');
    const cardHeight = measuredCard?.getBoundingClientRect().height || tileWidth * (4 / 3) + CARD_TEXT_HEIGHT;
    const availableHeight = container.clientHeight - GRID_PADDING;
    const rows = Math.max(1, Math.floor((availableHeight + GRID_GAP) / (cardHeight + GRID_GAP)));
    return Math.max(MIN_PAGE_SIZE, Math.min(MAX_PAGE_SIZE, cols * rows));
  }

  async function focusCard(id: number | null) {
    if (id === null) return;
    await tick();
    const el = container?.querySelector<HTMLButtonElement>(`[data-card="${id}"]`);
    el?.focus({ preventScroll: true });
    el?.scrollIntoView({ block: 'nearest' });
  }

  function move(delta: number, edgeKey?: 'prev' | 'next') {
    const before = galleryStore.selectedId;
    const after = galleryStore.moveSelection(delta);
    if (after === before && edgeKey) {
      if (edgeKey === 'next') galleryStore.next(true);
      else galleryStore.prev(true);
      return;
    }
    focusCard(after);
  }

  function onkeynav(e: KeyboardEvent, _index: number, gallery: Gallery) {
    const cols = columns();
    let handled = true;
    if (e.key === 'ArrowRight') move(1, 'next');
    else if (e.key === 'ArrowLeft') move(-1, 'prev');
    else if (e.key === 'ArrowDown') move(cols);
    else if (e.key === 'ArrowUp') move(-cols);
    else if (e.key === 'Enter') readerStore.open(gallery);
    else handled = false;
    if (handled) e.preventDefault();
  }

  $effect(() => {
    galleryStore.page;
    if (container) container.scrollTo({ top: 0 });
  });

  // After a keyboard-initiated page flip the store bumps focusRequest; move DOM
  // focus onto the freshly selected card. selectedId is read untracked so this
  // only fires on an explicit request, not on every selection change.
  $effect(() => {
    galleryStore.focusRequest;
    untrack(() => focusCard(galleryStore.selectedId));
  });

  onMount(() => {
    if (!container) return;
    let frame = 0;
    const update = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => galleryStore.setPageSize(pageSizeForContainer()));
    };
    const ro = new ResizeObserver(update);
    ro.observe(container);
    update();
    return () => {
      cancelAnimationFrame(frame);
      ro.disconnect();
    };
  });

  $effect(() => {
    settingsStore.tileMin;
    settingsStore.gridScalePct;
    galleryStore.setPageSize(pageSizeForContainer());
  });

  const emptyTitle = $derived(
    galleryStore.searching
      ? 'no matches'
      : galleryStore.view === 'favorites'
        ? 'no favorites'
        : galleryStore.view === 'history'
          ? 'no history'
          : galleryStore.view === 'downloads'
            ? 'no downloads'
            : 'no results'
  );

  const emptyText = $derived(
    galleryStore.searching
      ? 'Try another tag, artist, or language filter.'
      : galleryStore.view === 'favorites'
        ? 'Use the heart action on a gallery to save it here.'
        : galleryStore.view === 'history'
          ? 'Read a gallery and it will appear here.'
          : galleryStore.view === 'downloads'
            ? 'Download a gallery to build your local library.'
            : 'Try changing the type, language, or sort order.'
  );
</script>

<div bind:this={container} class="min-h-0 flex-1 overflow-auto p-3">
  {#if galleryStore.loading}
    <div class="grid h-full place-items-center">
      <div class="flex flex-col items-center gap-3 text-room-text-low">
        <Spinner class="size-5" />
        <span class="font-mono text-[10px] uppercase tracking-[0.25em]">loading</span>
      </div>
    </div>
  {:else if galleryStore.error}
    <div class="grid h-full place-items-center px-8 text-center">
      <div>
        <div class="font-mono text-[10px] uppercase tracking-[0.25em] text-room-text-low">error</div>
        <p class="mt-2 max-w-md text-[12px] text-room-text-mid">{galleryStore.error}</p>
        <button
          class="mt-4 rounded-[3px] border border-room-line px-3 py-1.5 text-[11px] text-room-text hover:border-room-line-strong"
          onclick={() => galleryStore.load(galleryStore.page)}>Retry</button
        >
      </div>
    </div>
  {:else if galleryStore.visible.length === 0}
    <div class="grid h-full place-items-center px-8 text-center">
      <div>
        <div class="font-mono text-[10px] uppercase tracking-[0.25em] text-room-text-low">
          {emptyTitle}
        </div>
        <p class="mt-2 max-w-xs text-[12px] text-room-text-mid">{emptyText}</p>
        {#if galleryStore.searching}
          <button
            class="mt-4 rounded-[3px] border border-room-line px-3 py-1.5 text-[11px] text-room-text hover:border-room-line-strong"
            onclick={() => galleryStore.clearSearch()}>Clear search</button
          >
        {/if}
      </div>
    </div>
  {:else}
    <div
      class="grid gap-2.5"
      style="grid-template-columns: repeat(auto-fill, minmax({settingsStore.effectiveTileMin}px, 1fr));"
      role="listbox"
      aria-label="Galleries"
    >
      {#each galleryStore.visible as gallery, index (gallery.id)}
        <GalleryCard
          {gallery}
          {index}
          selected={galleryStore.selectedId === gallery.id}
          tabindex={galleryStore.selectedId === gallery.id ? 0 : -1}
          onselect={(id) => galleryStore.select(id)}
          onopen={(g) => readerStore.open(g)}
          {onkeynav}
        />
      {/each}
    </div>
  {/if}
</div>
