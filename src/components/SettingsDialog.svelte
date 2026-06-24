<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { settingsStore, type Theme } from '$lib/settings-store.svelte';
  import { galleryStore } from '$lib/gallery-store.svelte';
  import { updateStore } from '$lib/update-store.svelte';
  import { uiStore } from '$lib/ui-store.svelte';
  import { clearImageCache, imageCacheSize, defaultDownloadDir } from '$lib/api';
  import { LANGUAGES, type Language } from '$lib/types';
  import { focusTrap } from '$lib/focus-trap';
  import Icon from './Icon.svelte';

  let { onclose }: { onclose: () => void } = $props();

  const sections = [
    { id: 'appearance', label: 'Appearance' },
    { id: 'reader', label: 'Reader' },
    { id: 'content', label: 'Content' },
    { id: 'downloads', label: 'Downloads' },
    { id: 'about', label: 'About' }
  ] as const;
  type Section = (typeof sections)[number]['id'];
  let activeSection = $state<Section>('appearance');

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
  let defaultDir = $state('');

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
    if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
    return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`;
  }

  async function refreshCache() {
    cacheBytes = await imageCacheSize();
  }

  onMount(async () => {
    refreshCache();
    defaultDir = (await defaultDownloadDir()) ?? '';
  });

  function selectLanguage(l: Language) {
    settingsStore.setLanguage(l);
    galleryStore.load(1);
  }

  function addTag(e: Event) {
    e.preventDefault();
    settingsStore.addBlacklist(newTag);
    newTag = '';
    galleryStore.load(galleryStore.page);
  }

  function removeTag(tag: string) {
    settingsStore.removeBlacklist(tag);
    galleryStore.load(galleryStore.page);
  }

  async function chooseFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') settingsStore.setDownloadDir(dir);
  }

  async function clearCache() {
    const ok = await uiStore.confirm({
      title: 'Clear image cache?',
      message: 'Cached images will be downloaded again when needed.',
      confirmLabel: 'Clear cache',
      tone: 'danger'
    });
    if (!ok) return;
    await clearImageCache();
    await refreshCache();
    uiStore.toast('Image cache cleared.', 'success');
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }

  function onSectionKeydown(e: KeyboardEvent, index: number) {
    let next = index;
    if (e.key === 'ArrowDown' || e.key === 'ArrowRight') next = (index + 1) % sections.length;
    else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') next = (index - 1 + sections.length) % sections.length;
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = sections.length - 1;
    else return;
    e.preventDefault();
    activeSection = sections[next].id;
  }

  function onGridScaleInput(e: Event) {
    const value = Number((e.currentTarget as HTMLInputElement).value);
    settingsStore.setGridScalePct(value);
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
    class="grid h-[560px] max-h-[88vh] w-full max-w-2xl grid-rows-[auto_minmax(0,1fr)] overflow-hidden rounded-[6px] border border-room-line bg-room-panel shadow-[0_16px_48px_rgba(0,0,0,0.5)] focus:outline-none max-sm:h-[min(640px,92vh)]"
    role="dialog"
    aria-modal="true"
    aria-label="Settings"
    tabindex="-1"
    use:focusTrap
  >
    <div class="flex items-center justify-between border-b border-room-line px-4 py-3">
      <div>
        <div class="font-mono text-[10px] uppercase tracking-[0.2em] text-room-text-low">settings</div>
        <div class="mt-0.5 text-[13px] text-room-text">
          {sections.find((s) => s.id === activeSection)?.label}
        </div>
      </div>
      <button
        class="grid size-7 place-items-center rounded-[3px] text-room-text-mid hover:bg-room-panel-hi hover:text-room-text"
        onclick={onclose}
        aria-label="Close"
      >
        <Icon name="close" class="size-4" />
      </button>
    </div>

    <div class="grid min-h-0 grid-cols-[150px_minmax(0,1fr)] max-sm:grid-cols-1 max-sm:grid-rows-[auto_minmax(0,1fr)]">
      <div
        class="flex flex-col border-r border-room-line bg-room-panel/40 py-2 max-sm:flex-row max-sm:overflow-x-auto max-sm:border-b max-sm:border-r-0 max-sm:py-0"
        role="tablist"
        aria-label="Settings sections"
      >
        {#each sections as section, i (section.id)}
          {@const isActive = activeSection === section.id}
          <button
            role="tab"
            aria-selected={isActive}
            class="relative px-4 py-2 text-left font-mono text-[11px] uppercase tracking-[0.16em] transition-colors duration-150 max-sm:shrink-0 {isActive
              ? 'text-room-accent'
              : 'text-room-text-mid hover:text-room-text'}"
            onclick={() => (activeSection = section.id)}
            onkeydown={(e) => onSectionKeydown(e, i)}
          >
            {section.label}
            {#if isActive}
              <span class="absolute bottom-1 left-4 right-4 h-px bg-room-accent" aria-hidden="true"></span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="min-h-0 overflow-auto">
        <div class="flex flex-col gap-5 p-4">
          {#if activeSection === 'appearance'}
            <div id="settings-appearance" role="tabpanel" aria-label="Appearance">
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
            </div>

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
              <div class="mb-2 flex items-center justify-between gap-3">
                <label class="text-[12px] text-room-text" for="grid-scale">Grid scale</label>
                <span class="font-mono text-[11px] tabular-nums text-room-accent">
                  {settingsStore.gridScalePct}%
                </span>
              </div>
              <input
                id="grid-scale"
                class="h-7 w-full"
                type="range"
                min="100"
                max="200"
                step="25"
                value={settingsStore.gridScalePct}
                oninput={onGridScaleInput}
              />
              <div class="mt-0.5 flex justify-between font-mono text-[9.5px] tabular-nums text-room-text-low">
                <span>100%</span>
                <span>150%</span>
                <span>200%</span>
              </div>
              <p class="mt-1.5 font-mono text-[10px] tabular-nums text-room-text-low">
                {settingsStore.effectiveTileMin}px
              </p>
            </section>
          {:else if activeSection === 'reader'}
            <div id="settings-reader" role="tabpanel" aria-label="Reader">
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
            </div>
          {:else if activeSection === 'content'}
            <div id="settings-content" role="tabpanel" aria-label="Content">
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
            </div>

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
                      onclick={() => removeTag(tag)}
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
          {:else if activeSection === 'downloads'}
            <div id="settings-downloads" role="tabpanel" aria-label="Downloads">
              <div class="mb-2 text-[12px] text-room-text">Download folder</div>
              <div class="flex items-center gap-1.5">
                <div
                  class="min-w-0 flex-1 truncate rounded-[3px] border border-room-line bg-room-bg px-2.5 py-1.5 font-mono text-[11px] text-room-text-mid"
                  title={settingsStore.downloadDir || defaultDir}
                >
                  {settingsStore.downloadDir || defaultDir || '…'}
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
              {#if !settingsStore.downloadDir}
                <p class="mt-1.5 text-[11px] text-room-text-low">
                  Your system Downloads folder, used until you choose another.
                </p>
              {/if}
            </div>

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
          {:else if activeSection === 'about'}
            <div id="settings-about" role="tabpanel" aria-label="About" class="flex flex-col gap-2">
              <div class="text-[13px] text-room-text">Vista</div>
              <div class="font-mono text-[11px] tabular-nums text-room-text-mid">
                {updateStore.currentVersion ? `v${updateStore.currentVersion}` : ''}
              </div>
              <p class="text-[11.5px] leading-relaxed text-room-text-low">
                A cross-platform gallery client built with Rust + Tauri + Svelte.
              </p>
            </div>

            <section>
              <div class="mb-2 text-[12px] text-room-text">Updates</div>
              <div class="flex items-center gap-2">
                <span class="font-mono text-[11px] tabular-nums text-room-text-mid">
                  {#if updateStore.status === 'checking'}
                    checking…
                  {:else if updateStore.status === 'available'}
                    <span class="text-room-accent">v{updateStore.newVersion} available</span>
                  {:else if updateStore.status === 'downloading'}
                    downloading {updateStore.percent}%
                  {:else if updateStore.status === 'ready'}
                    restarting…
                  {:else if updateStore.status === 'uptodate'}
                    up to date
                  {:else if updateStore.status === 'error'}
                    <span class="text-room-danger">check failed</span>
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
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>
