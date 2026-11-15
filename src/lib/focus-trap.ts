// Keeps the keyboard inside a modal dialog: Tab and Shift+Tab cycle through its controls instead
// of walking into the blurred page behind it. When the dialog closes, the focus goes back to where
// it was before it opened.

const FOCUSABLE = [
  "button:not([disabled])",
  "a[href]",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  "summary",
  '[tabindex]:not([tabindex="-1"])',
].join(",");

function focusable(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
    (el) => el.offsetParent !== null || el === document.activeElement,
  );
}

/** Svelte action for the dialog element. */
export function trapFocus(node: HTMLElement) {
  const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;

  function keydown(event: KeyboardEvent) {
    if (event.key !== "Tab") return;
    const items = focusable(node);
    if (items.length === 0) return;
    const first = items[0];
    const last = items[items.length - 1];
    const current = document.activeElement;
    if (!(current instanceof Node) || !node.contains(current)) {
      event.preventDefault();
      first.focus();
    } else if (event.shiftKey && current === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && current === last) {
      event.preventDefault();
      first.focus();
    }
  }

  document.addEventListener("keydown", keydown);
  return {
    destroy() {
      document.removeEventListener("keydown", keydown);
      if (before?.isConnected) before.focus();
    },
  };
}
