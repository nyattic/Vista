import type { Action } from 'svelte/action';

const SELECTOR =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

// Keep keyboard focus inside a modal dialog: focus the container on open, cycle
// Tab/Shift+Tab within the dialog's focusable elements, and restore focus to the
// previously focused element on close. The node must be focusable (tabindex="-1").
export const focusTrap: Action<HTMLElement> = (node) => {
  const previous = document.activeElement as HTMLElement | null;

  const focusables = () =>
    Array.from(node.querySelectorAll<HTMLElement>(SELECTOR)).filter((el) => el.offsetParent !== null);

  function onKeydown(e: KeyboardEvent) {
    if (e.key !== 'Tab') return;
    const items = focusables();
    if (items.length === 0) {
      e.preventDefault();
      node.focus();
      return;
    }
    const idx = document.activeElement ? items.indexOf(document.activeElement as HTMLElement) : -1;
    if (e.shiftKey) {
      if (idx <= 0) {
        e.preventDefault();
        items[items.length - 1].focus();
      }
    } else if (idx === -1 || idx === items.length - 1) {
      e.preventDefault();
      items[0].focus();
    }
  }

  node.addEventListener('keydown', onKeydown);
  node.focus();

  return {
    destroy() {
      node.removeEventListener('keydown', onKeydown);
      previous?.focus?.();
    }
  };
};
