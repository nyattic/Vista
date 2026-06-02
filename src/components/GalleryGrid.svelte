<script lang="ts">
  import { tick } from 'svelte';
  import { galleryStore } from '$lib/gallery-store.svelte';
  import { readerStore } from '$lib/reader-store.svelte';
  import { settingsStore } from '$lib/settings-store.svelte';
  import type { Gallery } from '$lib/types';
  import GalleryCard from './GalleryCard.svelte';
  import Spinner from './Spinner.svelte';

  let container = $state<HTMLDivElement | null>(null);

  function columns(): number {
    if (!container) return 1;
    const width = container.clientWidth - 24;
    return Math.max(1, Math.floor(width / (settingsStore.tileMin + 10)));
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
      if (edgeKey === 'next') galleryStore.next();
      else galleryStore.prev();
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
          no results
        </div>
        <p class="mt-2 text-[12px] text-room-text-mid">No galleries found.</p>
      </div>
    </div>
  {:else}
    <div
      class="grid gap-2.5"
      style="grid-template-columns: repeat(auto-fill, minmax({settingsStore.tileMin}px, 1fr));"
      role="grid"
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
