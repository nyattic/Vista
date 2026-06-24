<script lang="ts">
  import { onMount } from 'svelte';
  import { galleryStore } from '$lib/gallery-store.svelte';
  import { readerStore } from '$lib/reader-store.svelte';
  import { settingsStore } from '$lib/settings-store.svelte';
  import { downloadStore } from '$lib/download-store.svelte';
  import { libraryStore } from '$lib/library-store.svelte';
  import { updateStore } from '$lib/update-store.svelte';
  import Header from './components/Header.svelte';
  import Toolbar from './components/Toolbar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import GalleryGrid from './components/GalleryGrid.svelte';
  import Pager from './components/Pager.svelte';
  import Reader from './components/Reader.svelte';
  import SettingsDialog from './components/SettingsDialog.svelte';
  import DownloadsPanel from './components/DownloadsPanel.svelte';
  import UpdateBanner from './components/UpdateBanner.svelte';
  import ToastHost from './components/ToastHost.svelte';

  let systemLight = $state(false);
  let showSettings = $state(false);
  let showDownloads = $state(false);
  let showSidebar = $state(false);

  onMount(() => {
    const mq = window.matchMedia('(prefers-color-scheme: light)');
    systemLight = mq.matches;
    const handler = (e: MediaQueryListEvent) => (systemLight = e.matches);
    mq.addEventListener('change', handler);
    settingsStore.init();
    downloadStore.init();
    libraryStore.init();
    updateStore.init();
    galleryStore.load(1);
    return () => {
      mq.removeEventListener('change', handler);
      downloadStore.destroy();
    };
  });

  $effect(() => {
    const resolved =
      settingsStore.theme === 'system' ? (systemLight ? 'light' : 'dark') : settingsStore.theme;
    if (resolved === 'light') document.documentElement.dataset.theme = 'light';
    else delete document.documentElement.dataset.theme;
  });
</script>

<main class="grid h-screen grid-rows-[48px_36px_minmax(0,1fr)] bg-room-bg text-room-text">
  <Header
    sidebarOpen={showSidebar}
    ontogglesidebar={() => (showSidebar = !showSidebar)}
    onopensettings={() => (showSettings = true)}
    onopendownloads={() => (showDownloads = !showDownloads)}
  />
  <Toolbar />
  <section class="relative grid min-h-0 grid-cols-1 lg:grid-cols-[300px_minmax(0,1fr)]">
    {#if showSidebar}
      <button
        class="absolute inset-0 z-20 bg-black/35 lg:hidden"
        aria-label="Close details sidebar"
        onclick={() => (showSidebar = false)}
      ></button>
    {/if}
    <Sidebar
      class="z-30 max-lg:absolute max-lg:inset-y-0 max-lg:left-0 max-lg:w-[min(320px,86vw)] max-lg:shadow-[16px_0_36px_rgba(0,0,0,0.45)] {showSidebar
        ? 'max-lg:flex'
        : 'max-lg:hidden'}"
    />
    <div class="order-2 flex min-h-0 flex-col">
      <GalleryGrid />
      <Pager />
    </div>
  </section>
</main>

{#if readerStore.isOpen}
  <Reader />
{/if}

{#if showDownloads}
  <DownloadsPanel onclose={() => (showDownloads = false)} />
{/if}

<UpdateBanner />
<ToastHost />

{#if showSettings}
  <SettingsDialog onclose={() => (showSettings = false)} />
{/if}
