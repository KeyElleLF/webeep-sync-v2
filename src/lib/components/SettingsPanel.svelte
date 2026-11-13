<script lang="ts">
  import type { Settings } from "$lib/api";

  interface Props {
    settings: Settings;
    disabled: boolean;
    onchange: (settings: Settings) => void;
  }

  let { settings, disabled, onchange }: Props = $props();
  const CONCURRENCY_OPTIONS = [1, 2, 3, 4, 5, 6, 7, 8];
  const INTERVALS = [
    { minutes: 30, label: "ogni 30 minuti" },
    { minutes: 60, label: "ogni ora" },
    { minutes: 120, label: "ogni 2 ore (consigliato)" },
    { minutes: 240, label: "ogni 4 ore" },
    { minutes: 480, label: "ogni 8 ore" },
    { minutes: 1440, label: "una volta al giorno" },
  ];
</script>

<h3>Mentre l'app è aperta</h3>
<label class="row">
  <input
    type="checkbox"
    checked={settings.autosync}
    {disabled}
    onchange={(e) => onchange({ ...settings, autosync: e.currentTarget.checked })}
  />
  <span>Sincronizza automaticamente</span>
  <select
    value={settings.autosyncMinutes}
    disabled={disabled || !settings.autosync}
    aria-label="Intervallo"
    onchange={(e) => onchange({ ...settings, autosyncMinutes: Number(e.currentTarget.value) })}
  >
    {#each INTERVALS as i (i.minutes)}
      <option value={i.minutes}>{i.label}</option>
    {/each}
  </select>
</label>
<label class="row">
  <input
    type="checkbox"
    checked={settings.notifications}
    {disabled}
    onchange={(e) => onchange({ ...settings, notifications: e.currentTarget.checked })}
  />
  <span>Notifiche di sistema (nuovi file, avvisi, scadenze tra 24 ore)</span>
</label>
<p class="muted">
  L'app lavora solo mentre è aperta: all'apertura controlla subito gli avvisi e, se l'ultimo
  sync è più vecchio dell'intervallo scelto, sincronizza. Chiudendo la finestra si chiude tutto.
</p>

<h3>Download</h3>

<label class="row">
  <span>File scaricati contemporaneamente</span>
  <select
    value={settings.downloadConcurrency}
    {disabled}
    onchange={(e) => onchange({ ...settings, downloadConcurrency: Number(e.currentTarget.value) })}
  >
    {#each CONCURRENCY_OPTIONS as n}
      <option value={n}>{n}{n === 4 ? " (consigliato)" : ""}</option>
    {/each}
  </select>
</label>
<p class="muted">Valori alti non velocizzano molto e caricano di più WeBeep.</p>

<label class="row">
  <input
    type="checkbox"
    checked={settings.flattenMaterials}
    {disabled}
    onchange={(e) => onchange({ ...settings, flattenMaterials: e.currentTarget.checked })}
  />
  <span>Non creare la cartella «Materiali» (come WeBeep Sync v1)</span>
</label>
<p class="muted">
  Se la cambi, al prossimo sync i file già scaricati vengono spostati nella nuova posizione
  (quelli che hai modificato restano dove sono e ne viene scaricata una copia nuova).
</p>

<h3>Corsi superati</h3>
<label class="row">
  <span>Quando segno un corso come superato</span>
  <select
    value={settings.cleanupPolicy}
    {disabled}
    onchange={(e) =>
      onchange({
        ...settings,
        cleanupPolicy: e.currentTarget.value as Settings["cleanupPolicy"],
      })}
  >
    <option value="ask">chiedimi se eliminare i file</option>
    <option value="always">sposta subito i file nel Cestino</option>
    <option value="never">non eliminare mai i file</option>
  </select>
</label>
<p class="muted">
  Vengono spostati nel Cestino solo i file scaricati dall'app e non modificati da te.
</p>

<style>
  h3 {
    font-size: 1rem;
    margin: 18px 0 0;
  }

  h3:first-child {
    margin-top: 0;
  }

  .row {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-top: 12px;
  }

  select {
    font: inherit;
    padding: 4px 8px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--card);
    color: var(--text);
  }

  .muted {
    color: var(--muted);
    margin: 4px 0 0;
    font-size: 0.9em;
  }
</style>
