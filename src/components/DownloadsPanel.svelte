<script lang="ts">
  import { openDownloadFolder } from '$lib/api';
  import { downloadStore, type DownloadState } from '$lib/download-store.svelte';
  import { uiStore } from '$lib/ui-store.svelte';
  import Icon from './Icon.svelte';

  let { onclose }: { onclose: () => void } = $props();

  const jobs = $derived(downloadStore.list);

  function percent(j: DownloadState): number {
    if (j.total <= 0) return 0;
    return Math.min(100, Math.round((j.done / j.total) * 100));
  }

  function statusText(j: DownloadState): string {
    if (j.error === 'already downloaded') return 'Already downloaded';
    if (j.error) return `Failed · ${j.error}`;
    if (j.paused) return `Paused · ${j.done}/${j.total || '?'}`;
    if (j.finished && j.failed) return `Partial · ${j.total - j.failed}/${j.total} saved`;
    if (j.finished) return `Done · ${j.total} pages`;
    return `Downloading · ${j.done}/${j.total || '?'}`;
  }

  function failedText(j: DownloadState): string {
    const failed = j.failedPages?.length ?? j.failed ?? 0;
    if (failed <= 0) return '';
    return `${failed} page${failed === 1 ? '' : 's'} need retry`;
  }

  async function openFolder(id: number) {
    await openDownloadFolder(id).catch(() =>
      uiStore.toast('Could not open the download folder.', 'danger')
    );
  }

  async function removeJob(id: number) {
    const ok = await uiStore.confirm({
      title: 'Remove download from panel?',
      message: 'This only clears the visible job entry. Downloaded files are not deleted.',
      confirmLabel: 'Remove',
      tone: 'danger'
    });
    if (!ok) return;
    downloadStore.remove(id);
    uiStore.toast('Download removed from panel.', 'success');
  }

  async function clearFinished() {
    const ok = await uiStore.confirm({
      title: 'Clear finished downloads?',
      message: 'Finished and failed entries will be removed from the panel.',
      confirmLabel: 'Clear',
      tone: 'danger'
    });
    if (!ok) return;
    downloadStore.clearFinished();
    uiStore.toast('Finished downloads cleared.', 'success');
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window {onkeydown} />

<div
  class="fixed inset-0 z-40"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onclose();
  }}
>
  <div
    class="absolute right-3 top-12 flex max-h-[80vh] w-[360px] flex-col overflow-hidden rounded-[6px] border border-room-line bg-room-panel shadow-[0_16px_48px_rgba(0,0,0,0.5)]"
    role="dialog"
    aria-modal="false"
    aria-label="Downloads"
  >
    <div class="flex items-center justify-between border-b border-room-line px-3.5 py-2.5">
      <span class="font-mono text-[11px] uppercase tracking-[0.2em] text-room-text-mid">downloads</span>
      <div class="flex items-center gap-1">
        <button
          class="rounded-[3px] px-2 py-0.5 text-[11px] text-room-text-low hover:text-room-text disabled:opacity-40"
          onclick={clearFinished}
          disabled={!jobs.some((j) => j.finished || !!j.error)}
        >
          Clear done
        </button>
        <button
          class="grid size-6 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text"
          onclick={onclose}
          aria-label="Close"
        >
          <Icon name="close" class="size-3.5" />
        </button>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-auto">
      {#if jobs.length === 0}
        <div class="grid place-items-center px-6 py-10 text-center">
          <div>
            <div class="font-mono text-[10px] uppercase tracking-[0.22em] text-room-text-low">
              no downloads
            </div>
            <p class="mt-2 text-[12px] text-room-text-mid">
              Start a download from any gallery to track it here.
            </p>
          </div>
        </div>
      {:else}
        {#each jobs as j (j.id)}
          <div class="flex flex-col gap-1.5 border-b border-room-line/60 px-3.5 py-2.5">
            <div class="flex items-start gap-2">
              <div class="min-w-0 flex-1">
                <div class="truncate text-[12px] text-room-text">{j.title || `#${j.id}`}</div>
                <div
                  class="font-mono text-[10px] tabular-nums {j.error
                    ? 'text-room-danger'
                    : j.paused
                      ? 'text-room-warn'
                      : j.finished
                        ? 'text-room-accent'
                        : 'text-room-text-mid'}"
                >
                  {statusText(j)}
                </div>
                {#if failedText(j)}
                  <div class="mt-0.5 text-[10.5px] text-room-warn">{failedText(j)}</div>
                {/if}
              </div>
              <div class="flex shrink-0 items-center gap-0.5">
                {#if j.running}
                  <button
                    class="icon-tip icon-tip-left grid size-6 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text"
                    onclick={() => downloadStore.cancel(j.id)}
                    title="Pause"
                    aria-label="Pause"
                  >
                    <Icon name="pause" class="size-3.5" />
                  </button>
                {:else if j.paused || j.error}
                  <button
                    class="icon-tip icon-tip-left grid size-6 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text"
                    onclick={() => downloadStore.start(j.id, j.title, j.failedPages)}
                    title={j.failedPages?.length ? 'Retry failed pages' : j.error ? 'Retry' : 'Resume'}
                    aria-label={j.failedPages?.length ? 'Retry failed pages' : j.error ? 'Retry' : 'Resume'}
                  >
                    <Icon name="play" class="size-3.5" />
                  </button>
                {/if}
                {#if !j.running && !j.paused && !j.error && j.failedPages?.length}
                  <button
                    class="icon-tip icon-tip-left grid size-6 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text"
                    onclick={() => downloadStore.start(j.id, j.title, j.failedPages)}
                    title="Retry failed pages"
                    aria-label="Retry failed pages"
                  >
                    <Icon name="refresh" class="size-3.5" />
                  </button>
                {/if}
                {#if j.folder}
                  <button
                    class="icon-tip icon-tip-left grid size-6 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text"
                    onclick={() => openFolder(j.id)}
                    title="Open folder"
                    aria-label="Open folder"
                  >
                    <Icon name="folder" class="size-3.5" />
                  </button>
                {/if}
                {#if !j.running}
                  <button
                    class="icon-tip icon-tip-left grid size-6 place-items-center rounded-[3px] text-room-text-low hover:bg-room-panel-hi hover:text-room-text"
                    onclick={() => removeJob(j.id)}
                    title="Remove"
                    aria-label="Remove"
                  >
                    <Icon name="close" class="size-3.5" />
                  </button>
                {/if}
              </div>
            </div>
            <div class="h-1 overflow-hidden rounded-full bg-room-bg">
              <div
                class="h-full {j.error ? 'bg-room-danger' : j.paused || j.failed ? 'bg-room-warn' : 'bg-room-accent'}"
                style="width: {percent(j)}%"
              ></div>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>
