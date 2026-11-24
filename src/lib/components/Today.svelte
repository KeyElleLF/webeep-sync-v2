<script lang="ts">
  // "Oggi": what happened in the last days, day by day (files of each sync, announcements, new
  // links), with the deadlines always in view on the right. What arrived after the last time the
  // app was opened is marked as new; nothing has to be cleared by hand.
  import type { Announcement, CourseLink, CourseRow, Deadline, SyncRecord } from "$lib/api";
  import { courseTitle } from "$lib/course-title";
  import { displayName, locale, t, tn } from "$lib/i18n/index.svelte";
  import { openExternal } from "$lib/links";
  import Icon, { type IconName } from "./Icon.svelte";

  interface Props {
    history: SyncRecord[];
    announcements: Announcement[];
    links: CourseLink[];
    deadlines: Deadline[] | null;
    courses: CourseRow[];
    /** When the app was opened before this time (Unix seconds); null the first time. */
    lastOpened: number | null;
    onreveal: (courseId: number, path: string) => void;
    onfolder: (courseId: number) => void;
    onannouncement: (id: number) => void;
    onlink: (link: CourseLink) => void;
    ondeadlines: () => void;
  }

  let {
    history,
    announcements,
    links,
    deadlines,
    courses,
    lastOpened,
    onreveal,
    onfolder,
    onannouncement,
    onlink,
    ondeadlines,
  }: Props = $props();

  /** Days of the diary. */
  const DAYS = 7;
  const DAY = 86_400;
  const HOUR = 3_600;
  /** Files listed per course before "and N more". */
  const MAX_FILES = 6;
  const CONFLICT = / \(WeBeep aggiornato \d{4}-\d{2}-\d{2}\)(?=\.[^./]+$|$)/;

  interface FileItem {
    path: string;
    name: string;
    where: string;
    conflict: boolean;
  }
  type Entry =
    | { kind: "sync"; time: number; files: number; courses: { id: number; files: FileItem[] }[] }
    | { kind: "announcement"; time: number; item: Announcement }
    | { kind: "link"; time: number; item: CourseLink };

  const names = $derived(new Map(courses.map((c) => [c.id, courseTitle(c.name)])));
  const since = $derived(Math.floor(Date.now() / 1000) - DAYS * DAY);

  function fileItem(path: string, conflict: boolean): FileItem {
    const parts = path.split("/");
    return { path, name: parts.at(-1) ?? path, where: parts.slice(0, -1).join(" › "), conflict };
  }

  function syncEntry(record: SyncRecord): Entry {
    const byCourse = new Map<number, FileItem[]>();
    for (const f of record.files) {
      const list = byCourse.get(f.courseId) ?? [];
      list.push(fileItem(f.path, f.change === "conflict"));
      byCourse.set(f.courseId, list);
    }
    return {
      kind: "sync",
      time: record.finished,
      files: record.files.length,
      courses: [...byCourse].map(([id, files]) => ({ id, files })),
    };
  }

  /** Links added by the teacher: not the ones read at the first sync of their course. */
  const newLinks = $derived.by(() => {
    const first = new Map<number, number>();
    for (const l of links) first.set(l.courseId, Math.min(first.get(l.courseId) ?? l.firstSeen, l.firstSeen));
    return links.filter((l) => l.firstSeen >= since && (!l.seen || l.firstSeen > (first.get(l.courseId) ?? 0)));
  });

  const entries = $derived(
    [
      ...history.filter((r) => r.finished >= since).map(syncEntry),
      ...announcements
        .filter((a) => a.created >= since)
        .map((item): Entry => ({ kind: "announcement", time: item.created, item })),
      ...newLinks.map((item): Entry => ({ kind: "link", time: item.firstSeen, item })),
    ].sort((a, b) => b.time - a.time),
  );
  const hasNew = $derived(lastOpened !== null && entries.some((e) => e.time > lastOpened!));
  /** Index of the first entry already seen: the "Già visto" line goes before it. */
  const seenFrom = $derived(
    lastOpened === null || !hasNew ? -1 : entries.findIndex((e) => e.time <= lastOpened!),
  );

  const days = $derived.by(() => {
    const out: { key: string; title: string; date: string; items: { entry: Entry; index: number }[] }[] = [];
    entries.forEach((entry, index) => {
      const key = new Date(entry.time * 1000).toDateString();
      let day = out.at(-1);
      if (!day || day.key !== key) {
        day = { key, ...dayTitle(entry.time), items: [] };
        out.push(day);
      }
      day.items.push({ entry, index });
    });
    return out;
  });

  const TIME = $derived(new Intl.DateTimeFormat(locale(), { hour: "numeric", minute: "2-digit" }));
  const LONG_DATE = $derived(
    new Intl.DateTimeFormat(locale(), { weekday: "long", day: "numeric", month: "long" }),
  );
  const WEEKDAY = $derived(new Intl.DateTimeFormat(locale(), { weekday: "long", day: "numeric" }));
  const DAY_MONTH = $derived(new Intl.DateTimeFormat(locale(), { day: "numeric", month: "long" }));
  const RELATIVE = $derived(new Intl.RelativeTimeFormat(locale(), { numeric: "auto" }));

  function startOfDay(offset = 0): number {
    const now = new Date();
    return new Date(now.getFullYear(), now.getMonth(), now.getDate() + offset).getTime() / 1000;
  }

  function capitalize(text: string): string {
    return text.charAt(0).toUpperCase() + text.slice(1);
  }

  function dayTitle(time: number): { title: string; date: string } {
    const date = LONG_DATE.format(new Date(time * 1000));
    if (time >= startOfDay()) return { title: t("today.today"), date };
    if (time >= startOfDay(-1)) return { title: t("today.yesterday"), date };
    return { title: capitalize(date), date: "" };
  }

  /** "oggi alle 8:55", "ieri alle 21:10", "lunedì 23 novembre alle 21:10". */
  function when(time: number): string {
    const at = TIME.format(new Date(time * 1000));
    const day =
      time >= startOfDay()
        ? t("today.today").toLowerCase()
        : time >= startOfDay(-1)
          ? t("today.yesterday").toLowerCase()
          : LONG_DATE.format(new Date(time * 1000));
    return t("today.atTime", { day, time: at });
  }

  /** The first lines of an announcement, as plain text. */
  function excerpt(html: string): string {
    // A space where a paragraph or a line ends, so the words of two paragraphs do not touch.
    const spaced = html.replace(/<(\/p|\/div|\/li|br)\b/gi, " <$1");
    const text = new DOMParser().parseFromString(spaced, "text/html").body.textContent ?? "";
    return text.replace(/\s+/g, " ").trim();
  }

  function host(url: string): string {
    try {
      return new URL(url).hostname;
    } catch {
      return "";
    }
  }

  /** How close a deadline is: "Scaduta · 3 ore fa", "Tra 5 ore · 14:00", "Domani · 11:00". */
  function due(d: Deadline): { text: string; tone: "danger" | "warn" | "" } {
    const at = d.due;
    const now = Date.now() / 1000;
    const time = TIME.format(new Date(at * 1000));
    if (d.overdue || at < now) {
      return {
        text: `${t("deadlines.overdue")} · ${RELATIVE.format(Math.round((at - now) / HOUR), "hour")}`,
        tone: "danger",
      };
    }
    if (at - now < DAY) {
      return {
        text: `${capitalize(RELATIVE.format(Math.max(1, Math.round((at - now) / HOUR)), "hour"))} · ${time}`,
        tone: "warn",
      };
    }
    if (at < startOfDay(2)) return { text: `${t("today.tomorrow")} · ${time}`, tone: "" };
    if (at < startOfDay(7)) return { text: capitalize(WEEKDAY.format(new Date(at * 1000))), tone: "" };
    return { text: DAY_MONTH.format(new Date(at * 1000)), tone: "" };
  }

  function icon(entry: Entry): IconName {
    return entry.kind === "sync" ? "sync" : entry.kind === "link" ? "links" : "board";
  }

  function courseName(id: number): string {
    return names.get(id) ?? t("common.courseFallback", { id });
  }
</script>

<div class="cols">
  <section class="feed" aria-label={t("today.feedLabel")}>
    {#if lastOpened !== null && !hasNew && entries.length > 0}
      <p class="seen" role="separator">{t("today.nothingNew", { when: when(lastOpened) })}</p>
    {/if}
    {#if entries.length === 0}
      <p class="empty"><Icon name="check" size={18} />{t("today.empty")}</p>
    {/if}

    {#each days as day (day.key)}
      <div class="day">
        <h2>{day.title}</h2>
        {#if day.date}<span class="meta">{day.date}</span>{/if}
      </div>
      {#each day.items as { entry, index } (index)}
        {@const fresh = lastOpened !== null && entry.time > lastOpened}
        {#if index === seenFrom && lastOpened !== null}
          <p class="seen" role="separator">{t("today.seen", { when: when(lastOpened) })}</p>
        {/if}
        <article class="entry" class:fresh>
          <Icon name={icon(entry)} size={16} class="kind" />
          <div class="head">
            {#if fresh}
              <span class="new-dot" aria-hidden="true"></span>
              <span class="sr-only">{t("links.new")}</span>
            {/if}
            {#if entry.kind === "sync"}
              <h3>{tn("today.files", entry.files)} {tn("today.inCourses", entry.courses.length)}</h3>
            {:else if entry.kind === "announcement"}
              <h3>{entry.item.subject}</h3>
            {:else}
              <h3>{displayName(entry.item.name)}</h3>
            {/if}
          </div>
          <time datetime={new Date(entry.time * 1000).toISOString()}
            >{TIME.format(new Date(entry.time * 1000))}</time
          >

          <div class="body">
            {#if entry.kind === "sync"}
              {#each entry.courses as course (course.id)}
                <div class="course">
                  <span class="course-title">{courseName(course.id)}</span>
                  <button class="act" onclick={() => onfolder(course.id)}>
                    <Icon name="folder" size={14} />{t("today.openFolder")}
                  </button>
                </div>
                <ul class="files">
                  {#each course.files.slice(0, MAX_FILES) as file (file.path)}
                    <li>
                      <span class="name ellipsis" title={file.path}>{file.name}</span>
                      <span class="meta ellipsis">{file.where}</span>
                      <button
                        class="act"
                        aria-label={t("today.showOf", { name: file.name })}
                        onclick={() => onreveal(course.id, file.path)}
                      >
                        <Icon name="file" size={14} />{t("today.show")}
                      </button>
                    </li>
                  {/each}
                  {#if course.files.length > MAX_FILES}
                    <li class="more">
                      {t("courses.moreFiles", { count: course.files.length - MAX_FILES })}
                    </li>
                  {/if}
                </ul>
                {#each course.files.filter((f) => f.conflict) as file (file.path)}
                  <p class="conflict">
                    <Icon name="alert" size={14} />
                    <span>{t("today.conflict", { name: file.name.replace(CONFLICT, "") })}</span>
                  </p>
                {/each}
              {/each}
            {:else if entry.kind === "announcement"}
              <p class="meta">{courseName(entry.item.courseId)} · {entry.item.author}</p>
              <p class="excerpt">{excerpt(entry.item.message)}</p>
              <button class="act" onclick={() => onannouncement(entry.item.id)}>
                <Icon name="board" size={14} />{t("today.readOnBoard")}
              </button>
            {:else}
              <p class="meta">
                {courseName(entry.item.courseId)} · {t("today.newLink")} · {host(entry.item.url)}
              </p>
              <button class="act" onclick={() => onlink(entry.item)}>
                <Icon name="external" size={14} />{t("today.openInBrowser")}
              </button>
            {/if}
          </div>
        </article>
      {/each}
    {/each}
  </section>

  <section class="rail" aria-labelledby="today-deadlines">
    <h2 id="today-deadlines"><Icon name="deadlines" size={16} />{t("today.deadlines")}</h2>
    {#if deadlines === null}
      <p class="meta">{t("today.deadlinesLoading")}</p>
    {:else if deadlines.length === 0}
      <p class="meta">{t("deadlines.empty")}</p>
    {:else}
      <ul>
        {#each deadlines as d (d.id)}
          {@const status = due(d)}
          <li>
            <button
              class="deadline"
              title={d.url ? t("common.openOnWebeep") : undefined}
              disabled={!d.url}
              onclick={() => openExternal(d.url)}
            >
              <span class="due {status.tone}">{status.text}</span>
              <span class="what ellipsis">{d.name}</span>
              <span class="meta ellipsis">{courseTitle(d.courseName)}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
    <button class="more-link" onclick={ondeadlines}>{t("today.allDeadlines")}</button>
  </section>
</div>

<style>
  .cols {
    flex: 1;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 300px;
    min-height: 0;
  }

  @media (max-width: 1050px) {
    .cols {
      grid-template-columns: minmax(0, 1fr) 248px;
    }
  }

  .feed {
    min-height: 0;
    overflow-y: auto;
    padding: 4px 32px 28px;
  }

  @media (max-width: 1050px) {
    .feed {
      padding-inline: 20px;
    }
  }

  .day {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 18px 0 8px;
  }

  .day h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  .meta {
    margin: 0;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .empty {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 20px;
    color: var(--muted);
  }

  .entry {
    display: grid;
    grid-template-columns: 16px minmax(0, 1fr) auto;
    gap: 2px 14px;
    padding: 12px 0;
    border-top: 1px solid var(--border);
  }

  .entry :global(.kind) {
    margin-top: 2px;
    color: var(--muted);
  }

  .entry.fresh :global(.kind) {
    color: var(--accent);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  h3 {
    margin: 0;
    font-size: var(--fs-md);
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  time {
    color: var(--muted);
    font-size: var(--fs-sm);
    font-variant-numeric: tabular-nums;
  }

  .body {
    grid-column: 2 / -1;
    min-width: 0;
  }

  .course {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 8px 0 2px;
  }

  .course-title {
    font-weight: 500;
  }

  .course .act {
    margin-left: auto;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .files li {
    display: grid;
    grid-template-columns: minmax(0, auto) minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    margin: 0 -8px;
    padding: 3px 8px;
    border-radius: var(--r-sm);
    transition: background-color var(--t-out) var(--ease);
  }

  .files li:hover {
    background: var(--hover);
    transition-duration: var(--t-in);
  }

  .files .more {
    display: block;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  /* A quiet action on each item: grey at rest, clearer under the pointer. */
  .act {
    gap: 6px;
    padding: 2px 6px;
    border: none;
    border-radius: var(--r-sm);
    background: none;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .act:hover:not(:disabled),
  .files li:hover .act {
    background: var(--raised);
    color: var(--text);
  }

  .excerpt {
    max-width: 66ch;
    margin: 2px 0 6px;
    color: var(--muted);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .body > .act {
    margin-left: -6px;
  }

  .conflict {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    margin: 6px 0 0;
    padding: 8px 12px;
    border: 1px solid color-mix(in srgb, var(--warn) 45%, transparent);
    border-radius: var(--r-md);
    font-size: var(--fs-sm);
  }

  .conflict :global(.icon) {
    margin-top: 2px;
    color: var(--warn);
  }

  /* Where the new things end: one line in the "new" colour. */
  .seen {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 6px 0 2px;
    color: var(--accent);
    font-size: var(--fs-sm);
  }

  .seen::before,
  .seen::after {
    content: "";
    flex: 1;
    height: 1px;
    background: color-mix(in srgb, var(--accent) 45%, transparent);
  }

  .seen::before {
    flex: 0 0 16px;
  }

  .rail {
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow-y: auto;
    padding: 18px 20px 14px;
    border-left: 1px solid var(--border);
    background: var(--sidebar);
  }

  .rail h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 0 10px;
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  .rail li + li {
    border-top: 1px solid var(--border);
  }

  .deadline {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 1px;
    width: calc(100% + 20px);
    margin: 0 -10px;
    padding: 9px 10px;
    border: none;
    border-radius: var(--r-md);
    background: none;
    color: var(--text);
    text-align: left;
    white-space: normal;
  }

  .deadline:hover:not(:disabled) {
    background: var(--hover);
  }

  .deadline:active:not(:disabled) {
    transform: none;
  }

  .due {
    font-size: var(--fs-sm);
    font-weight: 600;
  }

  .due.warn {
    color: var(--warn);
  }

  .due.danger {
    color: var(--danger);
  }

  .what {
    font-weight: 500;
  }

  .more-link {
    align-self: flex-start;
    margin-top: auto;
    padding: 10px 0 0;
    border: none;
    background: none;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .more-link:hover:not(:disabled) {
    background: none;
    color: var(--text);
  }
</style>
