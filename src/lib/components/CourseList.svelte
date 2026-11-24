<script lang="ts">
  import type { CourseRow, ManualState } from "$lib/api";
  import { SECTIONS, type RowStatus } from "$lib/format";
  import { t, tn } from "$lib/i18n/index.svelte";
  import CourseItem from "./CourseItem.svelte";
  import InfoTip from "./InfoTip.svelte";
  import { view } from "$lib/view-state.svelte";

  interface Props {
    courses: CourseRow[];
    disabled: boolean;
    ontoggle: (course: CourseRow, selected: boolean) => void;
    onrename: (course: CourseRow, folder: string) => void;
    onstate: (course: CourseRow, state: ManualState | null) => void;
    onsemester: (course: CourseRow, semester: 1 | 2 | null) => void;
    /** A study plan is loaded. */
    planLoaded: boolean;
    onplan: () => void;
    /** Outcome of the last sync per course id. */
    results: Map<number, RowStatus>;
    /** Name of the course being synced right now. */
    syncingName: string | null;
    onfolder: (courseId: number) => void;
  }

  let {
    courses,
    disabled,
    ontoggle,
    onrename,
    onstate,
    onsemester,
    planLoaded,
    onplan,
    results,
    syncingName,
    onfolder,
  }: Props = $props();

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
  const visible = $derived(groups.filter((g) => !g.extra || view.showAllCourses));
  const hiddenCount = $derived(
    groups.filter((g) => g.extra).reduce((n, g) => n + g.courses.length, 0),
  );
  const selectedCount = $derived(courses.filter((c) => c.selected).length);

  function siblings(course: CourseRow): number {
    return courses.filter((c) => c.teachingKey === course.teachingKey && c.id !== course.id)
      .length;
  }
</script>

<div class="bar">
  <span class="selected"
    ><strong>{selectedCount}</strong> {tn("courses.selected", selectedCount)}
    <InfoTip text={t("courses.selectionHint")} label={t("courses.selectionInfoLabel")} /></span
  >
  <span class="plan">
    <button class="small" {disabled} onclick={onplan}>
      {planLoaded ? t("plan.loadedButton") : t("plan.importButton")}
    </button>
    <InfoTip text={t("plan.info")} label={t("plan.infoLabel")} />
  </span>
  <label>
    <input type="checkbox" bind:checked={view.showAllCourses} />
    {t("courses.showAll")}{#if hiddenCount > 0}&nbsp;({hiddenCount}){/if}
  </label>
</div>

<div class="table">
  <div class="row head" aria-hidden="true">
    <span></span>
    <span>{t("common.course")}</span>
    <span>{t("courses.col.semester")}</span>
    <span>{t("courses.col.folder")}</span>
    <span></span>
  </div>

  {#each visible as group (group.title)}
    <h2>
      {t(group.title)}
      <span class="count">{group.courses.length}</span>
      <span class="group-hint">{t(group.hint)}</span>
    </h2>
    <ul>
      {#each group.courses as course (course.id)}
        <CourseItem
          {course}
          siblings={siblings(course)}
          {disabled}
          {ontoggle}
          {onrename}
          {onstate}
          {onsemester}
          {onfolder}
          status={results.get(course.id) ?? null}
          syncing={course.selected && course.name === syncingName}
        />
      {/each}
    </ul>
  {/each}
</div>

<style>
  .bar {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: 8px 16px;
    margin-bottom: 16px;
  }

  .selected {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .selected strong {
    font-variant-numeric: tabular-nums;
  }

  .plan {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: auto;
  }

  button.small {
    padding: 4px 12px;
    font-size: var(--fs-sm);
  }

  .bar label {
    display: flex;
    gap: 8px;
    align-items: center;
    color: var(--muted);
    cursor: pointer;
    user-select: none;
    transition: color var(--t-out) var(--ease);
  }

  .bar label:hover {
    color: var(--text);
  }

  .table {
    /* Shared by every row (CourseItem uses it too), so the columns line up. */
    --course-cols: 32px minmax(0, 6fr) minmax(170px, 2fr) minmax(0, 3fr) 140px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    /* Clip, not hidden: the column header can then stick to the top while scrolling. */
    overflow: clip;
  }

  @media (max-width: 1050px) {
    .table {
      --course-cols: 28px minmax(0, 5fr) minmax(150px, 2fr) minmax(150px, 2fr) 112px;
    }
  }

  .head {
    position: sticky;
    top: -20px;
    z-index: 1;
    display: grid;
    grid-template-columns: var(--course-cols);
    gap: 12px;
    padding: 9px 12px;
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

  ul + h2 {
    border-top: 1px solid var(--border);
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

  .group-hint {
    color: var(--muted);
    font-size: var(--fs-md);
    font-weight: 400;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
</style>
