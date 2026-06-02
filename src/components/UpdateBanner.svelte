<script lang="ts">
  import { updateStore } from '$lib/update-store.svelte';
  import Icon from './Icon.svelte';

  const show = $derived(
    updateStore.status === 'available' ||
      updateStore.status === 'downloading' ||
      updateStore.status === 'ready'
  );
</script>

{#if show}
  <div
    class="fixed bottom-4 right-4 z-[55] w-[320px] rounded-[6px] border border-room-line bg-room-panel-hi shadow-[0_16px_48px_rgba(0,0,0,0.5)]"
    role="status"
  >
    <div class="flex items-start gap-3 p-3.5">
      <span class="mt-0.5 text-room-accent"><Icon name="download" class="size-4" /></span>
      <div class="min-w-0 flex-1">
        {#if updateStore.status === 'available'}
          <div class="text-[12.5px] text-room-text">Update available</div>
          <div class="font-mono text-[10.5px] text-room-text-mid">
            v{updateStore.currentVersion} → v{updateStore.newVersion}
          </div>
          <div class="mt-2.5 flex gap-1.5">
            <button
              class="flex-1 rounded-[3px] bg-room-accent py-1.5 text-[12px] font-medium text-room-floor transition hover:brightness-110"
              onclick={() => updateStore.install()}
            >
              Update & restart
            </button>
            <button
              class="rounded-[3px] border border-room-line px-3 text-[12px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
              onclick={() => updateStore.dismiss()}
            >
              Later
            </button>
          </div>
        {:else if updateStore.status === 'downloading'}
          <div class="text-[12.5px] text-room-text">Downloading update…</div>
          <div class="mt-2 h-1 overflow-hidden rounded-full bg-room-bg">
            <div class="h-full bg-room-accent" style="width: {updateStore.percent}%"></div>
          </div>
          <div class="mt-1 text-right font-mono text-[10px] tabular-nums text-room-text-low">
            {updateStore.percent}%
          </div>
        {:else if updateStore.status === 'ready'}
          <div class="text-[12.5px] text-room-text">Restarting to apply update…</div>
        {/if}
      </div>
    </div>
  </div>
{/if}
