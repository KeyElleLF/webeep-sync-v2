<script lang="ts" module>
  export interface Feedback {
    error: boolean;
    text: string;
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import type { PlanStatus } from "$lib/api";

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
  const DATE = new Intl.DateTimeFormat("it-IT", { dateStyle: "long" });

  // Files dragged onto the window arrive as paths from Tauri (the HTML drop event does not
  // fire in the app). They are accepted only while this dialog is open.
  onMount(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (payload.type === "enter" || payload.type === "over") dragging = true;
      if (payload.type === "leave") dragging = false;
      if (payload.type !== "drop") return;
      dragging = false;
      const pdf = payload.paths.find((p) => p.toLowerCase().endsWith(".pdf"));
      dropError = pdf ? null : "Trascina un file PDF.";
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
  <div class="card dialog" role="dialog" aria-modal="true" aria-labelledby="plan-title">
    <header>
      <h2 id="plan-title">Piano di studi</h2>
      <button class="link" onclick={onclose} disabled={busy} use:focusClose>Chiudi</button>
    </header>

    <p>
      Dal tuo piano degli studi l'app legge se ogni insegnamento è del 1° o del 2° semestre e divide
      i corsi di conseguenza.
    </p>

    {#if status}
      <p class="loaded">
        Piano caricato il {DATE.format(new Date(status.importedAt * 1000))}: {status.teachings}
        insegnamenti, {status.first} del 1° semestre e {status.second} del 2°.
        <button class="link" disabled={busy} onclick={onclear}>Rimuovi</button>
      </p>
    {/if}

    <div class="drop" class:dragging aria-busy={busy}>
      {#if busy}
        <p>Leggo il piano…</p>
      {:else}
        <p class="big">Trascina qui il PDF</p>
        <p class="muted">
          la ricevuta del piano degli studi, oppure la pagina «Piano degli studi» salvata in PDF
        </p>
        <button class="primary" onclick={onchoose}>Scegli file…</button>
      {/if}
    </div>

    <div class="paste">
      <label for="plan-paste">Oppure incolla la tabella degli insegnamenti</label>
      <textarea
        id="plan-paste"
        bind:value={pasted}
        rows="4"
        disabled={busy}
        placeholder="Seleziona la tabella nella pagina «Piano degli studi», copiala (Ctrl+C) e incollala qui (Ctrl+V)"
      ></textarea>
      <button disabled={busy || pasted.trim() === ""} onclick={() => onpaste(pasted)}>
        Leggi la tabella
      </button>
    </div>

    {#if dropError}
      <p class="error" role="alert">{dropError}</p>
    {:else if feedback}
      <p class:error={feedback.error} class:ok={!feedback.error} role="status">{feedback.text}</p>
    {/if}

    <details class="how">
      <summary>Non hai la ricevuta?</summary>
      <p>
        Dai Servizi Online del Polimi non si può più scaricare. Apri invece
        <strong>Carriera didattica → Piano degli studi</strong>, poi:
      </p>
      <ul>
        <li>
          <strong>per incollare:</strong> seleziona con il mouse la tabella degli insegnamenti
          (dall'intestazione all'ultima riga), copiala e incollala qui sopra: così l'app vede solo la
          tabella;
        </li>
        <li>
          <strong>per il PDF:</strong> Ctrl+P → «Salva come PDF». Per non includere i tuoi dati,
          seleziona prima la tabella e nella finestra di stampa scegli «Solo selezione» (in «Altre
          impostazioni»).
        </li>
      </ul>
    </details>

    <p class="note">
      Si tengono solo codici, nomi e semestri degli insegnamenti: dati personali, docenti e voti non
      vengono salvati, e il file non lascia il tuo PC. Le scelte del semestre fatte a mano vengono sostituite dal piano; puoi sempre
      cambiarle di nuovo dalla lista dei corsi.
    </p>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(0, 0, 0, 0.5);
  }

  .dialog {
    width: min(600px, 100%);
    max-height: calc(100vh - 48px);
    overflow-y: auto;
    box-sizing: border-box;
    margin: 0;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  header h2 {
    margin: 0;
  }

  .loaded {
    padding: 8px 12px;
    border-radius: 6px;
    background: var(--selected);
  }

  .drop {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 28px 16px;
    border: 2px dashed var(--border);
    border-radius: 10px;
    text-align: center;
    transition:
      border-color 0.15s,
      background 0.15s;
  }

  .drop.dragging {
    border-color: var(--accent);
    background: var(--hover);
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
    font: inherit;
    font-size: 0.9em;
    padding: 8px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg);
    color: var(--text);
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
    color: var(--accent);
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
