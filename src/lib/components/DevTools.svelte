<script lang="ts">
  import { api, type DumpReport } from "$lib/api";

  interface Props {
    disabled: boolean;
    run: <T>(label: string, action: () => Promise<T>) => Promise<T | undefined>;
  }

  let { disabled, run }: Props = $props();
  let report = $state<DumpReport | null>(null);

  async function dump() {
    report = (await run("Scarico i dati da WeBeep…", api.dumpPhase0)) ?? report;
  }
</script>

<details class="card">
  <summary>Strumenti di sviluppo · Fase 0</summary>
  <p class="muted">
    Salva le risposte grezze di WeBeep (corsi, filtri, struttura dei corsi in corso) in
    <code>fixtures/raw/</code>. Contengono dati personali: non vanno condivise.
  </p>
  <button onclick={dump} {disabled}>Esegui dump</button>

  {#if report}
    <div class="report">
      <p>Cartella: <code>{report.directory}</code></p>
      <p>File salvati ({report.files.length}): {report.files.join(", ")}</p>
      {#if report.errors.length > 0}
        <p class="error">Errori ({report.errors.length}):</p>
        <ul>
          {#each report.errors as message}
            <li>{message}</li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</details>

<style>
  .report {
    margin-top: 12px;
  }

  .error {
    color: var(--danger);
  }
</style>
