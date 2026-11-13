<script lang="ts">
  import type { Announcement, CourseRow } from "$lib/api";
  import { formatBytes } from "$lib/format";
  import { locale, t } from "$lib/i18n/index.svelte";
  import { discussionUrl, interceptLinks, openExternal } from "$lib/links";

  interface Props {
    announcements: Announcement[];
    courses: CourseRow[];
    muted: number[];
    disabled: boolean;
    onread: (id: number | null) => void;
    onmute: (courseId: number, muted: boolean) => void;
  }

  let { announcements, courses, muted, disabled, onread, onmute }: Props = $props();
  let courseFilter = $state<number | "all">("all");
  let unreadOnly = $state(false);
  let selectedId = $state<number | null>(null);
  let list = $state<HTMLUListElement>();

  const FULL = $derived(
    new Intl.DateTimeFormat(locale(), { dateStyle: "full", timeStyle: "short" }),
  );
  const TIME = $derived(new Intl.DateTimeFormat(locale(), { hour: "2-digit", minute: "2-digit" }));
  const DAY = $derived(new Intl.DateTimeFormat(locale(), { day: "numeric", month: "short" }));
  const DAY_YEAR = $derived(
    new Intl.DateTimeFormat(locale(), { day: "numeric", month: "short", year: "numeric" }),
  );

  const names = $derived(new Map(courses.map((c) => [c.id, c.name])));
  const withPosts = $derived(
    [...new Set(announcements.map((a) => a.courseId))]
      .map((id) => ({ id, name: names.get(id) ?? t("common.courseFallback", { id }) }))
      .sort((a, b) => a.name.localeCompare(b.name)),
  );
  const shown = $derived(
    announcements.filter(
      (a) => (courseFilter === "all" || a.courseId === courseFilter) && (!unreadOnly || !a.read),
    ),
  );
  const unread = $derived(announcements.filter((a) => !a.read).length);
  const selected = $derived(announcements.find((a) => a.id === selectedId) ?? null);

  /** Time for today's posts, day and month otherwise (with the year if not this year). */
  function shortDate(seconds: number): string {
    const date = new Date(seconds * 1000);
    const now = new Date();
    if (date.toDateString() === now.toDateString()) return TIME.format(date);
    return date.getFullYear() === now.getFullYear() ? DAY.format(date) : DAY_YEAR.format(date);
  }

  function select(a: Announcement) {
    selectedId = a.id;
    if (!a.read) onread(a.id);
  }

  /** Up and down arrows move through the list, like in a mail client. */
  function move(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const index = shown.findIndex((a) => a.id === selectedId);
    const step = event.key === "ArrowDown" ? 1 : -1;
    const next = shown[Math.min(Math.max(index + step, 0), shown.length - 1)];
    if (!next) return;
    select(next);
    list?.querySelector<HTMLButtonElement>(`[data-id="${next.id}"]`)?.focus();
  }
</script>

<div class="board">
  <section class="list-pane" aria-label={t("board.listLabel")}>
    <div class="filters">
      <select bind:value={courseFilter} aria-label={t("common.course")}>
        <option value="all">{t("board.allCourses")}</option>
        {#each withPosts as c (c.id)}
          <option value={c.id}>{c.name}</option>
        {/each}
      </select>
      <label>
        <input type="checkbox" bind:checked={unreadOnly} />
        {t("board.unreadOnly", { count: unread })}
      </label>
      {#if courseFilter !== "all"}
        {@const id = courseFilter}
        <label>
          <input
            type="checkbox"
            checked={muted.includes(id)}
            {disabled}
            onchange={(e) => onmute(id, e.currentTarget.checked)}
          />
          {t("board.mute")}
        </label>
      {/if}
    </div>

    {#if announcements.length === 0}
      <p class="empty">{t("board.empty", { button: t("board.checkNow") })}</p>
    {:else if shown.length === 0}
      <p class="empty">{t("board.noMatch")}</p>
    {/if}

    <ul bind:this={list}>
      {#each shown as a (a.id)}
        <li>
          <button
            class="item"
            class:unread={!a.read}
            class:active={a.id === selectedId}
            aria-current={a.id === selectedId ? "true" : undefined}
            data-id={a.id}
            onclick={() => select(a)}
            onkeydown={move}
          >
            <span class="top">
              <span class="subject ellipsis">{a.pinned ? "📌 " : ""}{a.subject}</span>
              <span class="date">{shortDate(a.created)}</span>
            </span>
            <span class="course ellipsis">{names.get(a.courseId) ?? ""}</span>
          </button>
        </li>
      {/each}
    </ul>
  </section>

  <article class="reader" aria-live="polite">
    {#if selected}
      <header>
        <h2>{selected.pinned ? "📌 " : ""}{selected.subject}</h2>
        <p class="meta">
          {names.get(selected.courseId) ?? ""} · {selected.author} · {FULL.format(
            new Date(selected.created * 1000),
          )}
        </p>
        <button class="link" onclick={() => openExternal(discussionUrl(selected.id))}>
          {t("common.openOnWebeep")}
        </button>
      </header>
      <!-- Sanitized by the backend (allow-list, no scripts); links open in the browser. -->
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="message" onclick={interceptLinks}>{@html selected.message}</div>
      {#if selected.attachments.length > 0}
        <div class="attachments">
          <p class="meta">{t("board.attachments")}</p>
          <ul>
            {#each selected.attachments as file (file.fileurl)}
              <li>{file.filename} ({formatBytes(file.filesize)})</li>
            {/each}
          </ul>
        </div>
      {/if}
    {:else if shown.length > 0}
      <p class="placeholder">{t("board.selectHint")}</p>
    {/if}
  </article>
</div>

<style>
  .board {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(280px, 360px) minmax(0, 1fr);
  }

  .list-pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-right: 1px solid var(--border);
    background: var(--card);
  }

  .filters {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--border);
  }

  .filters label {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--muted);
    font-size: 0.9em;
  }

  .empty {
    padding: 0 12px;
    color: var(--muted);
  }

  .list-pane ul {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .list-pane li + li {
    border-top: 1px solid var(--border);
  }

  .item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 8px 12px 8px 14px;
    border: none;
    border-left: 3px solid transparent;
    border-radius: 0;
    background: none;
    text-align: left;
  }

  .item:hover:not(.active) {
    background: var(--hover);
  }

  /* The left border marks unread posts only (buttons otherwise get an accent border on hover). */
  .item:hover:not(.unread) {
    border-left-color: transparent;
  }

  .item.active {
    background: var(--selected);
  }

  .item.unread {
    border-left-color: var(--accent);
  }

  .top {
    display: flex;
    gap: 8px;
    align-items: baseline;
  }

  .subject {
    flex: 1;
  }

  .unread .subject {
    font-weight: 700;
  }

  .date,
  .course {
    color: var(--muted);
    font-size: 0.85em;
  }

  .date {
    flex: none;
  }

  .reader {
    min-height: 0;
    overflow-y: auto;
    padding: 20px 28px;
  }

  .reader header {
    padding-bottom: 12px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }

  .reader h2 {
    margin: 0 0 4px;
    font-size: 1.2rem;
  }

  .meta {
    margin: 0 0 6px;
    color: var(--muted);
    font-size: 0.9em;
  }

  .message {
    max-width: 80ch;
    line-height: 1.5;
    overflow-wrap: anywhere;
  }

  .message :global(a) {
    color: var(--accent);
  }

  .message :global(pre) {
    white-space: pre-wrap;
  }

  .attachments {
    margin-top: 20px;
  }

  .attachments ul {
    margin: 0;
    padding-left: 20px;
  }

  .placeholder {
    margin-top: 40px;
    color: var(--muted);
    text-align: center;
  }
</style>
