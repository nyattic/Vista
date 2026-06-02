<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { settingsStore, type Theme } from '$lib/settings-store.svelte';
  import { galleryStore } from '$lib/gallery-store.svelte';
  import { updateStore } from '$lib/update-store.svelte';
  import { clearImageCache, imageCacheSize } from '$lib/api';
  import { LANGUAGES, type Language } from '$lib/types';
  import Icon from './Icon.svelte';

  let { onclose }: { onclose: () => void } = $props();

  const themes: { value: Theme; label: string }[] = [
    { value: 'system', label: 'System' },
    { value: 'dark', label: 'Dark' },
    { value: 'light', label: 'Light' }
  ];

  const readingModes = [
    { value: 'continuous', label: 'Continuous' },
    { value: 'paged', label: 'Paged' },
    { value: 'spread', label: 'Spread' }
  ] as const;

  const readingDirections = [
    { value: 'ltr', label: 'Left → Right' },
    { value: 'rtl', label: 'Right → Left' }
  ] as const;

  let newTag = $state('');
  let cacheBytes = $state<number | null>(null);

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
    if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
    return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`;
  }

  async function refreshCache() {
    cacheBytes = await imageCacheSize();
  }

  onMount(refreshCache);

  function selectLanguage(l: Language) {
    settingsStore.setLanguage(l);
    galleryStore.load(1);
  }

  function addTag(e: Event) {
    e.preventDefault();
    settingsStore.addBlacklist(newTag);
    newTag = '';
  }

  async function chooseFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') settingsStore.setDownloadDir(dir);
  }

  async function clearCache() {
    await clearImageCache();
    await refreshCache();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window {onkeydown} />

<div
  class="fixed inset-0 z-40 grid place-items-center bg-black/50 p-6"
  role="presentation"
  onclick={(e) => {
    if (e.target === e.currentTarget) onclose();
  }}
>
  <div
    class="flex max-h-[86vh] w-full max-w-md flex-col overflow-hidden rounded-[6px] border border-room-line bg-room-panel shadow-[0_16px_48px_rgba(0,0,0,0.5)]"
    role="dialog"
    aria-modal="true"
  >
    <div class="flex items-center justify-between border-b border-room-line px-4 py-3">
      <span class="font-mono text-[11px] uppercase tracking-[0.2em] text-room-text-mid">settings</span>
      <button
        class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text"
        onclick={onclose}
        aria-label="Close"
      >
        <Icon name="close" class="size-4" />
      </button>
    </div>

    <div class="flex flex-col gap-5 overflow-auto p-4">
      <section>
        <div class="mb-2 text-[12px] text-room-text">Theme</div>
        <div class="flex gap-1.5">
          {#each themes as t (t.value)}
            <button
              class="flex-1 rounded-[3px] border px-3 py-1.5 text-[12px] transition {settingsStore.theme ===
              t.value
                ? 'border-room-accent bg-room-panel-hi text-room-accent'
                : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
              onclick={() => settingsStore.setTheme(t.value)}>{t.label}</button
            >
          {/each}
        </div>
      </section>

      <section>
        <div class="mb-2 text-[12px] text-room-text">Default language</div>
        <div class="flex flex-wrap gap-1.5">
          {#each LANGUAGES as l (l.value)}
            <button
              class="rounded-[3px] border px-3 py-1.5 text-[12px] transition {settingsStore.language ===
              l.value
                ? 'border-room-accent bg-room-panel-hi text-room-accent'
                : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
              onclick={() => selectLanguage(l.value)}>{l.label}</button
            >
          {/each}
        </div>
      </section>

      <section>
        <div class="mb-2 text-[12px] text-room-text">Grid size</div>
        <div class="flex gap-1.5">
          {#each settingsStore.tileSizes as size (size.value)}
            <button
              class="flex-1 rounded-[3px] border px-3 py-1.5 text-[12px] transition {settingsStore.tileMin ===
              size.value
                ? 'border-room-accent bg-room-panel-hi text-room-accent'
                : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
              onclick={() => settingsStore.setTileMin(size.value)}>{size.label}</button
            >
          {/each}
        </div>
      </section>

      <section>
        <div class="mb-2 text-[12px] text-room-text">Reader layout</div>
        <div class="flex gap-1.5">
          {#each readingModes as m (m.value)}
            <button
              class="flex-1 rounded-[3px] border px-3 py-1.5 text-[12px] transition {settingsStore.readingMode ===
              m.value
                ? 'border-room-accent bg-room-panel-hi text-room-accent'
                : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
              onclick={() => settingsStore.setReadingMode(m.value)}>{m.label}</button
            >
          {/each}
        </div>
        {#if settingsStore.readingMode !== 'continuous'}
          <div class="mt-1.5 flex gap-1.5">
            {#each readingDirections as d (d.value)}
              <button
                class="flex-1 rounded-[3px] border px-3 py-1.5 text-[12px] transition {settingsStore.readingDirection ===
                d.value
                  ? 'border-room-accent bg-room-panel-hi text-room-accent'
                  : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
                onclick={() => settingsStore.setReadingDirection(d.value)}>{d.label}</button
              >
            {/each}
          </div>
        {/if}
      </section>

      <section>
        <div class="mb-2 text-[12px] text-room-text">Blacklist tags</div>
        <form class="flex gap-1.5" onsubmit={addTag}>
          <input
            class="h-8 min-w-0 flex-1 rounded-[3px] border border-room-line bg-room-bg px-2.5 text-[12px] text-room-text placeholder:text-room-text-low focus:border-room-accent focus:outline-none"
            placeholder="e.g. guro, scat, female:netorare"
            bind:value={newTag}
            spellcheck="false"
          />
          <button
            type="submit"
            class="rounded-[3px] border border-room-line px-3 text-[12px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
            >Add</button
          >
        </form>
        {#if settingsStore.blacklist.length}
          <div class="mt-2 flex flex-wrap gap-1.5">
            {#each settingsStore.blacklist as tag (tag)}
              <button
                class="flex items-center gap-1 rounded-[3px] border border-room-line bg-room-bg px-2 py-0.5 text-[11px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
                onclick={() => settingsStore.removeBlacklist(tag)}
                title="Remove"
              >
                {tag}
                <Icon name="close" class="size-3" />
              </button>
            {/each}
          </div>
        {:else}
          <p class="mt-1.5 text-[11px] text-room-text-low">
            Galleries containing these tags are hidden from results.
          </p>
        {/if}
      </section>

      <section>
        <div class="mb-2 text-[12px] text-room-text">Download folder</div>
        <div class="flex items-center gap-1.5">
          <div
            class="min-w-0 flex-1 truncate rounded-[3px] border border-room-line bg-room-bg px-2.5 py-1.5 font-mono text-[11px] text-room-text-mid"
            title={settingsStore.downloadDir}
          >
            {settingsStore.downloadDir || '(system default)'}
          </div>
          <button
            class="flex items-center gap-1.5 rounded-[3px] border border-room-line px-3 py-1.5 text-[12px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
            onclick={chooseFolder}
          >
            <Icon name="folder" class="size-4" /> Choose
          </button>
          {#if settingsStore.downloadDir}
            <button
              class="rounded-[3px] border border-room-line px-3 py-1.5 text-[12px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
              onclick={() => settingsStore.setDownloadDir('')}>Reset</button
            >
          {/if}
        </div>
      </section>

      <section>
        <div class="mb-2 text-[12px] text-room-text">Image cache</div>
        <div class="mb-2 flex flex-wrap gap-1.5">
          {#each settingsStore.cacheLimits as limit (limit.value)}
            <button
              class="rounded-[3px] border px-3 py-1.5 text-[12px] transition {settingsStore.cacheLimitMb ===
              limit.value
                ? 'border-room-accent bg-room-panel-hi text-room-accent'
                : 'border-room-line text-room-text-mid hover:border-room-line-strong hover:text-room-text'}"
              onclick={() => settingsStore.setCacheLimitMb(limit.value)}>{limit.label}</button
            >
          {/each}
        </div>
        <div class="flex items-center gap-2">
          <span class="font-mono text-[12px] tabular-nums text-room-text-mid">
            {cacheBytes === null ? '…' : formatBytes(cacheBytes)}
          </span>
          <button
            class="ml-auto flex items-center gap-1.5 rounded-[3px] border border-room-line px-3 py-1.5 text-[12px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
            onclick={clearCache}
          >
            <Icon name="trash" class="size-4" /> Clear cache
          </button>
        </div>
      </section>

      <section>
        <div class="mb-2 text-[12px] text-room-text">Updates</div>
        <div class="flex items-center gap-2">
          <span class="font-mono text-[11px] tabular-nums text-room-text-mid">
            {updateStore.currentVersion ? `v${updateStore.currentVersion}` : '…'}
            {#if updateStore.status === 'checking'}
              · checking…
            {:else if updateStore.status === 'available'}
              · <span class="text-room-accent">v{updateStore.newVersion} available</span>
            {:else if updateStore.status === 'downloading'}
              · downloading {updateStore.percent}%
            {:else if updateStore.status === 'ready'}
              · restarting…
            {:else if updateStore.status === 'uptodate'}
              · up to date
            {:else if updateStore.status === 'error'}
              · <span class="text-[#ff6b6b]">check failed</span>
            {/if}
          </span>
          {#if updateStore.status === 'available'}
            <button
              class="ml-auto rounded-[3px] bg-room-accent px-3 py-1.5 text-[12px] font-medium text-room-floor transition hover:brightness-110"
              onclick={() => updateStore.install()}
            >
              Update & restart
            </button>
          {:else}
            <button
              class="ml-auto rounded-[3px] border border-room-line px-3 py-1.5 text-[12px] text-room-text-mid hover:border-room-line-strong hover:text-room-text disabled:opacity-50"
              onclick={() => updateStore.check(false)}
              disabled={updateStore.status === 'checking' || updateStore.status === 'downloading'}
            >
              Check for updates
            </button>
          {/if}
        </div>
      </section>

      <div class="border-t border-room-line pt-3 font-mono text-[10.5px] text-room-text-low">
        Vista · Rust + Tauri + Svelte
      </div>
    </div>
  </div>
</div>
