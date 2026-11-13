<script lang="ts">
  // An "i" that shows an explanation on hover or keyboard focus.
  interface Props {
    text: string;
    label?: string;
  }

  let { text, label = "Informazioni" }: Props = $props();
  const id = `tip-${Math.random().toString(36).slice(2, 9)}`;
</script>

<span class="wrap">
  <span class="icon" tabindex="0" role="button" aria-label={label} aria-describedby={id}>i</span>
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
    font-size: 0.75rem;
    font-weight: 700;
    font-style: italic;
    font-family: Georgia, serif;
    cursor: help;
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
    background: var(--sidebar);
    color: var(--text);
    font-size: 0.85rem;
    font-weight: 400;
    line-height: 1.45;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    visibility: hidden;
    opacity: 0;
    transition: opacity 0.12s;
    pointer-events: none;
  }

  .wrap:hover .tip,
  .icon:focus-visible + .tip {
    visibility: visible;
    opacity: 1;
  }
</style>
