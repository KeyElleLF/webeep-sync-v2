<script lang="ts">
  // An "i" that shows an explanation on hover or keyboard focus.
  import { t } from "$lib/i18n/index.svelte";

  interface Props {
    text: string;
    /** Accessible name of the icon; a generic "Information" (translated) when missing. */
    label?: string;
  }

  let { text, label }: Props = $props();
  const id = `tip-${Math.random().toString(36).slice(2, 9)}`;
</script>

<span class="wrap">
  <span
    class="icon"
    tabindex="0"
    role="button"
    aria-label={label ?? t("common.info")}
    aria-describedby={id}>i</span
  >
  <span class="tip" role="tooltip" {id}>{text}</span>
</span>

<style>
  .wrap {
    position: relative;
    display: inline-flex;
  }

  .icon {
    display: inline-grid;
    place-items: center;
    width: 18px;
    height: 18px;
    border: 1px solid var(--accent);
    border-radius: 50%;
    color: var(--accent);
    font-size: 0.72rem;
    font-weight: 700;
    cursor: help;
    transition:
      background-color var(--t-out) var(--ease),
      color var(--t-out) var(--ease);
  }

  .icon:hover,
  .icon:focus-visible {
    background: var(--accent-strong);
    border-color: var(--accent-strong);
    color: var(--on-accent);
    transition-duration: var(--t-in);
  }

  .icon:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .tip {
    position: absolute;
    top: calc(100% + 8px);
    right: -8px;
    z-index: 5;
    width: 340px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: #1a1a1a;
    color: var(--text);
    font-size: 0.85rem;
    font-weight: 400;
    line-height: 1.5;
    white-space: normal;
    box-shadow: var(--shadow-pop);
    visibility: hidden;
    opacity: 0;
    transform: translateY(-4px);
    transition:
      opacity var(--t-out) var(--ease),
      transform var(--t-out) var(--ease),
      visibility 0s linear var(--t-out);
    pointer-events: none;
  }

  /* Shown after a short pause, so passing over the "i" does not flash it. */
  .wrap:hover .tip,
  .icon:focus-visible + .tip {
    visibility: visible;
    opacity: 1;
    transform: none;
    transition:
      opacity 160ms var(--ease) 120ms,
      transform 160ms var(--ease) 120ms,
      visibility 0s linear 120ms;
  }
</style>
