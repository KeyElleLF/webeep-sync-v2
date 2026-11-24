<script lang="ts">
  import type { Announcement, CourseRow } from "$lib/api";
  import { courseTitle } from "$lib/course-title";
  import { formatBytes } from "$lib/format";
  import { locale, t } from "$lib/i18n/index.svelte";
  import { discussionUrl, interceptLinks, openExternal } from "$lib/links";
  import Icon from "./Icon.svelte";
  import { view } from "$lib/view-state.svelte";

  interface Props {
    announcements: Announcement[];
    courses: CourseRow[];
    muted: number[];
    disabled: boolean;
    onread: (id: number | null) => void;
    onmute: (courseId: number, muted: boolean) => void;
  }

  let { announcements, courses, muted, disabled, onread, onmute }: Props = $props();
  let list = $state<HTMLUListElement>();

  const FULL = $derived(
    new Intl.DateTimeFormat(locale(), { dateStyle: "full", timeStyle: "short" }),
  );
  const TIME = $derived(new Intl.DateTimeFormat(locale(), { hour: "2-digit", minute: "2-digit" }));
  const DAY = $derived(new Intl.DateTimeFormat(locale(), { day: "numeric", month: "short" }));
  const DAY_YEAR = $derived(
    new Intl.DateTimeFormat(locale(), { day: "numeric", month: "short", year: "numeric" }),
  );

  const names = $derived(new Map(courses.map((c) => [c.id, courseTitle(c.name)])));
  const withPosts = $derived(
    [...new Set(announcements.map((a) => a.courseId))]
      .map((id) => ({ id, name: names.get(id) ?? t("common.courseFallback", { id }) }))
      .sort((a, b) => a.name.localeCompare(b.name)),
  );
  const shown = $derived(
    announcements.filter(
      (a) => (view.boardCourse === "all" || a.courseId === view.boardCourse) && (!view.boardUnreadOnly || !a.read),
    ),
  );
  const unread = $derived(announcements.filter((a) => !a.read).length);
  const selected = $derived(announcements.find((a) => a.id === view.boardSelected) ?? null);

  /** Time for today's posts, day and month otherwise (with the year if not this year). */
  function shortDate(seconds: number): string {
    const date = new Date(seconds * 1000);
    const now = new Date();
    if (date.toDateString() === now.toDateString()) return TIME.format(date);
    return date.getFullYear() === now.getFullYear() ? DAY.format(date) : DAY_YEAR.format(date);
  }

  function select(a: Announcement) {
    view.boardSelected = a.id;
    if (!a.read) onread(a.id);
  }

  /** Up and down arrows move through the list, like in a mail client. */
  function move(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const index = shown.findIndex((a) => a.id === view.boardSelected);
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
      <select bind:value={view.boardCourse} aria-label={t("common.course")}>
        <option value="all">{t("board.allCourses")}</option>
        {#each withPosts as c (c.id)}
          <option value={c.id}>{c.name}</option>
        {/each}
      </select>
      <label>
        <input type="checkbox" class="switch" bind:checked={view.boardUnreadOnly} />
        {t("board.unreadOnly", { count: unread })}
      </label>
      {#if view.boardCourse !== "all"}
        {@const id = view.boardCourse}
        <label>
          <input
            type="checkbox"
            class="switch"
            checked={muted.includes(id)}
            {disabled}
            onchange={(e) => onmute(id, e.currentTarget.checked)}
          />
          {t("board.mute")}
        </label>
      {/if}
    </div>

    {#if announcements.length === 0}
      <p class="empty"><Icon name="inbox" size={22} />{t("board.empty", { button: t("board.checkNow") })}</p>
    {:else if shown.length === 0}
      <p class="empty">{t("board.noMatch")}</p>
    {/if}

    <ul bind:this={list}>
      {#each shown as a (a.id)}
        <li>
          <button
            class="item"
            class:unread={!a.read}
            class:active={a.id === view.boardSelected}
            aria-current={a.id === view.boardSelected ? "true" : undefined}
            data-id={a.id}
            onclick={() => select(a)}
            onkeydown={move}
          >
            <span class="top">
              {#if !a.read}<span class="dot" aria-hidden="true"></span>{/if}
              {#if a.pinned}<Icon name="pin" size={13} class="pin" />{/if}
              <span class="subject ellipsis">{a.subject}</span>
              <span class="date">{shortDate(a.created)}</span>
            </span>
            <span class="course ellipsis">{names.get(a.courseId) ?? ""}</span>
          </button>
        </li>
      {/each}
    </ul>
  </section>

  <article class="reader">
    {#if selected}
      {#key selected.id}
      <div class="post">
      <header>
        <h2>
          {#if selected.pinned}<Icon name="pin" size={17} class="pin" />{/if}{selected.subject}
        </h2>
        <p class="meta">
          <span>{names.get(selected.courseId) ?? ""}</span> · {selected.author} · {FULL.format(
            new Date(selected.created * 1000),
          )}
        </p>
        <button class="link" onclick={() => openExternal(discussionUrl(selected.id))}>
          {t("common.openOnWebeep")}<Icon name="external" size={13} />
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
              <li>
                <Icon name="paperclip" size={14} />
                <span class="ellipsis">{file.filename}</span>
                <span class="size">{formatBytes(file.filesize)}</span>
              </li>
            {/each}
          </ul>
        </div>
      {/if}
      </div>
      {/key}
    {:else if shown.length > 0}
      <p class="placeholder"><Icon name="board" size={28} />{t("board.selectHint")}</p>
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
    background: var(--sidebar);
  }

  .filters {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    border-bottom: 1px solid var(--border);
  }

  .filters label {
    display: flex;
    gap: 10px;
    align-items: center;
    color: var(--muted);
    font-size: var(--fs-sm);
    cursor: pointer;
    user-select: none;
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 32px 20px;
    color: var(--muted);
    text-align: center;
  }

  .list-pane ul {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    list-style: none;
    margin: 0;
    padding: 6px;
  }

  .list-pane li + li {
    margin-top: 2px;
  }

  .item {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 3px;
    width: 100%;
    padding: 9px 12px;
    border: none;
    border-radius: var(--r-md);
    background: none;
    box-shadow: none;
    font-weight: 400;
    text-align: left;
    white-space: normal;
    transition:
      background-color var(--t-out) var(--ease),
      transform var(--t-press) var(--ease);
  }

  .item:hover:not(.active) {
    background: rgba(255, 255, 255, 0.04);
    transition-duration: var(--t-in);
  }

  .item:active {
    transform: scale(0.99);
  }

  .item.active {
    background: var(--selected);
  }

  .top {
    display: flex;
    gap: 7px;
    align-items: center;
    min-width: 0;
  }

  /* Unread: the cobalt dot, and the subject in bold. */
  .dot {
    flex: none;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
  }

  /* Pinned is not a warning: the icon alone marks it. */
  .top :global(.pin) {
    color: var(--muted);
  }

  .subject {
    flex: 1;
  }

  .unread .subject {
    font-weight: 600;
  }

  .date,
  .course {
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .date {
    flex: none;
    font-variant-numeric: tabular-nums;
  }

  .unread .date {
    color: var(--accent);
  }

  .reader {
    min-height: 0;
    overflow-y: auto;
    padding: 24px 32px;
  }

  .post {
    animation: rise-in var(--t-panel) var(--ease-enter);
  }

  .reader header {
    padding-bottom: 14px;
    margin-bottom: 18px;
    border-bottom: 1px solid var(--border);
  }

  .reader h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 0 6px;
    font-size: var(--fs-xl);
    font-weight: 600;
  }

  .reader h2 :global(.pin) {
    color: var(--muted);
  }

  .reader header button.link {
    gap: 5px;
  }

  .meta {
    margin: 0 0 8px;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .message {
    max-width: 75ch;
    font-size: var(--fs-md);
    line-height: 1.6;
    overflow-wrap: anywhere;
  }

  .message :global(p:first-child) {
    margin-top: 0;
  }

  .message :global(a) {
    color: var(--accent);
    text-decoration-color: color-mix(in srgb, var(--accent) 40%, transparent);
    text-underline-offset: 2px;
    transition: text-decoration-color var(--t-out) var(--ease);
  }

  .message :global(a:hover) {
    text-decoration-color: currentColor;
  }

  .message :global(pre) {
    white-space: pre-wrap;
    padding: 10px 12px;
    border-radius: var(--r-sm);
    background: var(--sidebar);
  }

  .attachments {
    max-width: 75ch;
    margin-top: 24px;
  }

  .attachments ul {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  /* Plain lines, not buttons: the files open from the announcement on WeBeep. */
  .attachments li {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    color: var(--muted);
  }

  .attachments li .ellipsis {
    flex: 1;
    color: var(--text);
  }

  .size {
    flex: none;
    font-size: var(--fs-sm);
    font-variant-numeric: tabular-nums;
  }

  .placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    margin-top: 80px;
    color: var(--muted);
    text-align: center;
  }

  .placeholder :global(.icon) {
    opacity: 0.6;
  }
</style>
