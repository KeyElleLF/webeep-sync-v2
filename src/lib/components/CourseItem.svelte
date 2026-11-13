<script lang="ts">
  import type { CourseRow, ManualState } from "$lib/api";
  import { isActive, semesterLabel, semesterNumber } from "$lib/format";

  interface Props {
    course: CourseRow;
    /** Other listed courses of the same teaching (affected by passed/ignored too). */
    siblings: number;
    disabled: boolean;
    ontoggle: (course: CourseRow, selected: boolean) => void;
    onrename: (course: CourseRow, folder: string) => void;
    onstate: (course: CourseRow, state: ManualState | null) => void;
    onsemester: (course: CourseRow, semester: 1 | 2 | null) => void;
  }

  let { course, siblings, disabled, ontoggle, onrename, onstate, onsemester }: Props = $props();
  let editing = $state(false);
  let draft = $state("");
  let confirming = $state(false);

  const active = $derived(isActive(course.state));
  const manualSemester = $derived(
    course.semester?.source === "manual" ? (course.semester.semester === "first" ? "1" : "2") : "",
  );
  // With a manual choice the automatic guess is not known, so it is not shown.
  const autoLabel = $derived.by(() => {
    if (course.semester?.source === "manual") return "Automatico";
    return course.semester ? `Automatico: ${semesterNumber(course.semester)}` : "Automatico: ?";
  });
  const details = $derived(
    [course.teachers.join(", "), course.academicYear, course.section].filter(Boolean).join(" · "),
  );

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
    onsemester(course, value === "1" ? 1 : value === "2" ? 2 : null);
  }

  function focus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<li class="row" class:inactive={!active} class:off={!course.selected}>
  <label class="pick">
    <input
      type="checkbox"
      checked={course.selected}
      {disabled}
      onchange={(e) => ontoggle(course, e.currentTarget.checked)}
    />
    <span class="course">
      <span class="name ellipsis" title={course.fullname}>{course.name}</span>
      <span class="meta ellipsis" title={details}>{details}</span>
    </span>
  </label>

  <span class="semester">
    {#if course.current}
      <select
        value={manualSemester}
        {disabled}
        aria-label="Semestre di {course.name}"
        title={semesterLabel(course.semester)}
        onchange={(e) => pickSemester(e.currentTarget.value)}
      >
        <option value="">{autoLabel}</option>
        <option value="1">1° semestre</option>
        <option value="2">2° semestre</option>
      </select>
    {:else}
      <span class="meta">—</span>
    {/if}
  </span>

  <span class="folder" class:editing>
    {#if !course.selected}
      <span class="meta">non sincronizzato</span>
    {:else if editing}
      <input
        type="text"
        bind:value={draft}
        aria-label="Nome della cartella"
        use:focus
        onkeydown={(e) => {
          if (e.key === "Enter") save();
          if (e.key === "Escape") editing = false;
        }}
      />
      <button class="small" onclick={save} {disabled}>Salva</button>
      <button class="small" onclick={() => (editing = false)}>Annulla</button>
    {:else}
      <span class="path ellipsis" title={course.folder}>{course.folder}</span>
      <button class="link" {disabled} onclick={startEditing}>Cambia</button>
    {/if}
  </span>

  <span class="actions" hidden={editing}>
    {#if course.state === "passed" || course.state === "ignored"}
      <button class="link" {disabled} onclick={() => onstate(course, null)}>Ripristina</button>
    {:else if !confirming}
      <button class="link" {disabled} onclick={() => (confirming = true)}>Superato</button>
      {#if active}
        <button class="link" {disabled} onclick={() => onstate(course, "ignored")}>Ignora</button>
      {/if}
    {/if}
  </span>

  {#if confirming}
    <div class="confirm">
      <span>
        Segnare <strong>{course.name}</strong> come superato{siblings > 0
          ? ` (anche gli altri ${siblings} corsi dello stesso insegnamento)`
          : ""}?
      </span>
      <button
        class="small primary"
        {disabled}
        onclick={() => {
          confirming = false;
          onstate(course, "passed");
        }}>Sì</button
      >
      <button class="small" onclick={() => (confirming = false)}>No</button>
    </div>
  {/if}
</li>

<style>
  .row {
    display: grid;
    grid-template-columns: var(--course-cols);
    gap: 4px 12px;
    align-items: center;
    padding: 6px 12px;
  }

  /* Rows are siblings rendered by different instances. */
  .row + :global(.row) {
    border-top: 1px solid var(--border);
  }

  .row:hover {
    background: var(--hover);
  }

  /* The checkbox and the name share one click target but keep their own columns. */
  .pick {
    grid-column: 1 / 3;
    display: grid;
    grid-template-columns: subgrid;
    align-items: center;
    min-width: 0;
    cursor: pointer;
  }

  .pick input {
    justify-self: center;
    margin: 0;
  }

  .course {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-weight: 600;
  }

  .inactive .name,
  .off .name {
    font-weight: 500;
  }

  .meta {
    color: var(--muted);
    font-size: 0.85em;
  }

  .semester select {
    width: 100%;
    font-size: 0.9em;
  }

  .folder,
  .actions {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .actions {
    justify-content: flex-end;
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

  .folder input {
    flex: 1;
    min-width: 0;
    font-size: 0.9em;
  }

  button.small {
    padding: 2px 10px;
    font-size: 0.85em;
  }

  .confirm {
    grid-column: 2 / -1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 4px 0 2px;
    font-size: 0.9em;
  }
</style>
