<script lang="ts">
  import type { Deadline } from "$lib/api";
  import { locale, t, type MessageKey } from "$lib/i18n/index.svelte";
  import { openExternal } from "$lib/links";

  interface Props {
    deadlines: Deadline[] | null;
  }

  type Group = "today" | "tomorrow" | "week" | "later";

  let { deadlines }: Props = $props();
  const TIME = $derived(
    new Intl.DateTimeFormat(locale(), {
      weekday: "short",
      day: "numeric",
      month: "short",
      hour: "2-digit",
      minute: "2-digit",
    }),
  );
  const KINDS: Record<string, MessageKey> = {
    assign: "deadlines.kind.assign",
    quiz: "deadlines.kind.quiz",
    forum: "deadlines.kind.forum",
    workshop: "deadlines.kind.workshop",
    lesson: "deadlines.kind.lesson",
  };
  /** Groups in display order, with their titles. */
  const GROUP_TITLES: Record<Group, MessageKey> = {
    today: "deadlines.group.today",
    tomorrow: "deadlines.group.tomorrow",
    week: "deadlines.group.week",
    later: "deadlines.group.later",
  };
  const DAY = 86_400_000;

  function group(due: number): Group {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const at = due * 1000;
    if (at < start + DAY) return "today";
    if (at < start + 2 * DAY) return "tomorrow";
    if (at < start + 7 * DAY) return "week";
    return "later";
  }

  /** Activity types WeBeep may add later are shown as they come. */
  function kindLabel(kind: string): string {
    const key = KINDS[kind];
    return key ? t(key) : kind;
  }

  const groups = $derived(
    (Object.keys(GROUP_TITLES) as Group[])
      .map((id) => ({ id, items: (deadlines ?? []).filter((d) => group(d.due) === id) }))
      .filter((g) => g.items.length > 0),
  );
</script>

<p class="hint">{t("deadlines.hint")}</p>

{#if deadlines === null}
  <p class="muted">{t("deadlines.notLoaded")}</p>
{:else if deadlines.length === 0}
  <p class="muted">{t("deadlines.empty")}</p>
{:else}
  <div class="table">
    <div class="row head" aria-hidden="true">
      <span>{t("deadlines.col.due")}</span>
      <span>{t("deadlines.col.activity")}</span>
      <span>{t("deadlines.col.kind")}</span>
      <span>{t("common.course")}</span>
      <span></span>
    </div>
    {#each groups as g (g.id)}
      <h3>{t(GROUP_TITLES[g.id])}</h3>
      <ul>
        {#each g.items as d (d.id)}
          <li class="row" class:overdue={d.overdue}>
            <span class="when">{TIME.format(new Date(d.due * 1000))}</span>
            <span class="name ellipsis" title={d.name}>{d.name}</span>
            <span class="meta">{kindLabel(d.kind)}</span>
            <span class="meta ellipsis" title={d.courseName}>{d.courseName}</span>
            <span class="open">
              {#if d.url}
                <button class="link" onclick={() => openExternal(d.url)}
                  >{t("common.openOnWebeep")}</button
                >
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
