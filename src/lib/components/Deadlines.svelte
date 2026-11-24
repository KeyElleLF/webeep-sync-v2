<script lang="ts">
  import type { Deadline } from "$lib/api";
  import { locale, t, type MessageKey } from "$lib/i18n/index.svelte";
  import { openExternal } from "$lib/links";
  import { courseTitle } from "$lib/course-title";
  import Icon from "./Icon.svelte";

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
  const HOUR = 3_600_000;
  const RELATIVE = $derived(new Intl.RelativeTimeFormat(locale(), { numeric: "auto" }));

  /** "tra 5 ore", "domani", "3 ore fa": how close it is, which a date alone does not say. */
  function relative(due: number): string {
    const diff = due * 1000 - Date.now();
    if (Math.abs(diff) < 2 * DAY) return RELATIVE.format(Math.round(diff / HOUR), "hour");
    return RELATIVE.format(Math.round(diff / DAY), "day");
  }

  function group(due: number): Group {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const at = due * 1000;
    if (at < start + DAY) return "today";
    if (at < start + 2 * DAY) return "tomorrow";
    if (at < start + 7 * DAY) return "week";
    return "later";
  }

  /** Due within a day and not yet passed: its time is shown in the warning colour. */
  function isSoon(d: Deadline): boolean {
    return !d.overdue && d.due * 1000 - Date.now() < DAY;
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

{#if deadlines === null}
  <p class="empty"><Icon name="deadlines" size={18} />{t("deadlines.notLoaded")}</p>
{:else if deadlines.length === 0}
  <p class="empty"><Icon name="check" size={18} />{t("deadlines.empty")}</p>
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
      <h2>{t(GROUP_TITLES[g.id])} <span class="count">{g.items.length}</span></h2>
      <ul>
        {#each g.items as d (d.id)}
          <li class="row" class:overdue={d.overdue} class:soon={isSoon(d)}>
            <span class="when">
              <span>{TIME.format(new Date(d.due * 1000))}</span>
              <span class="relative">{relative(d.due)}</span>
            </span>
            <span class="activity">
              {#if d.url}
                <!-- The whole row opens the activity on WeBeep (the button stretches over it). -->
                <button class="name open-row ellipsis" title={d.name} onclick={() => openExternal(d.url)}
                  >{d.name}<span class="sr-only"> — {t("common.openOnWebeep")}</span></button
                >
              {:else}
                <span class="name ellipsis" title={d.name}>{d.name}</span>
              {/if}
              {#if d.overdue}<span class="overdue-tag">{t("deadlines.overdue")}</span>{/if}
            </span>
            <span class="meta">{kindLabel(d.kind)}</span>
            <span class="meta ellipsis" title={courseTitle(d.courseName)}>{courseTitle(d.courseName)}</span>
            <span class="open" aria-hidden="true">
              {#if d.url}<Icon name="external" size={14} />{/if}
            </span>
          </li>
        {/each}
      </ul>
    {/each}
  </div>
{/if}

<style>
  .empty {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
  }

  .table {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    overflow: clip;
  }

  .row {
    display: grid;
    grid-template-columns: 170px minmax(0, 3fr) 90px minmax(0, 2fr) 24px;
    gap: 12px;
    align-items: center;
    padding: 9px 12px;
  }

  .head {
    position: sticky;
    top: -20px;
    z-index: 1;
    padding-block: 9px;
    color: var(--muted);
    font-size: var(--fs-sm);
    font-weight: 600;
    background: var(--card);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }

  h2 {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin: 0;
    padding: 12px 12px 7px;
    font-size: var(--fs-sm);
    font-weight: 600;
    background: rgba(0, 0, 0, 0.14);
    border-bottom: 1px solid var(--border);
    user-select: none;
  }

  .count {
    display: inline-block;
    min-width: 18px;
    padding: 0 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.07);
    color: var(--muted);
    font-size: var(--fs-sm);
    font-weight: 600;
    line-height: 18px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  ul + h2 {
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

  li {
    position: relative;
    transition: background-color var(--t-out) var(--ease);
  }

  li:has(.open-row) {
    cursor: pointer;
  }

  .open-row {
    justify-content: flex-start;
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
    color: inherit;
    font: inherit;
    font-weight: 600;
    text-align: left;
  }

  .open-row::after {
    content: "";
    position: absolute;
    inset: 0;
  }

  .open-row:focus-visible {
    outline: none;
  }

  li:has(.open-row:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  li:hover .open-row {
    color: var(--accent);
  }

  li:hover {
    background: rgba(255, 255, 255, 0.025);
    transition-duration: var(--t-in);
  }

  .when {
    display: flex;
    flex-direction: column;
    font-variant-numeric: tabular-nums;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .relative {
    color: var(--text);
    font-size: var(--fs-sm);
  }

  .name {
    font-weight: 600;
  }

  .meta {
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .activity {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  /* Urgency is the colour of the time only: amber within a day, red once passed, with the word
     «Scaduta» so it never relies on colour alone. */
  .soon .relative {
    color: var(--warn);
  }

  .overdue .when,
  .overdue .relative,
  .overdue-tag {
    color: var(--danger);
  }

  .overdue-tag {
    flex: none;
    font-size: var(--fs-sm);
    font-weight: 600;
  }

  .open {
    display: flex;
    justify-content: flex-end;
    color: var(--muted);
    transition:
      transform var(--t-out) var(--ease),
      color var(--t-out) var(--ease);
  }

  li:hover .open {
    color: var(--accent);
    transform: translate(2px, -2px);
    transition-duration: var(--t-in);
  }
</style>
