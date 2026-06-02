<script lang="ts">
  import { galleryStore } from '$lib/gallery-store.svelte';
  import Icon from './Icon.svelte';

  let pageInput = $state('1');

  $effect(() => {
    pageInput = String(galleryStore.page);
  });

  function jump(e: Event) {
    e.preventDefault();
    const n = Number(pageInput);
    if (Number.isFinite(n)) galleryStore.goToPage(n);
  }

  const atStart = $derived(galleryStore.page <= 1);
  const atEnd = $derived(galleryStore.page >= galleryStore.totalPages);
</script>

<footer
  class="flex h-11 shrink-0 items-center justify-center gap-1.5 border-t border-room-line bg-room-panel/40 px-4"
>
  <button
    class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text disabled:pointer-events-none disabled:opacity-30"
    disabled={atStart}
    onclick={() => galleryStore.goToPage(1)}
    aria-label="First page"
    title="First page"
  >
    <span class="font-mono text-[12px] leading-none">«</span>
  </button>
  <button
    class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text disabled:pointer-events-none disabled:opacity-30"
    disabled={atStart}
    onclick={() => galleryStore.prev()}
    aria-label="Previous"
  >
    <Icon name="chevron-left" class="size-4" />
  </button>

  <form class="flex items-center gap-1.5 px-1 font-mono text-[12px] tabular-nums" onsubmit={jump}>
    <input
      class="h-7 w-12 rounded-[3px] border border-room-line bg-room-panel text-center text-room-text focus:border-room-accent focus:outline-none"
      bind:value={pageInput}
      inputmode="numeric"
      aria-label="Page number"
    />
    <span class="text-room-text-low">/</span>
    <span class="min-w-10 text-room-text-mid">{galleryStore.totalPages.toLocaleString()}</span>
  </form>

  <button
    class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text disabled:pointer-events-none disabled:opacity-30"
    disabled={atEnd}
    onclick={() => galleryStore.next()}
    aria-label="Next"
  >
    <Icon name="chevron-right" class="size-4" />
  </button>
  <button
    class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text disabled:pointer-events-none disabled:opacity-30"
    disabled={atEnd}
    onclick={() => galleryStore.goToPage(galleryStore.totalPages)}
    aria-label="Last page"
    title="Last page"
  >
    <span class="font-mono text-[12px] leading-none">»</span>
  </button>

  {#if galleryStore.total > 0}
    <span class="ml-3 font-mono text-[10px] tabular-nums text-room-text-low">
      {galleryStore.total.toLocaleString()} items
    </span>
  {/if}
</footer>
