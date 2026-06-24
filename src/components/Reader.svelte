<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fileSrc } from '$lib/api';
  import { readerStore } from '$lib/reader-store.svelte';
  import { libraryStore } from '$lib/library-store.svelte';
  import { settingsStore } from '$lib/settings-store.svelte';
  import Icon from './Icon.svelte';

  const gallery = $derived(readerStore.gallery!);
  const mode = $derived(settingsStore.readingMode);
  const rtl = $derived(settingsStore.readingDirection === 'rtl');
  const lastIndex = $derived(gallery.files.length - 1);
  const step = $derived(mode === 'spread' ? 2 : 1);

  let scroller = $state<HTMLDivElement | null>(null);
  let pageEls: HTMLElement[] = $state([]);
  let current = $state(0);
  let initialPage = $state(0);
  let zoomed = $state<number | null>(null);
  let jumpInput = $state('1');
  let scrolled = false;
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  // Snapshot identity up front: `gallery` is derived from readerStore and becomes
  // null on close, so the unmount flush below must not read it.
  const galleryId = readerStore.gallery!.id;
  const pageTotal = readerStore.gallery!.pageCount;

  const leftIdx = $derived(mode === 'spread' ? (rtl ? current + 1 : current) : current);
  const rightIdx = $derived(mode === 'spread' ? (rtl ? current : current + 1) : current);

  onMount(() => {
    const g = readerStore.gallery!;
    const saved = libraryStore.progressOf(g.id);
    let start = readerStore.startPage || (saved && saved.page > 1 ? saved.page - 1 : 0);
    start = Math.max(0, Math.min(g.files.length - 1, start));
    initialPage = start;
    current = mode === 'spread' ? start - (start % 2) : start;
    libraryStore.record(g);
  });

  function clampBase(i: number): number {
    let v = Math.max(0, Math.min(lastIndex, i));
    if (mode === 'spread') v -= v % 2;
    return v;
  }

  function go(deltaReading: number) {
    current = clampBase(current + deltaReading * step);
  }

  function close() {
    readerStore.close();
  }

  function openZoom(i: number) {
    zoomed = i;
    current = i;
  }

  function closeZoom() {
    if (zoomed !== null) {
      const el = pageEls[zoomed];
      if (el && scroller) scroller.scrollTo({ top: el.offsetTop });
    }
    zoomed = null;
  }

  function zoomStep(delta: number) {
    if (zoomed === null) return;
    const next = Math.max(0, Math.min(lastIndex, zoomed + delta));
    zoomed = next;
    current = next;
  }

  function jumpTo(e: Event) {
    e.preventDefault();
    const n = Number(jumpInput);
    if (!Number.isFinite(n)) return;
    const idx = Math.max(0, Math.min(lastIndex, Math.floor(n) - 1));
    if (mode === 'continuous') {
      zoomed = idx;
      current = idx;
    } else {
      current = clampBase(idx);
    }
  }

  function setPageFromRange(e: Event) {
    const idx = Math.max(0, Math.min(lastIndex, Number((e.currentTarget as HTMLInputElement).value) - 1));
    if (mode === 'continuous') {
      current = idx;
      const el = pageEls[idx];
      if (el && scroller) scroller.scrollTo({ top: el.offsetTop });
    } else {
      current = clampBase(idx);
    }
  }

  $effect(() => {
    if (zoomed !== null) jumpInput = String(zoomed + 1);
    else if (mode !== 'continuous') jumpInput = String(current + 1);
  });

  $effect(() => {
    if (mode === 'spread' && current % 2 !== 0) current -= 1;
  });

  function onkeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (tag === 'INPUT' && e.key !== 'Escape') return;

    if (mode === 'continuous') {
      if (zoomed !== null) {
        if (e.key === 'ArrowLeft') {
          e.preventDefault();
          zoomStep(-1);
        } else if (e.key === 'ArrowRight' || e.key === ' ') {
          e.preventDefault();
          zoomStep(1);
        } else if (e.key === 'Escape') {
          closeZoom();
        }
        return;
      }
      if (e.key === 'Escape') close();
      else if (e.key === 'Home') scroller?.scrollTo({ top: 0 });
      else if (e.key === 'End') scroller?.scrollTo({ top: scroller?.scrollHeight ?? 0 });
      return;
    }

    if (e.key === 'Escape') {
      close();
      return;
    }
    const nextKey = rtl ? 'ArrowLeft' : 'ArrowRight';
    const prevKey = rtl ? 'ArrowRight' : 'ArrowLeft';
    if (e.key === nextKey || e.key === ' ') {
      e.preventDefault();
      go(1);
    } else if (e.key === prevKey) {
      e.preventDefault();
      go(-1);
    } else if (e.key === 'Home') {
      current = 0;
    } else if (e.key === 'End') {
      current = clampBase(lastIndex);
    }
  }

  $effect(() => {
    if (mode !== 'continuous') return;
    const el = pageEls[initialPage];
    if (el && scroller && !scrolled) {
      scroller.scrollTo({ top: el.offsetTop });
      scrolled = true;
    }
  });

  $effect(() => {
    const page = current + 1;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => libraryStore.saveProgress(galleryId, page, pageTotal), 600);
  });

  // Closing the reader within the debounce window must not drop the last page;
  // cancel the pending timer and persist the final position immediately.
  onDestroy(() => {
    clearTimeout(saveTimer);
    libraryStore.saveProgress(galleryId, current + 1, pageTotal);
  });

  $effect(() => {
    if (mode === 'continuous') return;
    for (let k = 1; k <= step + 2; k++) {
      const i = current + k;
      if (i < 0 || i > lastIndex) continue;
      const img = new Image();
      img.src = fileSrc(gallery.files[i], false);
    }
  });

  $effect(() => {
    if (mode !== 'continuous' || !scroller || !pageEls.length) return;
    const ratios = new Map<number, number>();
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          const idx = Number((e.target as HTMLElement).dataset.page);
          ratios.set(idx, e.intersectionRatio);
        }
        if (zoomed !== null) return;
        let best = current;
        let bestRatio = -1;
        for (const [idx, r] of ratios) {
          if (r > bestRatio) {
            bestRatio = r;
            best = idx;
          }
        }
        current = best;
      },
      { root: scroller, threshold: [0.1, 0.5, 0.9] }
    );
    for (const el of pageEls) if (el) io.observe(el);
    return () => io.disconnect();
  });
</script>

<svelte:window {onkeydown} />

<div class="fixed inset-0 z-50 flex flex-col bg-room-floor">
  <div
    class="flex h-11 shrink-0 items-center gap-3 border-b border-room-line bg-room-bg/90 px-4 backdrop-blur"
  >
    <button
      class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel hover:text-room-text"
      onclick={close}
      aria-label="Close"
    >
      <Icon name="close" class="size-4" />
    </button>
    <span class="truncate text-[12.5px] text-room-text">{gallery.title}</span>
    <span class="ml-auto font-mono text-[11px] tabular-nums text-room-text-mid">
      {current + 1} / {gallery.pageCount}
    </span>
    <input
      class="h-7 w-28 sm:w-44"
      type="range"
      min="1"
      max={gallery.pageCount}
      value={current + 1}
      aria-label="Reader page position"
      oninput={setPageFromRange}
    />
  </div>

  {#if mode === 'continuous'}
    <div
      bind:this={scroller}
      class="flex min-h-0 flex-1 flex-col items-center gap-1 overflow-auto py-1"
    >
      {#each gallery.files as file, i (i)}
        <button
          bind:this={pageEls[i]}
          data-page={i}
          class="block w-full max-w-[1000px] cursor-zoom-in bg-room-bg"
          style={`aspect-ratio: ${file.width || 7} / ${file.height || 10}`}
          onclick={() => openZoom(i)}
          aria-label={`Zoom page ${i + 1}`}
        >
          <img
            class="h-full w-full object-contain"
            src={fileSrc(file, false)}
            alt={`page ${i + 1}`}
            loading="lazy"
            draggable="false"
          />
        </button>
      {/each}
    </div>
  {:else}
    <div class="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden">
      {#if mode === 'spread'}
        <div class="flex h-full w-full items-center justify-center gap-0.5">
          {#if leftIdx <= lastIndex}
            <img
              class="h-full max-w-[50%] object-contain"
              src={fileSrc(gallery.files[leftIdx], false)}
              alt={`page ${leftIdx + 1}`}
              draggable="false"
            />
          {/if}
          {#if rightIdx <= lastIndex && rightIdx !== leftIdx}
            <img
             class="h-full max-w-[50%] object-contain"
              src={fileSrc(gallery.files[rightIdx], false)}
              alt={`page ${rightIdx + 1}`}
              draggable="false"
            />
          {/if}
        </div>
      {:else}
        <img
          class="max-h-full max-w-full object-contain"
          src={fileSrc(gallery.files[current], false)}
          alt={`page ${current + 1}`}
          draggable="false"
        />
      {/if}

      <button
        class="absolute left-0 top-0 h-full w-1/3 cursor-w-resize"
        onclick={() => go(rtl ? 1 : -1)}
        aria-label="Left"
        tabindex="-1"
      ></button>
      <button
        class="absolute right-0 top-0 h-full w-1/3 cursor-e-resize"
        onclick={() => go(rtl ? -1 : 1)}
        aria-label="Right"
        tabindex="-1"
      ></button>

      <button
        class="absolute left-3 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-black/40 text-white/80 transition hover:bg-black/60 hover:text-white disabled:pointer-events-none disabled:opacity-25"
        onclick={() => go(rtl ? 1 : -1)}
        disabled={current === 0}
        aria-label={rtl ? 'Next' : 'Previous'}
      >
        <Icon name="chevron-left" class="size-6" />
      </button>
      <button
        class="absolute right-3 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-black/40 text-white/80 transition hover:bg-black/60 hover:text-white disabled:pointer-events-none disabled:opacity-25"
        onclick={() => go(rtl ? -1 : 1)}
        disabled={current >= clampBase(lastIndex)}
        aria-label={rtl ? 'Previous' : 'Next'}
      >
        <Icon name="chevron-right" class="size-6" />
      </button>

      <form
        class="absolute left-1/2 bottom-3 flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-black/50 px-3 py-1 font-mono text-[11px] tabular-nums text-white/90"
        onsubmit={jumpTo}
      >
        <input
          class="w-10 rounded bg-white/10 text-center text-white focus:bg-white/20 focus:outline-none"
          bind:value={jumpInput}
          inputmode="numeric"
          aria-label="Jump to page"
        />
        <span class="text-white/60">/</span>
        <span>{gallery.pageCount}</span>
      </form>
    </div>
  {/if}
</div>

{#if zoomed !== null}
  <div class="fixed inset-0 z-[60] bg-black">
    <button
      class="absolute inset-0 cursor-zoom-out"
      onclick={closeZoom}
      aria-label="Close zoom"
      tabindex="-1"
    ></button>

    <img
      class="pointer-events-none absolute inset-0 m-auto max-h-full max-w-full object-contain"
      src={fileSrc(gallery.files[zoomed], false)}
      alt={`page ${zoomed + 1}`}
      draggable="false"
    />

    <button
      class="absolute left-3 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-black/40 text-white/80 transition hover:bg-black/60 hover:text-white disabled:pointer-events-none disabled:opacity-25"
      onclick={() => zoomStep(-1)}
      disabled={zoomed === 0}
      aria-label="Previous page"
    >
      <Icon name="chevron-left" class="size-6" />
    </button>
    <button
      class="absolute right-3 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-black/40 text-white/80 transition hover:bg-black/60 hover:text-white disabled:pointer-events-none disabled:opacity-25"
      onclick={() => zoomStep(1)}
      disabled={zoomed === lastIndex}
      aria-label="Next page"
    >
      <Icon name="chevron-right" class="size-6" />
    </button>

    <form
      class="absolute left-1/2 top-3 flex -translate-x-1/2 items-center gap-1.5 rounded-full bg-black/50 px-3 py-1 font-mono text-[11px] tabular-nums text-white/90"
      onsubmit={jumpTo}
    >
      <input
        class="w-10 rounded bg-white/10 text-center text-white focus:bg-white/20 focus:outline-none"
        bind:value={jumpInput}
        inputmode="numeric"
        aria-label="Jump to page"
      />
      <span class="text-white/60">/</span>
      <span>{gallery.pageCount}</span>
    </form>
    <button
      class="absolute right-3 top-3 grid size-9 place-items-center rounded-full bg-black/40 text-white/80 transition hover:bg-black/60 hover:text-white"
      onclick={closeZoom}
      aria-label="Close"
    >
      <Icon name="close" class="size-5" />
    </button>
  </div>
{/if}
