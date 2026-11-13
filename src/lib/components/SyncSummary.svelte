<script lang="ts">
  import type { CourseSummary } from "$lib/api";

  interface Props {
    summaries: CourseSummary[];
  }

  let { summaries }: Props = $props();

  function sum(pick: (s: CourseSummary) => number): number {
    return summaries.reduce((total, s) => total + pick(s), 0);
  }

  const totals = $derived({
    downloaded: sum((s) => s.report.downloaded.length),
    updated: sum((s) => s.report.updated.length),
    adopted: sum((s) => s.report.adopted),
    conflicts: sum((s) => s.report.conflicts.length),
    errors: sum((s) => s.report.errors.length + (s.error ? 1 : 0)),
  });

  function headline(s: CourseSummary): string {
    if (s.error) return "non sincronizzato";
    const r = s.report;
    const parts = [
      r.downloaded.length && `${r.downloaded.length} nuovi`,
      r.updated.length && `${r.updated.length} aggiornati`,
      r.adopted && `${r.adopted} già presenti`,
      r.conflicts.length && `${r.conflicts.length} conflitti`,
      r.errors.length && `${r.errors.length} errori`,
    ].filter(Boolean);
    return parts.length > 0 ? parts.join(" · ") : "nessuna novità";
  }
</script>

<p>
  <strong>{totals.downloaded}</strong> file nuovi, <strong>{totals.updated}</strong> aggiornati,
  <strong>{totals.adopted}</strong> già presenti,
  <strong class:warn={totals.conflicts > 0}>{totals.conflicts}</strong> conflitti,
  <strong class:bad={totals.errors > 0}>{totals.errors}</strong> errori.
</p>

{#each summaries as s (s.id)}
  <details class="course" class:failed={s.error}>
    <summary>
      <span class="name">{s.name}</span>
      <span class="meta">{headline(s)}</span>
    </summary>
    <p class="meta">Cartella: {s.folder}</p>
    {#if s.error}
      <p class="bad">{s.error}</p>
    {/if}
    {@render list("Nuovi", s.report.downloaded)}
    {@render list("Aggiornati", s.report.updated)}
    {@render list(
      "Conflitti: la versione nuova è stata salvata accanto alla tua, che non è stata toccata",
      s.report.conflicts,
    )}
    {@render list("Modificati da te (non toccati)", s.report.keptLocal)}
    {@render list("Rimossi da WeBeep (la tua copia resta)", s.report.removedRemotely)}
    {@render list(
      "Errori",
      s.report.errors.map((e) => `${e.path}: ${e.message}`),
    )}
    {@render list(
      "Contenuti esterni (da aprire su WeBeep)",
      s.externalLinks.map((l) => `${l.name} — ${l.url}`),
    )}
  </details>
{/each}

{#snippet list(title: string, items: string[])}
  {#if items.length > 0}
    <p class="title">{title} ({items.length})</p>
    <ul>
      {#each items as item}
        <li>{item}</li>
      {/each}
    </ul>
  {/if}
{/snippet}

<style>
  .course {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 12px;
    margin-top: 8px;
  }

  summary {
    cursor: pointer;
  }

  summary .meta {
    margin-left: 8px;
  }

  .name {
    font-weight: 600;
  }

  .meta {
    color: var(--muted);
  }

  .title {
    margin: 10px 0 4px;
    font-weight: 600;
  }

  ul {
    margin: 0;
    padding-left: 20px;
    overflow-wrap: anywhere;
  }

  .warn {
    color: var(--warn);
  }

  .bad,
  .failed .meta {
    color: var(--danger);
  }
</style>
