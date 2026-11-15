<script lang="ts">
  // Result of the last sync as one compact strip: the non-zero totals, and per-course details
  // (file names, conflicts, files the teacher removed, errors) behind "Details". Shown only when the
  // sync brought something: the course rows already mark new files and errors.
  import type { CourseSummary } from "$lib/api";
  import { t, tn } from "$lib/i18n/index.svelte";

  interface Props {
    summaries: CourseSummary[];
    onclose: () => void;
    onfolder: (courseId: number) => void;
  }

  let { summaries, onclose, onfolder }: Props = $props();
  let open = $state(false);

  function sum(pick: (s: CourseSummary) => number): number {
    return summaries.reduce((total, s) => total + pick(s), 0);
  }

  /** Something to tell about this course. Its links are news only when the teacher just added them:
   *  the others (the recordings archive of every course, for one) live in the Link tab. */
  function hasNews(s: CourseSummary): boolean {
    const r = s.report;
    return (
      !!s.error ||
      r.downloaded.length + r.updated.length + r.adopted + r.conflicts.length > 0 ||
      r.keptLocal.length + r.removedRemotely.length + r.errors.length + s.newLinks.length > 0
    );
  }

  const newsworthy = $derived(summaries.filter(hasNews));
  const totals = $derived({
    downloaded: sum((s) => s.report.downloaded.length),
    updated: sum((s) => s.report.updated.length),
    adopted: sum((s) => s.report.adopted),
    conflicts: sum((s) => s.report.conflicts.length),
    errors: sum((s) => s.report.errors.length + (s.error ? 1 : 0)),
  });
  const alert = $derived(totals.conflicts > 0 || totals.errors > 0);

  function headline(s: CourseSummary): string {
    if (s.error) return t("common.notSynced");
    const r = s.report;
    const parts = [
      r.downloaded.length && `${r.downloaded.length} ${tn("sync.new", r.downloaded.length)}`,
      r.updated.length && `${r.updated.length} ${tn("sync.updated", r.updated.length)}`,
      r.adopted && `${r.adopted} ${tn("sync.adopted", r.adopted)}`,
      r.conflicts.length && `${r.conflicts.length} ${tn("sync.conflicts", r.conflicts.length)}`,
      r.errors.length && `${r.errors.length} ${tn("sync.errors", r.errors.length)}`,
      r.removedRemotely.length &&
        `${r.removedRemotely.length} ${tn("sync.removedCount", r.removedRemotely.length)}`,
      s.newLinks.length && `${s.newLinks.length} ${tn("sync.newLinksCount", s.newLinks.length)}`,
    ].filter(Boolean);
    return parts.length > 0 ? parts.join(" · ") : t("sync.noChanges");
  }
</script>

{#if newsworthy.length > 0}
  <section class="strip" class:alert aria-label={t("sync.lastSync")}>
    <details bind:open>
      <summary>
        <span class="label">{t("sync.lastSync")}</span>
        <span class="stats">
          {#if totals.downloaded > 0}
            <span class="stat"
              ><strong>{totals.downloaded}</strong> {tn("sync.newFiles", totals.downloaded)}</span
            >
          {/if}
          {#if totals.updated > 0}
            <span class="stat"
              ><strong>{totals.updated}</strong> {tn("sync.updated", totals.updated)}</span
            >
          {/if}
          {#if totals.adopted > 0}
            <span class="stat"
              ><strong>{totals.adopted}</strong> {tn("sync.adopted", totals.adopted)}</span
            >
          {/if}
          {#if totals.conflicts > 0}
            <span class="stat warn"
              ><strong>{totals.conflicts}</strong> {tn("sync.conflicts", totals.conflicts)}</span
            >
          {/if}
          {#if totals.errors > 0}
            <span class="stat bad"
              ><strong>{totals.errors}</strong> {tn("sync.errors", totals.errors)}</span
            >
          {/if}
        </span>
        <span class="toggle">{open ? t("sync.hideDetails") : t("sync.details")}</span>
      </summary>

      <div class="courses">
        {#each newsworthy as s (s.id)}
          <details class="course" class:failed={s.error}>
            <summary>
              <span class="name course-name">{s.name}</span>
              <span class="meta">{headline(s)}</span>
            </summary>
            <div class="folder-line">
              <p class="meta">{t("sync.folder", { folder: s.folder })}</p>
              <button class="small" onclick={() => onfolder(s.id)}>{t("links.openFolder")}</button>
            </div>
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
              t("sync.list.newLinks"),
              s.newLinks.map((l) => l.name),
            )}
          </details>
        {/each}
      </div>
    </details>
    <button class="link close" onclick={onclose}>{t("common.close")}</button>
  </section>
{/if}

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
  .strip {
    position: relative;
    margin-bottom: 16px;
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    background: var(--card);
    animation: rise-in 220ms var(--ease-enter);
  }

  .strip.alert {
    border-color: color-mix(in srgb, var(--warn) 40%, var(--border));
  }

  .strip > details > summary {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 44px;
    padding: 6px 90px 6px 14px;
    box-sizing: border-box;
  }

  .label {
    flex: none;
    font-weight: 600;
  }

  .stats {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    min-width: 0;
  }

  .stat {
    padding: 2px 9px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.05);
    color: var(--muted);
    font-size: 0.88em;
  }

  .stat strong {
    color: var(--text);
    font-variant-numeric: tabular-nums;
  }

  .stat.warn {
    background: color-mix(in srgb, var(--warn) 12%, transparent);
  }

  .stat.warn strong {
    color: var(--warn);
  }

  .stat.bad {
    background: color-mix(in srgb, var(--danger) 12%, transparent);
  }

  .stat.bad strong {
    color: var(--danger);
  }

  .toggle {
    margin-left: auto;
    color: var(--accent);
    font-size: 0.9em;
    font-weight: 500;
  }

  /* "Close" sits over the strip's corner, outside the summary that opens the details. */
  .close {
    position: absolute;
    top: 13px;
    right: 14px;
  }

  .courses {
    padding: 0 14px 12px;
    animation: rise-in 200ms var(--ease-enter);
  }

  .course {
    border: 1px solid var(--border);
    border-radius: var(--r-md);
    padding: 8px 12px;
    margin-top: 6px;
    transition: border-color var(--t-out) var(--ease);
  }

  .course:hover,
  .course[open] {
    border-color: var(--border-strong);
  }

  .course summary {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }

  .course summary .meta {
    margin-left: auto;
    font-size: 0.88em;
    text-align: right;
  }

  .name {
    font-weight: 600;
  }

  .folder-line {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 8px;
  }

  .folder-line p {
    margin: 0;
  }

  .folder-line button {
    flex: none;
    padding: 3px 10px;
    font-size: 0.85em;
  }

  .meta {
    color: var(--muted);
  }

  .title {
    margin: 12px 0 4px;
    font-size: 0.9em;
    font-weight: 600;
  }

  ul {
    margin: 0;
    padding-left: 20px;
    color: var(--muted);
    font-size: 0.9em;
    overflow-wrap: anywhere;
  }

  .bad,
  .failed .meta {
    color: var(--danger);
  }

  .failed {
    border-color: color-mix(in srgb, var(--danger) 35%, transparent);
  }
</style>
