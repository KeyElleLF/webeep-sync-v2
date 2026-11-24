<script lang="ts">
  // One course in the list of the Corsi tab: the sync checkbox, the name (which opens the course
  // in the detail pane) and at most one signal from the last sync.
  import type { CourseRow } from "$lib/api";
  import { courseTitle } from "$lib/course-title";
  import type { RowStatus } from "$lib/format";
  import { t } from "$lib/i18n/index.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    course: CourseRow;
    /** Shown in the detail pane. */
    current: boolean;
    disabled: boolean;
    /** Outcome of the last sync, if this course was in it. */
    status: RowStatus | null;
    /** This course is being synced right now. */
    syncing: boolean;
    ontoggle: (course: CourseRow, selected: boolean) => void;
    onselect: (course: CourseRow) => void;
    /** Arrow keys move through the list. */
    onmove: (event: KeyboardEvent) => void;
  }

  let { course, current, disabled, status, syncing, ontoggle, onselect, onmove }: Props =
    $props();
  const name = $derived(courseTitle(course.name));
  /** A course of this year whose semester nobody knows yet: it waits for a choice. */
  const unknown = $derived(course.current && course.semester === null);
</script>

<li class="row" class:current class:off={!course.selected}>
  <input
    type="checkbox"
    checked={course.selected}
    {disabled}
    aria-label={t("courses.syncOf", { name })}
    onchange={(e) => ontoggle(course, e.currentTarget.checked)}
  />
  <button
    class="name"
    data-id={course.id}
    title={course.fullname}
    aria-current={current ? "true" : undefined}
    onclick={() => onselect(course)}
    onkeydown={onmove}>{name}</button
  >
  <span class="signal">
    {#if syncing}
      <span class="spinner" role="img" aria-label={t("courses.rowSyncing")}></span>
    {:else if status?.kind === "error"}
      <span class="error" role="img" aria-label={status.message}><Icon name="alert" size={14} /></span>
    {:else if status?.kind === "done" && status.added > 0}
      <span class="badge" aria-hidden="true">+{status.added}</span>
      <span class="sr-only">{t("courses.rowAdded", { count: status.added })}</span>
    {:else if unknown}
      <span class="word">{t("courses.semesterUnknownShort")}</span>
    {/if}
  </span>
</li>

<style>
  .row {
    display: grid;
    grid-template-columns: 16px minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    margin: 0 8px;
    padding: 0 12px;
    border-radius: var(--r-md);
    transition: background-color var(--t-out) var(--ease);
  }

  .row:hover {
    background: var(--hover);
    transition-duration: var(--t-in);
  }

  .row.current {
    background: var(--selected);
  }

  .name {
    display: block;
    min-width: 0;
    min-height: 36px;
    padding: 0;
    border: none;
    background: none;
    color: var(--text);
    font-weight: 500;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .name:focus-visible {
    outline-offset: -2px;
    border-radius: var(--r-sm);
  }

  /* The row draws the hover; the name stays text. */
  .name:hover:not(:disabled) {
    background: none;
  }

  .name:active:not(:disabled) {
    transform: none;
  }

  /* A course that is not synced reads as switched off, without hiding it. */
  .off .name {
    color: var(--muted);
  }

  .signal {
    display: flex;
    align-items: center;
  }

  .error {
    display: flex;
    color: var(--danger);
  }

  .word {
    color: var(--muted);
    font-size: var(--fs-sm);
  }
</style>
