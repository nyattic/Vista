<script lang="ts">
  import { galleryStore } from '$lib/gallery-store.svelte';
  import { clearHistory } from '$lib/api';
  import { SORT_ORDERS } from '$lib/types';
  import Icon from './Icon.svelte';

  async function onClearHistory() {
    await clearHistory();
    galleryStore.load(1);
  }
</script>

<nav class="flex h-9 items-stretch border-b border-room-line bg-room-panel/40">
  <div
    class="flex items-center gap-2 px-4 font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
  >
    {galleryStore.searching ? 'search' : galleryStore.view}
    <span class="tracking-normal text-room-text-mid lowercase">
      {galleryStore.visible.length} shown
    </span>
  </div>

  {#if galleryStore.view === 'browse'}
    <div class="ml-auto flex items-stretch border-l border-room-line">
      <div
        class="flex items-center px-4 font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low"
      >
        sort
      </div>
      {#each SORT_ORDERS as o (o.value)}
        {@const active = !galleryStore.searching && galleryStore.sort === o.value}
        <button
          class="relative px-3.5 text-[12px] tracking-tight transition-colors duration-150 disabled:opacity-30 {active
            ? 'bg-room-panel-hi text-room-accent'
            : 'text-room-text-mid hover:bg-room-panel-hi hover:text-room-text'}"
          disabled={galleryStore.searching}
          onclick={() => galleryStore.setSort(o.value)}
        >
          {o.label}
          {#if active}
            <span class="absolute inset-x-3 bottom-0 h-px bg-room-accent"></span>
          {/if}
        </button>
      {/each}
    </div>
  {:else if galleryStore.view === 'history' && galleryStore.items.length > 0}
    <button
      class="ml-auto flex items-center gap-1.5 border-l border-room-line px-4 text-[12px] text-room-text-mid transition-colors hover:bg-room-panel-hi hover:text-room-text"
      onclick={onClearHistory}
    >
      <Icon name="trash" class="size-3.5" /> Clear history
    </button>
  {/if}
</nav>

