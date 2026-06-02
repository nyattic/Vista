<script lang="ts">
  let {
    name,
    class: cls = 'size-4',
    filled = false
  }: { name: string; class?: string; filled?: boolean } = $props();
</script>

<svg
  class={cls}
  viewBox="0 0 24 24"
  fill={filled ? 'currentColor' : 'none'}
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'search'}
    <circle cx="11" cy="11" r="7" />
    <path d="m20 20-3.5-3.5" />
  {:else if name === 'settings'}
    <circle cx="12" cy="12" r="3" />
    <path
      d="M12 2.5a1 1 0 0 1 1 1v.7a8 8 0 0 1 2 .85l.5-.5a1 1 0 1 1 1.4 1.4l-.5.5a8 8 0 0 1 .85 2h.7a1 1 0 1 1 0 2h-.7a8 8 0 0 1-.85 2l.5.5a1 1 0 1 1-1.4 1.4l-.5-.5a8 8 0 0 1-2 .85v.7a1 1 0 1 1-2 0v-.7a8 8 0 0 1-2-.85l-.5.5a1 1 0 0 1-1.4-1.4l.5-.5a8 8 0 0 1-.85-2h-.7a1 1 0 1 1 0-2h.7a8 8 0 0 1 .85-2l-.5-.5a1 1 0 0 1 1.4-1.4l.5.5a8 8 0 0 1 2-.85v-.7a1 1 0 0 1 1-1Z"
    />
  {:else if name === 'heart'}
    <path d="M12 20s-7-4.6-9.3-9A4.7 4.7 0 0 1 12 6.5 4.7 4.7 0 0 1 21.3 11C19 15.4 12 20 12 20Z" />
  {:else if name === 'close'}
    <path d="M6 6l12 12M18 6 6 18" />
  {:else if name === 'chevron-left'}
    <path d="m15 6-6 6 6 6" />
  {:else if name === 'chevron-right'}
    <path d="m9 6 6 6-6 6" />
  {:else if name === 'chevron-up'}
    <path d="m6 15 6-6 6 6" />
  {:else if name === 'book'}
    <path d="M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5z" />
    <path d="M4 20.5A2.5 2.5 0 0 1 6.5 18H20" />
  {:else if name === 'external'}
    <path d="M14 4h6v6" />
    <path d="M20 4 10 14" />
    <path d="M19 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1h5" />
  {:else if name === 'person'}
    <circle cx="12" cy="8" r="3.2" />
    <path d="M5.5 20a6.5 6.5 0 0 1 13 0" />
  {:else if name === 'globe'}
    <circle cx="12" cy="12" r="9" />
    <path d="M3 12h18M12 3c2.5 2.5 2.5 15 0 18M12 3c-2.5 2.5-2.5 15 0 18" />
  {:else if name === 'doc'}
    <path d="M7 3h7l4 4v14H7z" />
    <path d="M14 3v4h4" />
  {:else if name === 'layers'}
    <path d="m12 3 9 5-9 5-9-5 9-5Z" />
    <path d="m3 13 9 5 9-5" />
  {:else if name === 'download'}
    <path d="M12 3v12" />
    <path d="m7 11 5 4 5-4" />
    <path d="M5 20h14" />
  {:else if name === 'folder'}
    <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
  {:else if name === 'trash'}
    <path d="M4 7h16" />
    <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
    <path d="M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13" />
  {:else if name === 'grid'}
    <rect x="3.5" y="3.5" width="7" height="7" rx="1" />
    <rect x="13.5" y="3.5" width="7" height="7" rx="1" />
    <rect x="3.5" y="13.5" width="7" height="7" rx="1" />
    <rect x="13.5" y="13.5" width="7" height="7" rx="1" />
  {:else if name === 'clock'}
    <circle cx="12" cy="12" r="9" />
    <path d="M12 7.5V12l3 2" />
  {:else if name === 'play'}
    <path d="M7 4.5 19 12 7 19.5z" />
  {:else if name === 'pause'}
    <path d="M9 5v14M15 5v14" />
  {:else if name === 'book-open'}
    <path d="M12 6.5C10.5 5 7.5 4.5 4 5v13c3.5-.5 6.5 0 8 1.5 1.5-1.5 4.5-2 8-1.5V5c-3.5-.5-6.5 0-8 1.5Z" />
    <path d="M12 6.5v13" />
  {:else if name === 'columns'}
    <rect x="4" y="4" width="7" height="16" rx="1" />
    <rect x="13" y="4" width="7" height="16" rx="1" />
  {:else if name === 'scroll'}
    <rect x="5" y="3.5" width="14" height="17" rx="2" />
    <path d="M9 8h6M9 12h6M9 16h4" />
  {:else if name === 'share'}
    <circle cx="6" cy="12" r="2.5" />
    <circle cx="18" cy="6" r="2.5" />
    <circle cx="18" cy="18" r="2.5" />
    <path d="m8.2 10.8 7.6-3.6M8.2 13.2l7.6 3.6" />
  {:else if name === 'check'}
    <path d="m5 12.5 4.5 4.5L19 7" />
  {/if}
</svg>
