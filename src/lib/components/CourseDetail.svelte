<script lang="ts">
  // The course chosen in the list of the Corsi tab: whether it syncs, its semester and folder,
  // what the last sync brought, its links, and "passed" / "ignore".
  import type { CourseLink, CourseRow, ManualState, SemesterSource } from "$lib/api";
  import { courseTitle } from "$lib/course-title";
  import { isActive, semesterNumber, type RowStatus } from "$lib/format";
  import { displayName, t, tn, type MessageKey } from "$lib/i18n/index.svelte";
  import Icon from "./Icon.svelte";
  import Switch from "./Switch.svelte";

  interface Props {
    course: CourseRow;
    /** Other listed courses of the same teaching (affected by passed/ignored too). */
    siblings: number;
    disabled: boolean;
    status: RowStatus | null;
    syncing: boolean;
    links: CourseLink[];
    ontoggle: (course: CourseRow, selected: boolean) => void;
    onrename: (course: CourseRow, folder: string) => void;
    onstate: (course: CourseRow, state: ManualState | null) => void;
    onsemester: (course: CourseRow, semester: 1 | 2 | null) => void;
    onfolder: (courseId: number) => void;
    onopenlink: (link: CourseLink) => void;
  }

  let {
    course,
    siblings,
    disabled,
    status,
    syncing,
    links,
    ontoggle,
    onrename,
    onstate,
    onsemester,
    onfolder,
    onopenlink,
  }: Props = $props();

  /** File names listed before "and N more". */
  const MAX_FILES = 8;
  const SOURCES: Record<SemesterSource, MessageKey> = {
    manual: "courses.sourceShort.manual",
    studyPlan: "courses.sourceShort.studyPlan",
    fileDates: "courses.sourceShort.fileDates",
    startDate: "courses.sourceShort.startDate",
  };
  const STATE_NOTES: Partial<Record<CourseRow["state"], MessageKey>> = {
    passed: "courses.statePassed",
    ignored: "courses.stateIgnored",
    past: "courses.statePast",
  };

  let editing = $state(false);
  let draft = $state("");
  let confirming = $state(false);

  const name = $derived(courseTitle(course.name));
  const active = $derived(isActive(course.state));
  const retired = $derived(course.state === "passed" || course.state === "ignored");
  const meta = $derived(
    [
      course.teachers.join(", "),
      course.academicYear,
      course.section,
      course.code ? t("courses.code", { code: course.code }) : null,
    ]
      .filter(Boolean)
      .join(" · "),
  );
  const semesterChoice = $derived(
    course.semester?.source === "manual"
      ? course.semester.semester === "first"
        ? "1"
        : "2"
      : "auto",
  );
  const semesterText = $derived(
    course.semester
      ? t("courses.semesterShort", {
          semester: semesterNumber(course.semester),
          source: t(SOURCES[course.semester.source]),
        })
      : t("courses.semesterUnknown"),
  );
  const SEMESTER_OPTIONS = $derived([
    { value: "auto", label: t("courses.auto") },
    { value: "1", label: t("courses.semester1Short") },
    { value: "2", label: t("courses.semester2Short") },
  ]);

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

  /** The question takes the place of the button that asked it: the safe answer gets the focus. */
  function focusButton(node: HTMLButtonElement) {
    node.focus();
  }

  function host(url: string): string {
    try {
      return new URL(url).hostname;
    } catch {
      return "";
    }
  }
</script>

<article class="detail" aria-labelledby="course-detail-title">
  <h2 id="course-detail-title" title={course.fullname}>{name}</h2>
  <p class="meta">{meta}</p>

  <div class="box">
    <div class="srow">
      <div class="lab">
        <span id="course-sync-label">{t("courses.syncThis")}</span>
        <span class="d">{t("courses.syncThisHint")}</span>
      </div>
      <div class="ctl">
        <Switch
          checked={course.selected}
          {disabled}
          labelledby="course-sync-label"
          onchange={(selected) => ontoggle(course, selected)}
        />
      </div>
    </div>

    {#if course.current && active}
      <div class="srow">
        <div class="lab">
          <span>{t("courses.col.semester")}</span>
          <span class="d" class:unknown={!course.semester}>{semesterText}</span>
        </div>
        <div class="segm" role="group" aria-label={t("courses.semesterOf", { name })}>
          {#each SEMESTER_OPTIONS as option (option.value)}
            <button
              aria-pressed={semesterChoice === option.value}
              {disabled}
              onclick={() => pickSemester(option.value)}>{option.label}</button
            >
          {/each}
        </div>
      </div>
    {/if}

    {#if course.selected}
      <div class="srow">
        <div class="lab">
          <span id="course-folder-label">{t("courses.col.folder")}</span>
          {#if editing}
            <input
              type="text"
              bind:value={draft}
              aria-labelledby="course-folder-label"
              use:focus
              onkeydown={(e) => {
                if (e.key === "Enter") save();
                if (e.key === "Escape") editing = false;
              }}
            />
          {:else}
            <span class="d path" title={course.folder}>
              <Icon name="folder" size={14} /><span class="ellipsis">{course.folder}</span>
            </span>
          {/if}
        </div>
        <div class="ctl">
          {#if editing}
            <button onclick={save} {disabled}>{t("common.save")}</button>
            <button onclick={() => (editing = false)}>{t("common.cancel")}</button>
          {:else}
            <button
              {disabled}
              aria-label={t("courses.renameOf", { name })}
              onclick={startEditing}>{t("courses.rename")}</button
            >
            <button
              aria-label={t("courses.openOf", { name })}
              onclick={() => onfolder(course.id)}>{t("courses.open")}</button
            >
          {/if}
        </div>
      </div>
    {/if}

    {#if syncing || status}
      <div class="srow stack">
        <div class="lab">
          <span>{t("courses.lastSync")}</span>
          <span class="d">
            {#if syncing}
              {t("courses.rowSyncing")}
            {:else if status?.kind === "done"}
              {status.added > 0
                ? t("courses.rowAdded", { count: status.added })
                : t("courses.lastSyncNothing")}
            {/if}
          </span>
        </div>
        {#if !syncing && status?.kind === "error"}
          <p class="error"><Icon name="alert" size={14} />{status.message}</p>
        {:else if !syncing && status?.kind === "done" && status.files.length > 0}
          <ul class="files">
            {#each status.files.slice(0, MAX_FILES) as file (file)}
              <li><Icon name="file" size={14} /><span class="ellipsis" title={file}>{file}</span></li>
            {/each}
            {#if status.files.length > MAX_FILES}
              <li class="more">
                {t("courses.moreFiles", { count: status.files.length - MAX_FILES })}
              </li>
            {/if}
          </ul>
        {/if}
      </div>
    {/if}

    {#if links.length > 0}
      <div class="srow stack">
        <div class="lab">
          <span>{t("courses.links")}</span>
          <span class="d">{t("courses.linksHint")}</span>
        </div>
        <ul class="links">
          {#each links as link (link.id)}
            <li>
              <button class="open" title={link.url} onclick={() => onopenlink(link)}>
                {#if !link.seen}
                  <span class="new-dot" aria-hidden="true"></span>
                  <span class="sr-only">{t("links.new")}</span>
                {/if}
                <span class="name">{displayName(link.name)}</span>
                <Icon name="external" size={13} />
              </button>
              <span class="where ellipsis">{host(link.url)}</span>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>

  {#if STATE_NOTES[course.state]}
    <p class="state">{t(STATE_NOTES[course.state] as MessageKey)}</p>
  {/if}

  {#if confirming}
    <div class="confirm" role="group" aria-label={t("courses.markPassed")}>
      <p>
        {t("courses.confirmPassed.before")}
        <strong>{name}</strong>
        {t("courses.confirmPassed.after")}{siblings > 0
          ? ` ${tn("courses.confirmSiblings", siblings)}`
          : ""}?
      </p>
      <button
        class="primary"
        {disabled}
        onclick={() => {
          confirming = false;
          onstate(course, "passed");
        }}>{t("common.yes")}</button
      >
      <button use:focusButton onclick={() => (confirming = false)}>{t("common.no")}</button>
    </div>
  {:else}
    <div class="actions">
      {#if retired}
        <button {disabled} onclick={() => onstate(course, null)}>{t("courses.restore")}</button>
      {:else}
        <button {disabled} onclick={() => (confirming = true)}>{t("courses.markPassed")}</button>
        {#if active}
          <button {disabled} onclick={() => onstate(course, "ignored")}>{t("courses.ignore")}</button>
        {/if}
      {/if}
    </div>
    <p class="note">{t("courses.stateNote")}</p>
  {/if}
</article>

<style>
  .detail {
    min-width: 0;
    max-width: 760px;
  }

  h2 {
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .meta {
    margin: 6px 0 18px;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .box {
    border: 1px solid var(--border);
    border-radius: var(--r-lg);
    background: var(--card);
  }

  .srow {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 60px;
    padding: 11px 16px;
    box-sizing: border-box;
  }

  .srow + .srow {
    border-top: 1px solid var(--border);
  }

  .srow.stack {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .lab {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .d {
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .d.unknown {
    color: var(--warn);
  }

  .path {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .lab input {
    margin-top: 4px;
    max-width: 420px;
  }

  .ctl {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* Auto · 1° · 2°: one choice of three, side by side. */
  .segm {
    flex: none;
    display: inline-flex;
    gap: 2px;
    padding: 2px;
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    background: var(--bg);
  }

  .segm button {
    min-width: 44px;
    padding: 4px 12px;
    border: none;
    border-radius: var(--r-sm);
    background: none;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .segm button:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .segm button[aria-pressed="true"] {
    background: var(--raised);
    color: var(--text);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .files li {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 28px;
    min-width: 0;
  }

  .files li :global(.icon) {
    color: var(--muted);
  }

  .files .more {
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .error {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 0;
    color: var(--text);
  }

  .error :global(.icon) {
    margin-top: 2px;
    color: var(--danger);
  }

  .links li {
    display: flex;
    align-items: baseline;
    gap: 10px;
    min-height: 28px;
    min-width: 0;
  }

  .open {
    flex: none;
    gap: 6px;
    padding: 0;
    border: none;
    background: none;
    color: var(--text);
  }

  .open:hover:not(:disabled),
  .open:focus-visible {
    background: none;
    color: var(--accent);
  }

  .open :global(.icon) {
    color: var(--muted);
  }

  .open .name {
    font-weight: 500;
  }

  .where {
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .state {
    margin: 16px 0 0;
    color: var(--muted);
  }

  .actions {
    display: flex;
    gap: 8px;
    margin-top: 20px;
  }

  .note {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .confirm {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 20px;
    padding: 12px 14px;
    border: 1px solid var(--border-strong);
    border-radius: var(--r-md);
    background: var(--card);
  }

  .confirm p {
    flex: 1 1 260px;
    margin: 0;
  }
</style>
