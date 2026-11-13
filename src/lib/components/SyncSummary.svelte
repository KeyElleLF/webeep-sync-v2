<script lang="ts">
  import type { CourseSummary } from "$lib/api";
  import { t, tn } from "$lib/i18n/index.svelte";

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
    if (s.error) return t("common.notSynced");
    const r = s.report;
    const parts = [
      r.downloaded.length && `${r.downloaded.length} ${tn("sync.new", r.downloaded.length)}`,
      r.updated.length && `${r.updated.length} ${tn("sync.updated", r.updated.length)}`,
      r.adopted && `${r.adopted} ${tn("sync.adopted", r.adopted)}`,
      r.conflicts.length && `${r.conflicts.length} ${tn("sync.conflicts", r.conflicts.length)}`,
      r.errors.length && `${r.errors.length} ${tn("sync.errors", r.errors.length)}`,
    ].filter(Boolean);
    return parts.length > 0 ? parts.join(" · ") : t("sync.noChanges");
  }
</script>

<p>
  <strong>{totals.downloaded}</strong> {tn("sync.newFiles", totals.downloaded)},
  <strong>{totals.updated}</strong> {tn("sync.updated", totals.updated)},
  <strong>{totals.adopted}</strong> {tn("sync.adopted", totals.adopted)},
  <strong class:warn={totals.conflicts > 0}>{totals.conflicts}</strong>
  {tn("sync.conflicts", totals.conflicts)},
  <strong class:bad={totals.errors > 0}>{totals.errors}</strong>
  {tn("sync.errors", totals.errors)}.
</p>

{#each summaries as s (s.id)}
  <details class="course" class:failed={s.error}>
    <summary>
      <span class="name">{s.name}</span>
      <span class="meta">{headline(s)}</span>
    </summary>
    <p class="meta">{t("sync.folder", { folder: s.folder })}</p>
    {#if s.error}
      <p class="bad">{s.error}</p>
    {/if}
    {@render list(t("sync.list.new"), s.report.downloaded)}
    {@render list(t("sync.list.updated"), s.report.updated)}
    {@render list(t("sync.list.conflicts"), s.report.conflicts)}
    {@render list(t("sync.list.keptLocal"), s.report.keptLocal)}
    {@render list(t("sync.list.removed"), s.report.removedRemotely)}
    {@render list(
      t("sync.list.errors"),
      s.report.errors.map((e) => `${e.path}: ${e.message}`),
    )}
    {@render list(
      t("sync.list.external"),
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
