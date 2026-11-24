<script lang="ts">
  // The Corsi tab: the courses in groups on the left, the chosen one in detail on the right.
  import type { CourseLink, CourseRow, ManualState } from "$lib/api";
  import { SECTIONS, type RowStatus } from "$lib/format";
  import { t } from "$lib/i18n/index.svelte";
  import CourseDetail from "./CourseDetail.svelte";
  import CourseItem from "./CourseItem.svelte";
  import Icon from "./Icon.svelte";
  import { view } from "$lib/view-state.svelte";

  interface Props {
    courses: CourseRow[];
    /** Links of every course (the detail shows those of the chosen one). */
    links: CourseLink[];
    disabled: boolean;
    ontoggle: (course: CourseRow, selected: boolean) => void;
    onrename: (course: CourseRow, folder: string) => void;
    onstate: (course: CourseRow, state: ManualState | null) => void;
    onsemester: (course: CourseRow, semester: 1 | 2 | null) => void;
    /** Outcome of the last sync per course id. */
    results: Map<number, RowStatus>;
    /** Name of the course being synced right now. */
    syncingName: string | null;
    onfolder: (courseId: number) => void;
    onopenlink: (link: CourseLink) => void;
  }

  let {
    courses,
    links,
    disabled,
    ontoggle,
    onrename,
    onstate,
    onsemester,
    results,
    syncingName,
    onfolder,
    onopenlink,
  }: Props = $props();

  let list = $state<HTMLElement>();

  const groups = $derived(
    SECTIONS.map((section) => ({
      ...section,
      courses: courses
        .filter((c) => section.states.includes(c.state))
        .sort(
          (a, b) =>
            b.academicYear.localeCompare(a.academicYear) || a.name.localeCompare(b.name),
        ),
    })).filter((g) => g.courses.length > 0),
  );
  const main = $derived(groups.filter((g) => !g.extra));
  /** Passed, ignored and past courses: behind one closed entry at the bottom. */
  const extra = $derived(groups.filter((g) => g.extra));
  const extraCount = $derived(extra.reduce((n, g) => n + g.courses.length, 0));
  /** The courses in the order they are listed, for the arrow keys. */
  const shown = $derived(
    [...main, ...(view.showAllCourses ? extra : [])].flatMap((g) => g.courses),
  );
  const chosen = $derived(
    courses.find((c) => c.id === view.courseSelected) ?? shown[0] ?? null,
  );

  function select(course: CourseRow) {
    view.courseSelected = course.id;
  }

  function move(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const index = shown.findIndex((c) => c.id === chosen?.id);
    const step = event.key === "ArrowDown" ? 1 : -1;
    const next = shown[Math.min(Math.max(index + step, 0), shown.length - 1)];
    if (!next) return;
    select(next);
    list?.querySelector<HTMLButtonElement>(`[data-id="${next.id}"]`)?.focus();
  }

  function siblings(course: CourseRow): number {
    return courses.filter((c) => c.teachingKey === course.teachingKey && c.id !== course.id)
      .length;
  }
</script>

{#snippet group(g: (typeof groups)[number])}
  <h2>{t(g.title)} <span class="count">{g.courses.length}</span></h2>
  <ul>
    {#each g.courses as course (course.id)}
      <CourseItem
        {course}
        current={course.id === chosen?.id}
        {disabled}
        status={results.get(course.id) ?? null}
        syncing={course.selected && course.name === syncingName}
        {ontoggle}
        onselect={select}
        onmove={move}
      />
    {/each}
  </ul>
{/snippet}

<div class="panes">
  <section class="list" aria-label={t("courses.listLabel")} bind:this={list}>
    {#each main as g (g.title)}
      {@render group(g)}
    {/each}
    {#if extraCount > 0}
      <button
        class="more"
        aria-expanded={view.showAllCourses}
        onclick={() => (view.showAllCourses = !view.showAllCourses)}
      >
        <Icon name="chevron" size={14} class="chevron" />
        {t("courses.section.others")}
        <span class="count">{extraCount}</span>
      </button>
      {#if view.showAllCourses}
        {#each extra as g (g.title)}
          {@render group(g)}
        {/each}
      {/if}
    {/if}
  </section>

  <div class="detail-pane">
    {#if chosen}
      {#key chosen.id}
        <CourseDetail
          course={chosen}
          siblings={siblings(chosen)}
          {disabled}
          status={results.get(chosen.id) ?? null}
          syncing={chosen.selected && chosen.name === syncingName}
          links={links.filter((l) => l.courseId === chosen.id)}
          {ontoggle}
          {onrename}
          {onstate}
          {onsemester}
          {onfolder}
          {onopenlink}
        />
      {/key}
    {:else}
      <p class="empty">{t("courses.noneSelected")}</p>
    {/if}
  </div>
</div>

<style>
  .panes {
    flex: 1;
    display: grid;
    grid-template-columns: 340px minmax(0, 1fr);
    min-height: 0;
  }

  @media (max-width: 1050px) {
    .panes {
      grid-template-columns: 290px minmax(0, 1fr);
    }
  }

  .list {
    min-height: 0;
    overflow-y: auto;
    padding: 4px 0 16px;
    border-right: 1px solid var(--border);
  }

  h2 {
    display: flex;
    align-items: baseline;
    gap: 6px;
    margin: 0;
    padding: 14px 20px 6px;
    color: var(--muted);
    font-size: var(--fs-sm);
    font-weight: 600;
  }

  .count {
    font-weight: 400;
    font-variant-numeric: tabular-nums;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .more {
    justify-content: flex-start;
    gap: 8px;
    width: calc(100% - 16px);
    min-height: 36px;
    margin: 10px 8px 0;
    padding: 0 12px;
    border: none;
    border-radius: var(--r-md);
    background: none;
    color: var(--muted);
    font-size: var(--fs-sm);
    font-weight: 600;
  }

  .more:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .more :global(.chevron) {
    transition: transform var(--t-panel) var(--ease);
  }

  .more[aria-expanded="true"] :global(.chevron) {
    transform: rotate(90deg);
  }

  .detail-pane {
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    padding: 22px 28px 28px;
  }

  .empty {
    color: var(--muted);
  }
</style>
