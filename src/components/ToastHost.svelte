<script lang="ts">
  import { uiStore } from '$lib/ui-store.svelte';
  import { focusTrap } from '$lib/focus-trap';
  import Icon from './Icon.svelte';

  const confirm = $derived(uiStore.confirmState);

  function toneClass(tone: string): string {
    if (tone === 'success') return 'border-room-accent text-room-accent';
    if (tone === 'warning') return 'border-room-warn text-room-warn';
    if (tone === 'danger') return 'border-room-danger text-room-danger';
    return 'border-room-line text-room-text';
  }

  function onkeydown(e: KeyboardEvent) {
    if (!confirm) return;
    if (e.key === 'Escape') uiStore.answerConfirm(false);
  }
</script>

<svelte:window {onkeydown} />

<div class="fixed bottom-4 left-1/2 z-[70] flex w-[min(420px,calc(100vw-24px))] -translate-x-1/2 flex-col gap-2 pointer-events-none">
  {#each uiStore.toasts as toast (toast.id)}
    <div
      class="pointer-events-auto flex items-center gap-2 rounded-[6px] border bg-room-panel-hi px-3 py-2 shadow-[0_12px_36px_rgba(0,0,0,0.42)] {toneClass(toast.tone)}"
      role="status"
    >
      <span class="shrink-0">
        <Icon
          name={toast.tone === 'success' ? 'check' : toast.tone === 'warning' ? 'refresh' : toast.tone === 'danger' ? 'close' : 'doc'}
          class="size-3.5"
        />
      </span>
      <span class="min-w-0 flex-1 text-[12px] text-room-text">{toast.message}</span>
      <button
        class="grid size-5 shrink-0 place-items-center rounded-[3px] text-room-text-low hover:bg-room-panel hover:text-room-text"
        onclick={() => uiStore.dismissToast(toast.id)}
        aria-label="Dismiss notification"
      >
        <Icon name="close" class="size-3" />
      </button>
    </div>
  {/each}
</div>

{#if confirm}
  <div class="fixed inset-0 z-[80] grid place-items-center bg-black/55 p-5">
    <div
      class="w-full max-w-sm rounded-[6px] border border-room-line bg-room-panel shadow-[0_18px_52px_rgba(0,0,0,0.55)] focus:outline-none"
      role="dialog"
      aria-modal="true"
      aria-label={confirm.title}
      tabindex="-1"
      use:focusTrap
    >
      <div class="border-b border-room-line px-4 py-3">
        <div class="text-[13px] text-room-text">{confirm.title}</div>
        <p class="mt-1 text-[12px] leading-relaxed text-room-text-mid">{confirm.message}</p>
      </div>
      <div class="flex justify-end gap-1.5 px-4 py-3">
        <button
          class="rounded-[3px] border border-room-line px-3 py-1.5 text-[12px] text-room-text-mid hover:border-room-line-strong hover:text-room-text"
          onclick={() => uiStore.answerConfirm(false)}
        >
          Cancel
        </button>
        <button
          class="rounded-[3px] px-3 py-1.5 text-[12px] font-medium text-room-floor transition hover:brightness-110 {confirm.tone === 'danger'
            ? 'bg-room-danger'
            : 'bg-room-accent'}"
          onclick={() => uiStore.answerConfirm(true)}
        >
          {confirm.confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
