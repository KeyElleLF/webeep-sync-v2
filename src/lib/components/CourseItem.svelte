<script lang="ts">
  import type { CourseRow, ManualState, SemesterSource } from "$lib/api";
  import { isActive, semesterLabel, semesterNumber, type RowStatus } from "$lib/format";
  import { t, tn, type MessageKey } from "$lib/i18n/index.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    course: CourseRow;
    /** Other listed courses of the same teaching (affected by passed/ignored too). */
    siblings: number;
    disabled: boolean;
    ontoggle: (course: CourseRow, selected: boolean) => void;
    onrename: (course: CourseRow, folder: string) => void;
    onstate: (course: CourseRow, state: ManualState | null) => void;
    onsemester: (course: CourseRow, semester: 1 | 2 | null) => void;
    onfolder: (courseId: number) => void;
    /** Outcome of the last sync, if this course was in it. */
    status: RowStatus | null;
    /** This course is being synced right now. */
    syncing: boolean;
  }

  let {
    course,
    siblings,
    disabled,
    ontoggle,
    onrename,
    onstate,
    onsemester,
    onfolder,
    status,
    syncing,
  }: Props = $props();
  let editing = $state(false);
  let draft = $state("");
  let confirming = $state(false);
  let choosingSemester = $state(false);
  /** The line under the row with the files of the last sync or the reason of its error. */
  let detail = $state(false);

  /** File names listed under the row before "and N more". */
  const MAX_FILES = 6;
  const SOURCES: Record<SemesterSource, MessageKey> = {
    manual: "courses.sourceShort.manual",
    studyPlan: "courses.sourceShort.studyPlan",
    fileDates: "courses.sourceShort.fileDates",
    startDate: "courses.sourceShort.startDate",
  };

  const active = $derived(isActive(course.state));
  const manualSemester = $derived(
    course.semester?.source === "manual" ? (course.semester.semester === "first" ? "1" : "2") : "",
  );
  // With a manual choice the automatic guess is not known, so it is not shown.
  const autoLabel = $derived.by(() => {
    if (course.semester?.source === "manual") return t("courses.auto");
    return t("courses.autoWith", { semester: semesterNumber(course.semester) });
  });
  const semesterText = $derived(
    course.semester
      ? t("courses.semesterShort", {
          semester: semesterNumber(course.semester),
          source: t(SOURCES[course.semester.source]),
        })
      : "",
  );
  const details = $derived(
    [course.teachers.join(", "), course.academicYear, course.section].filter(Boolean).join(" · "),
  );
  const pickId = $derived(`course-${course.id}`);

  function startEditing() {
    draft = course.folder;
    editing = true;
  }

  function save() {
    editing = false;
    const folder = draft.trim();
    if (folder && folder !== course.folder) onrename(course, folder);
  }

  function pickSemester(value: string) {
    choosingSemester = false;
    onsemester(course, value === "1" ? 1 : value === "2" ? 2 : null);
  }

  function focus(node: HTMLInputElement | HTMLSelectElement) {
    node.focus();
    if (node instanceof HTMLInputElement) node.select();
  }
</script>

<li
  class="row"
  class:inactive={!active}
  class:pending={course.state === "examPending"}
  class:off={!course.selected}
>
  <div class="pick">
    <input
      id={pickId}
      type="checkbox"
      checked={course.selected}
      {disabled}
      onchange={(e) => ontoggle(course, e.currentTarget.checked)}
    />
    <span class="course">
      <span class="title">
        <label class="name ellipsis" for={pickId} title={course.fullname}>{course.name}</label>
        {#if syncing}
          <span class="spinner" role="img" aria-label={t("courses.rowSyncing")}></span>
        {:else if status?.kind === "error"}
          <button
            class="state error"
            aria-expanded={detail}
            title={status.message}
            onclick={() => (detail = !detail)}
          >
            <Icon name="alert" size={13} /><span class="sr-only">{status.message}</span>
          </button>
        {:else if status?.kind === "done" && status.added > 0}
          <button
            class="state added"
            aria-expanded={detail}
            title={t("courses.rowAdded", { count: status.added })}
            onclick={() => (detail = !detail)}
          >
            +{status.added}<span class="sr-only"
              >{t("courses.rowAdded", { count: status.added })}</span
            >
          </button>
        {/if}
      </span>
      <span class="meta ellipsis" title={details}>{details}</span>
    </span>
  </div>

  <span class="semester">
    {#if !course.current}
      <span class="meta">—</span>
    {:else if choosingSemester}
      <select
        value={manualSemester}
        {disabled}
        aria-label={t("courses.semesterOf", { name: course.name })}
        use:focus
        onchange={(e) => pickSemester(e.currentTarget.value)}
        onblur={() => (choosingSemester = false)}
        onkeydown={(e) => e.key === "Escape" && (choosingSemester = false)}
      >
        <option value="">{autoLabel}</option>
        <option value="1">{t("courses.semester1")}</option>
        <option value="2">{t("courses.semester2")}</option>
      </select>
    {:else if course.semester}
      <span class="semester-text ellipsis" title={semesterLabel(course.semester)}
        >{semesterText}</span
      >
      <button
        class="link"
        {disabled}
        aria-label={t("courses.semesterOf", { name: course.name })}
        onclick={() => (choosingSemester = true)}>{t("common.change")}</button
      >
    {:else}
      <button class="link unknown" {disabled} onclick={() => (choosingSemester = true)}
        >{t("courses.semesterPick")}</button
      >
    {/if}
  </span>

  <span class="folder" class:editing>
    {#if !course.selected}
      <span class="meta">{t("common.notSynced")}</span>
    {:else if editing}
      <input
        type="text"
        bind:value={draft}
        aria-label={t("courses.folderName")}
        use:focus
        onkeydown={(e) => {
          if (e.key === "Enter") save();
          if (e.key === "Escape") editing = false;
        }}
      />
      <button class="small" onclick={save} {disabled}>{t("common.save")}</button>
      <button class="small" onclick={() => (editing = false)}>{t("common.cancel")}</button>
    {:else}
      {#if course.folder === course.name}
        <span class="path same" title={course.folder}>{t("courses.folderSame")}</span>
      {:else}
        <span class="path ellipsis" title={course.folder}>{course.folder}</span>
      {/if}
      <button class="link" {disabled} onclick={startEditing}>{t("common.change")}</button>
    {/if}
  </span>

  <span class="actions" hidden={editing}>
    {#if course.state === "passed" || course.state === "ignored"}
      <button class="link" {disabled} onclick={() => onstate(course, null)}
        >{t("courses.restore")}</button
      >
    {:else if !confirming}
      <button class="link" {disabled} onclick={() => (confirming = true)}
        >{t("courses.passed")}</button
      >
      {#if active}
        <button class="link" {disabled} onclick={() => onstate(course, "ignored")}
          >{t("courses.ignore")}</button
        >
      {/if}
    {/if}
  </span>

  {#if detail && status && !syncing}
    <div class="detail" class:error={status.kind === "error"}>
      {#if status.kind === "error"}
        <p>{status.message}</p>
      {:else}
        <ul>
          {#each status.files.slice(0, MAX_FILES) as file (file)}
            <li class="ellipsis" title={file}>{file}</li>
          {/each}
          {#if status.files.length > MAX_FILES}
            <li class="more">
              {t("courses.moreFiles", { count: status.files.length - MAX_FILES })}
            </li>
          {/if}
        </ul>
      {/if}
      <button class="small" onclick={() => onfolder(course.id)}>
        <Icon name="folder" size={14} />{t("links.openFolder")}
      </button>
    </div>
  {/if}

  {#if confirming}
    <div class="confirm">
      <span>
        {t("courses.confirmPassed.before")}
        <strong>{course.name}</strong>
        {t("courses.confirmPassed.after")}{siblings > 0
          ? ` ${tn("courses.confirmSiblings", siblings)}`
          : ""}?
      </span>
      <button
        class="small primary"
        {disabled}
        onclick={() => {
          confirming = false;
          onstate(course, "passed");
        }}>{t("common.yes")}</button
      >
      <button class="small" onclick={() => (confirming = false)}>{t("common.no")}</button>
    </div>
  {/if}
</li>

<style>
  .row {
    display: grid;
    grid-template-columns: var(--course-cols);
    gap: 4px 12px;
    align-items: center;
    padding: 7px 12px;
    transition: background-color var(--t-out) var(--ease);
  }

  /* Rows are siblings rendered by different instances. */
  .row + :global(.row) {
    border-top: 1px solid var(--border);
  }

  .row:hover,
  .row:focus-within {
    background: rgba(255, 255, 255, 0.025);
    transition-duration: var(--t-in);
  }

  /* The checkbox and the name keep their own columns; clicking the name ticks the box. */
  .pick {
    grid-column: 1 / 3;
    display: grid;
    grid-template-columns: subgrid;
    align-items: center;
    min-width: 0;
  }

  .pick input {
    justify-self: center;
    margin: 0;
  }

  .course {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }

  .title {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  /* WeBeep sends names in capitals: small caps keep the shape without shouting. */
  .name {
    font-weight: 650;
    font-variant-caps: all-small-caps;
    font-size: 1.15em;
    letter-spacing: 0.03em;
    cursor: pointer;
    transition: color var(--t-out) var(--ease);
  }

  .state {
    flex: none;
    display: inline-flex;
    align-items: center;
    gap: 0;
    padding: 0 6px;
    border: none;
    border-radius: 9px;
    box-shadow: none;
    font-size: 0.82em;
    font-weight: 600;
    line-height: 18px;
    font-variant-numeric: tabular-nums;
    animation: pop 260ms var(--ease-enter);
  }

  .state.added {
    background: color-mix(in srgb, var(--ok) 14%, transparent);
    color: var(--ok);
  }

  .state.added:hover {
    background: color-mix(in srgb, var(--ok) 26%, transparent);
  }

  .state.error {
    padding: 0 5px;
    background: color-mix(in srgb, var(--danger) 16%, transparent);
    color: var(--danger);
  }

  .state.error:hover {
    background: color-mix(in srgb, var(--danger) 28%, transparent);
  }

  .inactive .name,
  .off .name {
    font-weight: 500;
  }

  /* An unticked course reads as switched off, without hiding it. */
  .off .name {
    color: var(--muted);
  }

  .meta {
    color: var(--muted);
    font-size: 0.85em;
  }

  /* The semester reads as text; "Cambia" opens the choice, like the folder next to it. */
  .semester,
  .folder,
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .semester-text {
    color: var(--muted);
    font-size: 0.9em;
    white-space: nowrap;
  }

  .semester select {
    width: 100%;
    font-size: 0.9em;
    padding-block: 4px;
  }

  .actions {
    justify-content: flex-end;
    gap: 10px;
  }

  /* Row actions stay grey until the row is pointed at: one row of colour at a time. */
  .semester button.link,
  .folder button.link,
  .actions button.link {
    color: var(--muted);
  }

  /* An unknown semester asks for a choice in plain sight. */
  .semester button.unknown {
    color: var(--accent);
  }

  /* "Superato" and "Ignora" appear on the row under the pointer or holding the focus. */
  .actions {
    opacity: 0;
    transition: opacity var(--t-out) var(--ease);
  }

  .row:hover .actions,
  .row:focus-within .actions {
    opacity: 1;
    transition-duration: var(--t-in);
  }

  /* Passed and ignored courses keep "Ripristina" in view: it is how they come back. Courses with
     an exam pending keep "Superato": it is the action expected there. */
  .inactive .actions,
  .pending .actions {
    opacity: 1;
  }

  .row:hover .semester button.link,
  .row:hover .folder button.link,
  .row:hover .actions button.link,
  .semester button.link:focus-visible,
  .folder button.link:focus-visible,
  .actions button.link:focus-visible {
    color: var(--accent);
  }

  /* While renaming, the folder field also takes the actions column. */
  .folder.editing {
    grid-column: 4 / -1;
  }

  .actions[hidden] {
    display: none;
  }

  .path {
    font-size: 0.9em;
  }

  .path.same {
    color: var(--muted);
    white-space: nowrap;
  }

  .folder input {
    flex: 1;
    min-width: 0;
    font-size: 0.9em;
    padding-block: 4px;
  }

  button.small {
    padding: 3px 10px;
    font-size: 0.85em;
  }

  .detail,
  .confirm {
    grid-column: 2 / -1;
    display: flex;
    gap: 12px;
    margin: 2px 0 4px;
    padding: 8px 12px;
    border-radius: var(--r-md);
    animation: rise-in 200ms var(--ease-enter);
  }

  .detail {
    align-items: flex-start;
    background: color-mix(in srgb, var(--ok) 8%, transparent);
    font-size: 0.88em;
  }

  .detail.error {
    background: color-mix(in srgb, var(--danger) 9%, transparent);
  }

  .detail ul,
  .detail p {
    flex: 1;
    min-width: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .detail .more {
    color: var(--muted);
  }

  .detail button {
    flex: none;
  }

  .confirm {
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    background: var(--selected);
    font-size: 0.9em;
  }

  .confirm span {
    flex: 1;
    min-width: 200px;
  }
</style>
