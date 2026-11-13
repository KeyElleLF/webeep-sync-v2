<script lang="ts">
  import type { Deadline } from "$lib/api";
  import { openExternal } from "$lib/links";

  interface Props {
    deadlines: Deadline[] | null;
  }

  let { deadlines }: Props = $props();
  const TIME = new Intl.DateTimeFormat("it-IT", {
    weekday: "short",
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
  const KINDS: Record<string, string> = {
    assign: "Consegna",
    quiz: "Quiz",
    forum: "Forum",
    workshop: "Workshop",
    lesson: "Lezione",
  };
  const DAY = 86_400_000;

  function group(due: number): string {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const at = due * 1000;
    if (at < start + DAY) return "Oggi";
    if (at < start + 2 * DAY) return "Domani";
    if (at < start + 7 * DAY) return "Questa settimana";
    return "Più avanti";
  }

  const groups = $derived(
    ["Oggi", "Domani", "Questa settimana", "Più avanti"]
      .map((title) => ({ title, items: (deadlines ?? []).filter((d) => group(d.due) === title) }))
      .filter((g) => g.items.length > 0),
  );
</script>

<p class="hint">Consegne e quiz dei prossimi 14 giorni. Ricevi un promemoria 24 ore prima.</p>

{#if deadlines === null}
  <p class="muted">Scadenze non ancora caricate.</p>
{:else if deadlines.length === 0}
  <p class="muted">Nessuna scadenza nei prossimi 14 giorni.</p>
{:else}
  <div class="table">
    <div class="row head" aria-hidden="true">
      <span>Scadenza</span>
      <span>Attività</span>
      <span>Tipo</span>
      <span>Corso</span>
      <span></span>
    </div>
    {#each groups as g (g.title)}
      <h3>{g.title}</h3>
      <ul>
        {#each g.items as d (d.id)}
          <li class="row" class:overdue={d.overdue}>
            <span class="when">{TIME.format(new Date(d.due * 1000))}</span>
            <span class="name ellipsis" title={d.name}>{d.name}</span>
            <span class="meta">{KINDS[d.kind] ?? d.kind}</span>
            <span class="meta ellipsis" title={d.courseName}>{d.courseName}</span>
            <span class="open">
              {#if d.url}
                <button class="link" onclick={() => openExternal(d.url)}>Apri su WeBeep</button>
              {/if}
            </span>
          </li>
        {/each}
      </ul>
    {/each}
  </div>
{/if}

<style>
  .hint {
    margin: 0 0 16px;
    color: var(--muted);
    font-size: 0.9em;
  }

  .table {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .row {
    display: grid;
    grid-template-columns: 150px minmax(0, 3fr) 90px minmax(0, 2fr) 120px;
    gap: 12px;
    align-items: center;
    padding: 7px 12px;
  }

  .head {
    color: var(--muted);
    font-size: 0.85em;
    font-weight: 600;
    border-bottom: 1px solid var(--border);
  }

  h3 {
    margin: 0;
    padding: 10px 12px 6px;
    font-size: 0.95rem;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
  }

  ul + h3 {
    border-top: 1px solid var(--border);
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  li + li {
    border-top: 1px solid var(--border);
  }

  li:hover {
    background: var(--hover);
  }

  .when {
    font-variant-numeric: tabular-nums;
  }

  .name {
    font-weight: 600;
  }

  .meta {
    color: var(--muted);
    font-size: 0.9em;
  }

  .overdue .when,
  .overdue .name {
    color: var(--danger);
  }

  .open {
    text-align: right;
  }
</style>
