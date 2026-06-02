<script lang="ts">
  import { onMount } from 'svelte';
  import { imageSrc } from '$lib/api';
  import { readerStore } from '$lib/reader-store.svelte';
  import { libraryStore } from '$lib/library-store.svelte';
  import Icon from './Icon.svelte';

  const gallery = $derived(readerStore.gallery!);

  let scroller = $state<HTMLDivElement | null>(null);
  let pageEls: HTMLElement[] = $state([]);
  let current = $state(0);
  let initialPage = $state(0);
  let zoomed = $state<number | null>(null);
  let jumpInput = $state('1');
  let scrolled = false;
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  const lastIndex = $derived(gallery.files.length - 1);

  onMount(() => {
    const g = readerStore.gallery!;
    const saved = libraryStore.progressOf(g.id);
    initialPage = readerStore.startPage || (saved && saved.page > 1 ? saved.page - 1 : 0);
    current = initialPage;
    libraryStore.record(g);
  });

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

  function step(delta: number) {
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
    zoomed = idx;
    current = idx;
  }

  $effect(() => {
    if (zoomed !== null) jumpInput = String(zoomed + 1);
  });

  function onkeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (tag === 'INPUT' && e.key !== 'Escape') return;
    if (zoomed !== null) {
      if (e.key === 'ArrowLeft') {
        e.preventDefault();
        step(-1);
      } else if (e.key === 'ArrowRight' || e.key === ' ') {
        e.preventDefault();
        step(1);
      } else if (e.key === 'Escape') {
        closeZoom();
      }
      return;
    }
    if (e.key === 'Escape') close();
    else if (e.key === 'Home') scroller?.scrollTo({ top: 0 });
    else if (e.key === 'End') scroller?.scrollTo({ top: scroller?.scrollHeight ?? 0 });
  }

  $effect(() => {
    const el = pageEls[initialPage];
    if (el && scroller && !scrolled) {
      scroller.scrollTo({ top: el.offsetTop });
      scrolled = true;
    }
  });

  $effect(() => {
    const page = current + 1;
    const total = gallery.pageCount;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => libraryStore.saveProgress(gallery.id, page, total), 600);
    return () => clearTimeout(saveTimer);
  });

  $effect(() => {
    if (!scroller || !pageEls.length) return;
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
  </div>

  <div bind:this={scroller} class="flex min-h-0 flex-1 flex-col items-center gap-1 overflow-auto py-1">
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
          src={imageSrc(file.hash, false)}
          alt={`page ${i + 1}`}
          loading="lazy"
          draggable="false"
        />
      </button>
    {/each}
  </div>
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
      src={imageSrc(gallery.files[zoomed].hash, false)}
      alt={`page ${zoomed + 1}`}
      draggable="false"
    />

    <button
      class="absolute left-3 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-black/40 text-white/80 transition hover:bg-black/60 hover:text-white disabled:pointer-events-none disabled:opacity-25"
      onclick={() => step(-1)}
      disabled={zoomed === 0}
      aria-label="Previous page"
    >
      <Icon name="chevron-left" class="size-6" />
    </button>
    <button
      class="absolute right-3 top-1/2 grid size-11 -translate-y-1/2 place-items-center rounded-full bg-black/40 text-white/80 transition hover:bg-black/60 hover:text-white disabled:pointer-events-none disabled:opacity-25"
      onclick={() => step(1)}
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
