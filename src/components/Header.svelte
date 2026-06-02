<script lang="ts">
  import { galleryStore, type View } from '$lib/gallery-store.svelte';
  import { searchHistoryStore } from '$lib/search-history-store.svelte';
  import { downloadStore } from '$lib/download-store.svelte';
  import { tagSuggestions, type Suggestion } from '$lib/api';
  import { GALLERY_TYPES } from '$lib/types';
  import Icon from './Icon.svelte';

  let {
    onopensettings,
    onopendownloads
  }: { onopensettings: () => void; onopendownloads: () => void } = $props();

  const activeDownloads = $derived(downloadStore.activeCount);

  const views: { value: View; icon: string; label: string }[] = [
    { value: 'browse', icon: 'grid', label: 'Browse' },
    { value: 'favorites', icon: 'heart', label: 'Favorites' },
    { value: 'history', icon: 'clock', label: 'History' },
    { value: 'downloads', icon: 'folder', label: 'Library' }
  ];

  let focused = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);
  let suggestions = $state<Suggestion[]>([]);
  let activeIndex = $state(-1);
  let token = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  const showSuggest = $derived(focused && galleryStore.query.trim().length > 0 && suggestions.length > 0);
  const showHistory = $derived(
    focused && galleryStore.query.trim().length === 0 && searchHistoryStore.items.length > 0
  );

  $effect(() => {
    const q = galleryStore.query;
    if (!focused || q.trim().length === 0) {
      suggestions = [];
      activeIndex = -1;
      return;
    }
    clearTimeout(timer);
    const t = ++token;
    timer = setTimeout(async () => {
      try {
        const s = await tagSuggestions(q);
        if (t === token) {
          suggestions = s;
          activeIndex = -1;
        }
      } catch {
        if (t === token) suggestions = [];
      }
    }, 180);
  });

  function namespaceClass(ns: string): string {
    if (ns === 'female') return 'text-room-female';
    if (ns === 'male') return 'text-room-male';
    return 'text-room-text-low';
  }

  function applySuggestion(s: Suggestion) {
    const parts = galleryStore.query.split(' ');
    parts[parts.length - 1] = s.value;
    galleryStore.query = parts.join(' ') + ' ';
    activeIndex = -1;
    suggestions = [];
    inputEl?.focus();
  }

  function submit(e: Event) {
    e.preventDefault();
    if (showSuggest && activeIndex >= 0) {
      applySuggestion(suggestions[activeIndex]);
      return;
    }
    const q = galleryStore.query.trim();
    if (q) searchHistoryStore.add(q);
    galleryStore.submitSearch();
    focused = false;
    inputEl?.blur();
  }

  function onkeydown(e: KeyboardEvent) {
    if (!showSuggest) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(suggestions.length - 1, activeIndex + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(-1, activeIndex - 1);
    } else if (e.key === 'Tab') {
      e.preventDefault();
      applySuggestion(suggestions[activeIndex >= 0 ? activeIndex : 0]);
    } else if (e.key === 'Escape') {
      suggestions = [];
      activeIndex = -1;
    }
  }

  function applyHistory(q: string) {
    searchHistoryStore.add(q);
    galleryStore.applyQuery(q);
    focused = false;
  }
</script>

<header class="flex h-12 items-center gap-3 border-b border-room-line px-4">
  <span class="shrink-0 select-none font-mono text-[15px] font-medium tracking-tight text-room-text">
    Vista
  </span>

  <div class="h-5 w-px shrink-0 bg-room-line"></div>

  <div class="flex shrink-0 items-center gap-0.5">
    {#each views as v (v.value)}
      <button
        class="grid size-7 place-items-center rounded-[3px] transition-colors duration-150 {galleryStore.view ===
        v.value
          ? 'bg-room-panel-hi text-room-accent'
          : 'text-room-text-mid hover:bg-room-panel hover:text-room-text'}"
        onclick={() => galleryStore.setView(v.value)}
        title={v.label}
        aria-label={v.label}
      >
        <Icon name={v.icon} class="size-4" filled={v.value === 'favorites' && galleryStore.view === 'favorites'} />
      </button>
    {/each}
  </div>

  {#if galleryStore.view === 'browse'}
    <div class="h-5 w-px shrink-0 bg-room-line"></div>
    <div class="flex shrink-0 items-center gap-0.5" role="tablist">
      {#each GALLERY_TYPES as t (t.value)}
        <button
          role="tab"
          aria-selected={!galleryStore.searching && galleryStore.gtype === t.value}
          class="rounded-[3px] px-2 py-1 text-[11.5px] tracking-tight transition-colors duration-150 {!galleryStore.searching &&
          galleryStore.gtype === t.value
            ? 'bg-room-panel-hi text-room-accent'
            : 'text-room-text-mid hover:bg-room-panel hover:text-room-text'}"
          onclick={() => galleryStore.setType(t.value)}
        >
          {t.label}
        </button>
      {/each}
    </div>
  {/if}

  <form class="relative ml-2 w-full max-w-[520px]" onsubmit={submit}>
    <div
      class="flex h-8 items-center rounded-[3px] border border-room-line bg-room-panel transition-colors focus-within:border-room-accent"
    >
      <span class="grid place-items-center pl-2.5 pr-1.5 text-room-text-low">
        <Icon name="search" class="size-3.5" />
      </span>
      <input
        bind:this={inputEl}
        class="min-w-0 flex-1 bg-transparent text-[12.5px] text-room-text placeholder:text-room-text-low focus:outline-none"
        placeholder="search  e.g.  artist:foo  language:korean  female:..."
        spellcheck="false"
        autocomplete="off"
        bind:value={galleryStore.query}
        onfocus={() => (focused = true)}
        onblur={() => setTimeout(() => (focused = false), 140)}
        onkeydown={onkeydown}
      />
      {#if galleryStore.searching}
        <button
          type="button"
          class="px-2 text-room-text-low hover:text-room-text"
          onclick={() => galleryStore.clearSearch()}
          aria-label="Clear search"
        >
          <Icon name="close" class="size-3.5" />
        </button>
      {/if}
      <button
        type="submit"
        class="h-full border-l border-room-line px-3 font-mono text-[10.5px] uppercase tracking-[0.16em] text-room-text-mid hover:text-room-text"
      >
        go
      </button>
    </div>

    {#if showSuggest}
      <div
        class="absolute left-0 right-0 top-9 z-30 max-h-80 overflow-auto rounded-[3px] border border-room-line bg-room-panel-hi shadow-[0_8px_24px_rgba(0,0,0,0.45)]"
      >
        {#each suggestions as s, i (s.namespace + s.label)}
          <button
            type="button"
            class="flex w-full items-center gap-2 px-3 py-1.5 text-left {i === activeIndex
              ? 'bg-room-bg text-room-accent'
              : 'hover:bg-room-bg/60'}"
            onmousedown={(e) => {
              e.preventDefault();
              applySuggestion(s);
            }}
            onmouseenter={() => (activeIndex = i)}
          >
            <span class="truncate text-[12.5px] text-room-text">{s.label}</span>
            <span class="font-mono text-[9.5px] uppercase tracking-wide {namespaceClass(s.namespace)}"
              >{s.namespace}</span
            >
            <span class="ml-auto font-mono text-[10px] tabular-nums text-room-text-low"
              >{s.count.toLocaleString()}</span
            >
          </button>
        {/each}
      </div>
    {:else if showHistory}
      <div
        class="absolute left-0 right-0 top-9 z-30 max-h-72 overflow-auto rounded-[3px] border border-room-line bg-room-panel-hi shadow-[0_8px_24px_rgba(0,0,0,0.45)]"
      >
        <div
          class="flex items-center justify-between px-3 py-1.5 font-mono text-[9.5px] uppercase tracking-[0.2em] text-room-text-low"
        >
          <span>recent</span>
          <button class="hover:text-room-text" onmousedown={() => searchHistoryStore.clear()}>clear</button>
        </div>
        {#each searchHistoryStore.items as item (item)}
          <div class="group flex items-center hover:bg-room-bg/60">
            <button
              class="flex-1 truncate px-3 py-1.5 text-left font-mono text-[11.5px] text-room-text-mid group-hover:text-room-text"
              onmousedown={() => applyHistory(item)}>{item}</button
            >
            <button
              class="px-2.5 text-room-text-low opacity-0 group-hover:opacity-100 hover:text-room-text"
              onmousedown={() => searchHistoryStore.remove(item)}
              aria-label="Remove"
            >
              <Icon name="close" class="size-3" />
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </form>

  <div class="ml-auto flex shrink-0 items-center gap-2">
    <button
      class="relative grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel hover:text-room-text"
      onclick={onopendownloads}
      aria-label="Downloads"
      title="Downloads"
    >
      <Icon name="download" class="size-4" />
      {#if activeDownloads > 0}
        <span
          class="absolute -right-0.5 -top-0.5 grid min-w-3.5 place-items-center rounded-full bg-room-accent px-1 font-mono text-[8px] leading-[14px] text-room-floor"
          >{activeDownloads}</span
        >
      {/if}
    </button>
    <button
      class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel hover:text-room-text"
      onclick={onopensettings}
      aria-label="Settings"
    >
      <Icon name="settings" class="size-4" />
    </button>
  </div>
</header>
