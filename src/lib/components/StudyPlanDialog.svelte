<script lang="ts" module>
  export interface Feedback {
    error: boolean;
    text: string;
  }
</script>

<script lang="ts">
  import { trapFocus } from "$lib/focus-trap";
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { PlanStatus } from "$lib/api";
  import { locale, t, tn } from "$lib/i18n/index.svelte";

  interface Props {
    status: PlanStatus | null;
    feedback: Feedback | null;
    busy: boolean;
    onchoose: () => void;
    ondrop: (path: string) => void;
    /** The table of the "Piano degli studi" page, copied and pasted. */
    onpaste: (text: string) => void;
    onclear: () => void;
    onclose: () => void;
  }

  let { status, feedback, busy, onchoose, ondrop, onpaste, onclear, onclose }: Props = $props();
  let dragging = $state(false);
  let pasted = $state("");
  let dropError = $state<string | null>(null);
  const DATE = $derived(new Intl.DateTimeFormat(locale(), { dateStyle: "long" }));

  // Files dragged onto the window arrive as paths from Tauri (the HTML drop event does not
  // fire in the app). They are accepted only while this dialog is open.
  onMount(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (payload.type === "enter" || payload.type === "over") dragging = true;
      if (payload.type === "leave") dragging = false;
      if (payload.type !== "drop") return;
      dragging = false;
      const pdf = payload.paths.find((p) => p.toLowerCase().endsWith(".pdf"));
      dropError = pdf ? null : t("plan.dropPdf");
      if (pdf && !busy) ondrop(pdf);
    });
    return () => void unlisten.then((stop) => stop());
  });

  // Once read, the pasted text is not kept on screen.
  $effect(() => {
    if (feedback && !feedback.error) pasted = "";
  });

  function focusClose(node: HTMLElement) {
    node.focus();
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && onclose()} />

<div class="overlay">
  <div class="card dialog" role="dialog"
    use:trapFocus aria-modal="true" aria-labelledby="plan-title">
    <header>
      <h2 id="plan-title">{t("plan.title")}</h2>
      <button class="link" onclick={onclose} disabled={busy} use:focusClose
        >{t("common.close")}</button
      >
    </header>

    <p>{t("plan.intro")}</p>

    {#if status}
      <p class="loaded">
        {tn("plan.loaded", status.teachings, {
          date: DATE.format(new Date(status.importedAt * 1000)),
          first: status.first,
          second: status.second,
        })}
        <button class="link" disabled={busy} onclick={onclear}>{t("plan.remove")}</button>
      </p>
    {/if}

    <div class="drop" class:dragging aria-busy={busy}>
      {#if busy}
        <p>{t("plan.reading")}</p>
      {:else}
        <p class="big">{t("plan.dropHere")}</p>
        <p class="muted">{t("plan.dropHint")}</p>
        <button class="primary" onclick={onchoose}>{t("plan.chooseFile")}</button>
      {/if}
    </div>

    <div class="paste">
      <label for="plan-paste">{t("plan.pasteLabel")}</label>
      <textarea
        id="plan-paste"
        bind:value={pasted}
        rows="4"
        disabled={busy}
        placeholder={t("plan.pastePlaceholder")}
      ></textarea>
      <button disabled={busy || pasted.trim() === ""} onclick={() => onpaste(pasted)}>
        {t("plan.readTable")}
      </button>
    </div>

    {#if dropError}
      <p class="error" role="alert">{dropError}</p>
    {:else if feedback}
      <p class:error={feedback.error} class:ok={!feedback.error} role="status">{feedback.text}</p>
    {/if}

    <details class="how">
      <summary>{t("plan.noReceipt")}</summary>
      <p>
        {t("plan.how.before")}
        <strong>{t("plan.how.page")}</strong>{t("plan.how.after")}
      </p>
      <ul>
        <li><strong>{t("plan.how.paste.label")}</strong> {t("plan.how.paste.text")}</li>
        <li><strong>{t("plan.how.pdf.label")}</strong> {t("plan.how.pdf.text")}</li>
      </ul>
    </details>

    <p class="note">{t("plan.note")}</p>
  </div>
</div>

<style>
  /* Overlay, entrance and shadow: .overlay / .dialog in styles.css. */
  .dialog {
    width: min(600px, 100%);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  header h2 {
    margin: 0;
    font-size: 1.15rem;
  }

  .loaded {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px 10px;
    padding: 9px 12px;
    border-radius: var(--r-md);
    background: var(--selected);
  }

  .drop {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 30px 16px;
    border: 1.5px dashed var(--border-strong);
    border-radius: var(--r-lg);
    background: var(--bg);
    text-align: center;
    transition:
      border-color var(--t-in) var(--ease),
      background-color var(--t-in) var(--ease),
      transform var(--t-panel) var(--ease);
  }

  /* A file held over the window: the zone lights up and leans in. */
  .drop.dragging {
    border-color: var(--accent);
    border-style: solid;
    background: var(--selected);
    transform: scale(1.01);
  }

  .drop button {
    margin-top: 6px;
  }

  .drop p {
    margin: 0;
  }

  .big {
    font-size: 1.1rem;
    font-weight: 600;
  }

  .paste {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 14px;
  }

  .paste label {
    font-weight: 600;
  }

  .paste button {
    align-self: flex-start;
  }

  textarea {
    font-size: 0.9em;
    padding: 9px 10px;
    resize: vertical;
  }

  .how {
    margin-top: 14px;
    font-size: 0.9em;
  }

  .how summary {
    cursor: pointer;
    color: var(--accent);
  }

  .how ul {
    margin: 4px 0 0;
    padding-left: 20px;
  }

  .how li + li {
    margin-top: 4px;
  }

  .ok {
    color: var(--ok);
  }

  .error {
    color: var(--danger);
  }

  .note {
    margin-bottom: 0;
    color: var(--muted);
    font-size: 0.85em;
  }
</style>
