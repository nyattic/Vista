<script lang="ts">
  import { fileSrc, openDownloadFolder, removeHistory, removeDownload } from '$lib/api';
  import { galleryStore } from '$lib/gallery-store.svelte';
  import { readerStore } from '$lib/reader-store.svelte';
  import { downloadStore } from '$lib/download-store.svelte';
  import { libraryStore } from '$lib/library-store.svelte';
  import { uiStore } from '$lib/ui-store.svelte';
  import { formatDate, parseTag, tagToQuery } from '$lib/format';
  import Icon from './Icon.svelte';

  let { class: cls = '' }: { class?: string } = $props();

  const gallery = $derived(galleryStore.selected);
  const dl = $derived(gallery ? downloadStore.get(gallery.id) : undefined);
  const fav = $derived(gallery ? libraryStore.isFavorite(gallery.id) : false);
  const downloaded = $derived(gallery ? libraryStore.isDownloaded(gallery.id) : false);
  const prog = $derived(gallery ? libraryStore.progressOf(gallery.id) : undefined);
  const failedPages = $derived(dl?.failedPages ?? gallery?.downloadFailedPages ?? []);

  function searchFor(prefix: string, value: string) {
    galleryStore.applyQuery(`${prefix}:${value.replace(/ /g, '_')}`);
  }

  async function toggleFav() {
    if (!gallery) return;
    const nowFav = await libraryStore.toggleFavorite(gallery);
    uiStore.toast(nowFav ? 'Added to favorites.' : 'Removed from favorites.', 'success');
    if (!nowFav && galleryStore.view === 'favorites') galleryStore.load(1);
  }

  async function dropHistory() {
    if (!gallery) return;
    const ok = await uiStore.confirm({
      title: 'Remove from history?',
      message: 'This gallery will disappear from the history view.',
      confirmLabel: 'Remove',
      tone: 'danger'
    });
    if (!ok) return;
    await removeHistory(gallery.id);
    if (galleryStore.view === 'history') galleryStore.load(1);
    uiStore.toast('Removed from history.', 'success');
  }

  async function dropDownload() {
    if (!gallery) return;
    const ok = await uiStore.confirm({
      title: 'Remove download record?',
      message: 'Files on disk will not be deleted, but this item will leave the downloads view.',
      confirmLabel: 'Remove',
      tone: 'danger'
    });
    if (!ok) return;
    await removeDownload(gallery.id).catch(() => {});
    libraryStore.markNotDownloaded(gallery.id);
    downloadStore.remove(gallery.id);
    if (galleryStore.view === 'downloads') galleryStore.load(1);
    uiStore.toast('Download record removed.', 'success');
  }

  async function openFolder() {
    if (!gallery) return;
    await openDownloadFolder(gallery.id).catch(() =>
      uiStore.toast('Could not open the download folder.', 'danger')
    );
  }

  function retryFailed() {
    if (!gallery || failedPages.length === 0) return;
    downloadStore.start(gallery.id, gallery.title, failedPages);
  }

  let shareCopied = $state(false);
  let shareTimer: ReturnType<typeof setTimeout> | undefined;

  async function share() {
    if (!gallery) return;
    const url = `https://hitomi.la/reader/${gallery.id}.html`;
    try {
      await navigator.clipboard.writeText(url);
      uiStore.toast('Link copied.', 'success');
      shareCopied = true;
      clearTimeout(shareTimer);
      shareTimer = setTimeout(() => (shareCopied = false), 1500);
    } catch {
      uiStore.toast('Could not copy the link.', 'danger');
    }
  }

  const metaRows = $derived(
    gallery
      ? [
          { label: 'artist', prefix: 'artist', values: gallery.artists },
          { label: 'group', prefix: 'group', values: gallery.groups },
          { label: 'series', prefix: 'series', values: gallery.series },
          { label: 'character', prefix: 'character', values: gallery.characters }
        ].filter((r) => r.values.length)
      : []
  );
</script>

<aside class="order-1 flex min-h-0 flex-col border-r border-room-line bg-room-panel/40 {cls}">
  {#if gallery}
    {#key gallery.id}
      <div class="min-h-0 overflow-auto">
        <button
          class="group relative block w-full overflow-hidden bg-room-floor"
          onclick={() => readerStore.open(gallery)}
          aria-label="Read"
        >
          {#if gallery.files.length}
            <img
              class="aspect-[3/4] w-full object-cover"
              src={fileSrc(gallery.files[0], true)}
              alt={gallery.title}
              draggable="false"
            />
          {/if}
          <span
            class="absolute inset-0 grid place-items-center bg-black/0 opacity-0 transition group-hover:bg-black/40 group-hover:opacity-100"
          >
            <span class="flex items-center gap-1.5 text-[12px] text-white">
              <Icon name="book" class="size-4" /> Read
            </span>
          </span>
        </button>

        <div class="flex flex-col gap-3 p-3.5">
          <div>
            <div class="font-mono text-[10.5px] tabular-nums text-room-text-low">#{gallery.id}</div>
            <h2 class="mt-0.5 text-[13.5px] leading-snug text-room-text">{gallery.title}</h2>
          </div>

          <div class="flex flex-wrap gap-1.5">
            <span class="rounded-[3px] bg-room-accent px-2 py-0.5 text-[10.5px] capitalize text-room-floor"
              >{gallery.type || 'all'}</span
            >
            {#if gallery.language}
              <span class="rounded-[3px] bg-room-panel-hi px-2 py-0.5 text-[10.5px] text-room-text"
                >{gallery.language}</span
              >
            {/if}
            <span class="rounded-[3px] bg-room-panel-hi px-2 py-0.5 text-[10.5px] tabular-nums text-room-text"
              >{gallery.pageCount} pages</span
            >
            {#if gallery.date}
              <span class="rounded-[3px] px-2 py-0.5 text-[10.5px] tabular-nums text-room-text-low"
                >{formatDate(gallery.date)}</span
              >
            {/if}
          </div>

          <div class="flex flex-col gap-1.5">
            <div class="flex gap-1.5">
              <button
                class="flex flex-1 items-center justify-center gap-2 rounded-[3px] bg-room-accent py-2 text-[12.5px] font-medium text-room-floor transition hover:brightness-110"
                onclick={() => readerStore.open(gallery)}
              >
                <Icon name="book" class="size-4" />
                {prog && prog.page > 1 ? `Continue · p${prog.page}` : 'Read'}
              </button>
              <button
                class="icon-tip flex items-center justify-center rounded-[3px] border px-3 transition {fav
                  ? 'border-room-fav/50 text-room-fav'
                  : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
                onclick={toggleFav}
                title={fav ? 'Remove favorite' : 'Add favorite'}
                aria-label={fav ? 'Remove favorite' : 'Add favorite'}
              >
                <Icon name="heart" class="size-4" filled={fav} />
              </button>
              <button
                class="icon-tip flex items-center justify-center rounded-[3px] border px-3 transition {shareCopied
                  ? 'border-room-accent/50 text-room-accent'
                  : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
                onclick={share}
                title="Copy link"
                aria-label={shareCopied ? 'Copied' : 'Copy link'}
              >
                <Icon name={shareCopied ? 'check' : 'share'} class="size-4" />
              </button>
              {#if dl?.running}
                <button
                  class="icon-tip flex items-center justify-center gap-1.5 rounded-[3px] border border-room-line px-3 text-room-text-mid transition hover:border-room-line-strong hover:text-room-text"
                  onclick={() => downloadStore.cancel(gallery.id)}
                  title="Pause download"
                  aria-label="Pause download"
                >
                  <Icon name="close" class="size-4" />
                </button>
              {:else if failedPages.length > 0}
                <button
                  class="icon-tip flex items-center justify-center gap-1.5 rounded-[3px] border border-room-line px-3 text-room-text-mid transition hover:border-room-line-strong hover:text-room-text"
                  onclick={retryFailed}
                  title="Retry failed pages"
                  aria-label="Retry failed pages"
                >
                  <Icon name="refresh" class="size-4" />
                </button>
              {:else if downloaded}
                <button
                  class="icon-tip flex items-center justify-center gap-1.5 rounded-[3px] border border-room-line px-3 text-room-accent transition hover:border-room-line-strong"
                  onclick={openFolder}
                  title="Open downloaded folder"
                  aria-label="Open downloaded folder"
                >
                  <Icon name="folder" class="size-4" />
                </button>
              {:else}
                <button
                  class="icon-tip flex items-center justify-center gap-1.5 rounded-[3px] border border-room-line px-3 text-room-text-mid transition hover:border-room-line-strong hover:text-room-text"
                  onclick={() => downloadStore.start(gallery.id, gallery.title)}
                  title={dl?.paused ? 'Resume download' : 'Download'}
                  aria-label={dl?.paused ? 'Resume download' : 'Download'}
                >
                  <Icon name="download" class="size-4" />
                </button>
              {/if}
            </div>

            {#if prog && prog.total > 0}
              <div class="flex items-center gap-2 text-[10.5px] text-room-text-low">
                <div class="h-1 flex-1 overflow-hidden rounded-full bg-room-bg">
                  <div
                    class="h-full bg-room-accent"
                    style="width: {libraryStore.percent(gallery.id)}%"
                  ></div>
                </div>
                <span class="tabular-nums">{prog.page}/{prog.total}</span>
              </div>
            {/if}

            {#if galleryStore.view === 'history'}
              <button
                class="text-left text-[11px] text-room-text-low hover:text-room-text"
                onclick={dropHistory}>Remove from history</button
              >
            {/if}
            {#if galleryStore.view === 'downloads' && failedPages.length > 0}
              <button
                class="text-left text-[11px] text-room-warn hover:text-room-text"
                onclick={retryFailed}>Retry {failedPages.length} failed page{failedPages.length === 1 ? '' : 's'}</button
              >
            {/if}
            {#if galleryStore.view === 'downloads'}
              <button
                class="text-left text-[11px] text-room-text-low hover:text-room-text"
                onclick={dropDownload}>Remove from downloads</button
              >
            {/if}
            {#if dl}
              <div class="text-[11px]">
                {#if dl.error === 'already downloaded'}
                  <span class="text-room-accent">Already downloaded</span>
                {:else if dl.error}
                  <span class="text-room-danger">Download failed: {dl.error}</span>
                {:else if dl.paused}
                  <span class="text-room-warn">Paused · {dl.done}/{dl.total || '?'}</span>
                {:else if dl.finished && dl.failed}
                  <span class="text-room-warn"
                    >Partial · {dl.total - dl.failed}/{dl.total} saved · {dl.failed} failed</span
                  >
                {:else if dl.finished}
                  <span class="text-room-accent">Downloaded · {dl.total} pages</span>
                {:else}
                  <span class="text-room-text-mid"
                    >Downloading… {dl.done}/{dl.total || '?'}</span
                  >
                {/if}
              </div>
            {/if}
          </div>

          {#each metaRows as row (row.label)}
            <div>
              <div class="mb-1 font-mono text-[9.5px] uppercase tracking-[0.2em] text-room-text-low">
                {row.label}
              </div>
              <div class="flex flex-wrap gap-1">
                {#each row.values as value, i (`${value}-${i}`)}
                  <button
                    class="rounded-[3px] border border-room-line px-1.5 py-0.5 text-[11px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
                    onclick={() => searchFor(row.prefix, value)}>{value}</button
                  >
                {/each}
              </div>
            </div>
          {/each}

          {#if gallery.tags.length}
            <div>
              <div class="mb-1 font-mono text-[9.5px] uppercase tracking-[0.2em] text-room-text-low">
                tags · {gallery.tags.length}
              </div>
              <div class="flex flex-wrap gap-1">
                {#each gallery.tags as tag, i (`${tag}-${i}`)}
                  {@const parsed = parseTag(tag)}
                  <button
                    class="rounded-[3px] border px-1.5 py-0.5 text-[11px] {parsed.kind === 'male'
                      ? 'border-room-male/40 text-room-male'
                      : parsed.kind === 'female'
                        ? 'border-room-female/40 text-room-female'
                        : 'border-room-line text-room-text-mid hover:text-room-text'}"
                    onclick={() => galleryStore.applyQuery(tagToQuery(tag))}>{parsed.label}</button
                  >
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/key}
  {:else}
    <div class="grid h-full place-items-center px-6">
      <div class="text-center">
        <div class="font-mono text-[10px] uppercase tracking-[0.25em] text-room-text-low">
          no selection
        </div>
        <p class="mt-2 max-w-[220px] text-[12px] text-room-text-mid">
          Select a gallery to inspect tags, progress, and download actions.
        </p>
      </div>
    </div>
  {/if}
</aside>
