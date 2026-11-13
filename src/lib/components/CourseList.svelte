<script lang="ts">
  import type { CourseRow, ManualState } from "$lib/api";
  import { SECTIONS } from "$lib/format";
  import { t, tn } from "$lib/i18n/index.svelte";
  import CourseItem from "./CourseItem.svelte";
  import InfoTip from "./InfoTip.svelte";

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
  }

  let { courses, disabled, ontoggle, onrename, onstate, onsemester, planLoaded, onplan }: Props =
    $props();
  let showAll = $state(false);

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
  const visible = $derived(groups.filter((g) => !g.extra || showAll));
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
  <span><strong>{selectedCount}</strong> {tn("courses.selected", selectedCount)}</span>
  <span class="plan">
    <button class="small" {disabled} onclick={onplan}>
      {planLoaded ? t("plan.loadedButton") : t("plan.importButton")}
    </button>
    <InfoTip text={t("plan.info")} label={t("plan.infoLabel")} />
  </span>
  <label>
    <input type="checkbox" bind:checked={showAll} />
    {t("courses.showAll")}{#if hiddenCount > 0}&nbsp;({hiddenCount}){/if}
  </label>
</div>
<p class="hint">{t("courses.selectionHint")}</p>

<div class="table">
  <div class="row head" aria-hidden="true">
    <span></span>
    <span>{t("common.course")}</span>
    <span>{t("courses.col.semester")}</span>
    <span>{t("courses.col.folder")}</span>
    <span></span>
  </div>

  {#each visible as group (group.title)}
    <h3>
      {t(group.title)}
      <span class="count">{group.courses.length}</span>
      <span class="group-hint">{t(group.hint)}</span>
    </h3>
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
  }

  .plan {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: auto;
  }

  button.small {
    padding: 4px 12px;
    font-size: 0.9em;
  }

  .bar label {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--muted);
  }

  .hint {
    margin: 4px 0 16px;
    color: var(--muted);
    font-size: 0.9em;
  }

  .table {
    /* Shared by every row (CourseItem uses it too), so the columns line up. */
    --course-cols: 32px minmax(0, 5fr) 160px minmax(0, 4fr) 150px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  @media (max-width: 1050px) {
    .table {
      --course-cols: 28px minmax(0, 5fr) 130px minmax(0, 3fr) 130px;
    }
  }

  .head {
    display: grid;
    grid-template-columns: var(--course-cols);
    gap: 12px;
    padding: 8px 12px;
    color: var(--muted);
    font-size: 0.85em;
    font-weight: 600;
    border-bottom: 1px solid var(--border);
  }

  h3 {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin: 0;
    padding: 10px 12px 6px;
    font-size: 0.95rem;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
  }

  ul + h3 {
    border-top: 1px solid var(--border);
  }

  .count {
    color: var(--muted);
    font-weight: 400;
  }

  .group-hint {
    color: var(--muted);
    font-size: 0.85em;
    font-weight: 400;
  }

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
</style>
