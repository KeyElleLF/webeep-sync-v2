<script lang="ts">
  import "$lib/styles.css";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import {
    api,
    ANNOUNCEMENTS_EVENT,
    SYNC_FINISHED_EVENT,
    SYNC_FAILED_EVENT,
    SYNC_STARTED_EVENT,
    SYNC_PROGRESS_EVENT,
    type Announcement,
    type CleanupPreview,
    type CourseLink,
    type CourseRow,
    type CourseSummary,
    type Deadline,
    type ManualState,
    type PassedSuggestion,
    type PlanStatus,
    type AppInfo,
    type SessionInfo,
    type Settings,
    type StateOutcome,
    type SyncProgress,
  } from "$lib/api";
  import { courseTitle } from "$lib/course-title";
  import { formatBytes, isActive, type RowStatus } from "$lib/format";
  import {
    type Lang,
    type MessageKey,
    lang,
    setLang,
    systemLang,
    t,
    tn,
  } from "$lib/i18n/index.svelte";
  import { bugReportUrl, openExternal } from "$lib/links";
  import Board from "$lib/components/Board.svelte";
  import CleanupDialog from "$lib/components/CleanupDialog.svelte";
  import CourseList from "$lib/components/CourseList.svelte";
  import Deadlines from "$lib/components/Deadlines.svelte";
  import Links from "$lib/components/Links.svelte";
  import DevTools from "$lib/components/DevTools.svelte";
  import SupportPanel from "$lib/components/SupportPanel.svelte";
  import UpdateBanner from "$lib/components/UpdateBanner.svelte";
  import InfoTip from "$lib/components/InfoTip.svelte";
  import {
    RECHECK_MS,
    availableUpdate,
    checkForUpdate,
    updateDismissed,
  } from "$lib/updates.svelte";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import StudyPlanDialog, { type Feedback } from "$lib/components/StudyPlanDialog.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Icon, { type IconName } from "$lib/components/Icon.svelte";
  import { installSmoothScroll } from "$lib/smooth-scroll";
  import SyncSummary from "$lib/components/SyncSummary.svelte";

  type Tab = "courses" | "links" | "board" | "deadlines" | "settings";
  const TITLES: Record<Tab, MessageKey> = {
    courses: "nav.courses",
    links: "nav.links",
    board: "nav.board",
    deadlines: "nav.deadlines",
    settings: "nav.settings",
  };
  const ICONS: Record<Tab, IconName> = {
    courses: "courses",
    links: "links",
    board: "board",
    deadlines: "deadlines",
    settings: "settings",
  };
  const SYNC_LABEL: MessageKey = "status.syncing";
  const DONE_MS = 8000;
  const FOLDER_MESSAGES = {
    same: null,
    moved: "courses.folderMoved",
    repointed: "courses.folderRepointed",
  } as const;

  let tab = $state<Tab>("courses");
  let session = $state<SessionInfo | null>(null);
  /** What the app is doing, as a text key (translated when shown, so it follows the language). */
  let busy = $state<MessageKey | null>(null);
  let error = $state<string | null>(null);
  /** The last error, shown until closed: the next action does not wipe it (`error` is per action).
   *  It says what failed, offers to retry when that makes sense, and stays on the tab it came from. */
  let shownError = $state<ShownError | null>(null);
  /** A sync started from the button (an automatic one shows up through `progress` only). */
  let syncRequested = $state(false);
  /** An automatic sync is running (known from its start event, before any progress). */
  let backgroundSync = $state(false);
  let cancelling = $state(false);
  /** "Tutto aggiornato · …" after a sync, for a few seconds in the status bar. */
  let doneMessage = $state<string | null>(null);
  let doneTimer: ReturnType<typeof setTimeout> | undefined;
  /** Outcome of the last sync per course, kept for the course list after the summary is closed. */
  let results = $state<Map<number, RowStatus>>(new Map());
  let notice = $state<string | null>(null);
  let root = $state<string | null>(null);
  let settings = $state<Settings | null>(null);
  let lastSync = $state<number | null>(null);
  let courses = $state<CourseRow[] | null>(null);
  let suggestions = $state<PassedSuggestion[]>([]);
  let cleanups = $state<CleanupPreview[]>([]);
  let cleanupLater = $state(false);
  let summaries = $state<CourseSummary[] | null>(null);
  let progress = $state<SyncProgress | null>(null);
  let announcements = $state<Announcement[]>([]);
  let muted = $state<number[]>([]);
  let deadlines = $state<Deadline[] | null>(null);
  let links = $state<CourseLink[]>([]);
  let planStatus = $state<PlanStatus | null>(null);
  let planOpen = $state(false);
  let planFeedback = $state<Feedback | null>(null);
  let appInfo = $state<AppInfo | null>(null);
  let checkingUpdate = $state(false);
  let updateMessage = $state<string | null>(null);
  const showUpdate = $derived(availableUpdate() !== null && !updateDismissed());

  const idle = $derived(busy === null);
  const syncRunning = $derived(syncRequested || backgroundSync || progress !== null);
  /** Courses, folders and the study plan stay still while a sync uses them; the rest stays usable. */
  const canEdit = $derived(idle && !syncRunning);
  const unread = $derived(announcements.filter((a) => !a.read).length);
  // Links of passed, ignored or past courses stay saved but are not shown.
  const activeNames = $derived(
    new Map((courses ?? []).filter((c) => isActive(c.state)).map((c) => [c.id, courseTitle(c.name)])),
  );
  const visibleLinks = $derived(links.filter((l) => activeNames.has(l.courseId)));
  const newLinks = $derived(visibleLinks.filter((l) => !l.seen).length);
  const upcomingIds = $derived(
    new Set((courses ?? []).filter((c) => c.state === "upcoming").map((c) => c.id)),
  );
  const navItems = $derived(
    (Object.keys(TITLES) as Tab[]).map((id) => ({
      id,
      label: t(TITLES[id]),
      icon: ICONS[id],
      badge: id === "board" ? unread : id === "links" ? newLinks : undefined,
      alert: shownError?.tab === id && tab !== id,
    })),
  );

  interface ShownError {
    text: string;
    title: MessageKey;
    retry: (() => void) | null;
    tab: Tab | null;
  }

  /** What a failed action is called, how to try again and where it belongs. */
  const FAILURES: Partial<Record<MessageKey, Omit<ShownError, "text">>> = {
    "status.syncing": { title: "error.sync", retry: () => void sync(), tab: "courses" },
    "status.checkingBoard": { title: "error.board", retry: () => void refreshBoard(), tab: "board" },
    "status.loadingDeadlines": {
      title: "error.deadlines",
      retry: () => void loadDeadlines(),
      tab: "deadlines",
    },
    "status.loadingCourses": {
      title: "error.courses",
      retry: () => void loadCourses(),
      tab: "courses",
    },
    "status.readingLinks": { title: "error.links", retry: () => void loadLinks(), tab: "links" },
    "status.loggingIn": { title: "error.login", retry: () => void login(), tab: null },
  };

  function fail(label: MessageKey, err: unknown) {
    error = err instanceof Error ? err.message : String(err);
    const known = FAILURES[label];
    shownError = {
      text: error,
      ...(known ?? { title: "error.generic", retry: null, tab: session?.loggedIn ? tab : null }),
    };
  }

  function retry(failure: ShownError) {
    shownError = null;
    failure.retry?.();
  }

  async function run<T>(label: MessageKey, action: () => Promise<T>): Promise<T | undefined> {
    busy = label;
    error = null;
    notice = null;
    try {
      return await action();
    } catch (err) {
      fail(label, err);
      return undefined;
    } finally {
      busy = null;
    }
  }

  type Outcome<T> = { ok: true; value: T } | { ok: false; message: string };

  /** Like `run`, but tells this action's own outcome (the shared `error` may come from another). */
  async function attempt<T>(label: MessageKey, action: () => Promise<T>): Promise<Outcome<T>> {
    let outcome: Outcome<T> = { ok: false, message: "" };
    await run(label, async () => {
      try {
        outcome = { ok: true, value: await action() };
      } catch (err) {
        outcome = { ok: false, message: err instanceof Error ? err.message : String(err) };
        throw err;
      }
    });
    return outcome;
  }

  /** The saved language, or Windows' one on the first run (then saved). */
  async function initLanguage() {
    const saved = await api.getLanguage().catch(() => null);
    const chosen = saved ?? systemLang();
    setLang(chosen);
    if (!saved) void api.setLanguage(chosen).catch(() => undefined);
  }

  async function changeLanguage(next: Lang) {
    setLang(next);
    await run("status.saving", () => api.setLanguage(next));
  }

  async function refresh() {
    await initLanguage();
    appInfo = await api.appInfo().catch(() => null);
    session = (await run("status.checkingSession", api.authStatus)) ?? session;
    if (session?.loggedIn) await loadLibrary();
  }

  async function loadLibrary() {
    root = (await run("status.readingSettings", api.syncRoot)) ?? root;
    settings = (await run("status.readingSettings", api.getSettings)) ?? settings;
    lastSync = (await run("status.readingSettings", api.lastSync)) ?? lastSync;
    announcements = (await run("status.readingBoard", api.listAnnouncements)) ?? announcements;
    await loadLinks();
    planStatus = (await run("status.readingSettings", api.studyPlanStatus)) ?? null;
    muted = (await run("status.readingBoard", api.mutedCourses)) ?? muted;
    await loadCourses();
    cleanups = (await run("status.checkingPassed", api.pendingCleanups)) ?? cleanups;
  }

  async function loadCourses() {
    const listing = await run("status.loadingCourses", api.listCourses);
    if (!listing) return;
    courses = listing.courses;
    suggestions = listing.suggestions;
  }

  async function login() {
    session = (await run("status.loggingIn", api.login)) ?? session;
    if (!session?.loggedIn) return;
    // A failed attempt before this one is no longer news.
    shownError = null;
    await loadLibrary();
  }

  async function logout() {
    await run("status.loggingOut", api.logout);
    session = { loggedIn: false, fullname: null, moodleRelease: null };
    shownError = null;
    results = new Map();
    courses = null;
    summaries = null;
    suggestions = [];
    tab = "courses";
  }

  async function chooseRoot() {
    const chosen = await run("status.choosingFolder", api.chooseSyncRoot);
    if (chosen) {
      root = chosen;
      await loadCourses();
    }
  }

  async function toggle(course: CourseRow, selected: boolean) {
    const update = (value: boolean) =>
      (courses = courses?.map((c) => (c.id === course.id ? { ...c, selected: value } : c)) ?? null);
    update(selected);
    const saved = await attempt("status.savingSelection", () =>
      api.setCourseSelected(course.id, selected),
    );
    if (!saved.ok) update(!selected);
  }

  async function rename(course: CourseRow, folder: string) {
    const change = await run("status.changingFolder", () =>
      api.setCourseFolder(course.id, folder),
    );
    if (change === undefined) return;
    await loadCourses();
    const message = FOLDER_MESSAGES[change];
    notice = message ? t(message) : null;
  }

  async function setState(teachingKey: string, state: ManualState | null) {
    const outcome = await run("status.updatingCourse", () =>
      api.setTeachingState(teachingKey, state),
    );
    if (!outcome) return;
    await loadCourses();
    cleanups = (await run("status.checkingPassed", api.pendingCleanups)) ?? cleanups;
    cleanupLater = false;
    notice = stateNotice(outcome);
  }

  function stateNotice(outcome: StateOutcome): string | null {
    if (outcome.cleaned.length === 0) return null;
    const trashed = outcome.cleaned.reduce((n, r) => n + r.trashed, 0);
    const freed = outcome.cleaned.reduce((n, r) => n + r.freed, 0);
    const kept = outcome.cleaned.reduce((n, r) => n + r.kept, 0);
    const trashedText = tn("cleanup.trashed", trashed, { size: formatBytes(freed) });
    return kept > 0 ? `${trashedText} ${tn("cleanup.keptModified", kept)}` : trashedText;
  }

  async function setSemester(course: CourseRow, semester: 1 | 2 | null) {
    await run("status.savingSemester", () =>
      api.setTeachingSemester(course.teachingKey, semester),
    );
    await loadCourses();
  }

  async function dismiss(suggestion: PassedSuggestion) {
    await run("status.saving", () => api.dismissSuggestion(suggestion.courseId));
    suggestions = suggestions.filter((s) => s.courseId !== suggestion.courseId);
  }

  async function confirmCleanup(preview: CleanupPreview, includeModified: boolean) {
    const report = await run("status.trashing", () =>
      api.confirmCleanup(preview.courseId, includeModified),
    );
    if (!report) return;
    cleanups = cleanups.filter((c) => c.courseId !== preview.courseId);
    const trashed = tn("cleanup.trashed", report.trashed, { size: formatBytes(report.freed) });
    const skipped = report.skipped.length
      ? ` ${tn("cleanup.skipped", report.skipped.length)}`
      : "";
    notice = `${courseTitle(preview.name)}: ${trashed}${skipped}`;
  }

  async function declineCleanup(preview: CleanupPreview) {
    await run("status.saving", () => api.declineCleanup(preview.courseId));
    cleanups = cleanups.filter((c) => c.courseId !== preview.courseId);
  }

  async function saveSettings(next: Settings) {
    settings = (await run("status.savingSettings", () => api.setSettings(next))) ?? settings;
  }

  /** The sync has a state of its own, so the board, links and deadlines stay usable meanwhile. */
  async function sync() {
    summaries = null;
    results = new Map();
    doneMessage = null;
    cancelling = false;
    syncRequested = true;
    try {
      const result = await api.syncNow();
      progress = null;
      await synced(result);
    } catch (err) {
      // Pressed just as an automatic sync started: that one goes on and reports by itself.
      if (backgroundSync) return;
      progress = null;
      fail(SYNC_LABEL, err);
      // A failed sync may mean the session expired: update the account card, keep the error.
      session = (await api.authStatus().catch(() => null)) ?? session;
    } finally {
      syncRequested = false;
      cancelling = false;
    }
  }

  async function cancelSync() {
    cancelling = true;
    await api.cancelSync().catch((err) => fail("status.syncing", err));
  }

  function showDone(message: string) {
    doneMessage = message;
    clearTimeout(doneTimer);
    doneTimer = setTimeout(() => (doneMessage = null), DONE_MS);
  }

  function doneText(result: CourseSummary[]): string {
    if (cancelling) return tn("sync.cancelled", result.length);
    const files = result.reduce((n, s) => n + s.report.downloaded.length + s.report.updated.length, 0);
    return files > 0 ? tn("sync.doneNews", files) : tn("sync.doneNothing", result.length);
  }

  async function synced(result: CourseSummary[]) {
    showDone(doneText(result));
    cancelling = false;
    summaries = result;
    results = new Map(result.map((s) => [s.id, rowStatus(s)]));
    // A sync that went through makes an older error stale.
    if (result.every((s) => !s.error)) shownError = null;
    lastSync = Math.floor(Date.now() / 1000);
    // File dates just learned refine the semester guesses.
    await loadCourses();
    await loadLinks();
  }

  function rowStatus(s: CourseSummary): RowStatus {
    if (s.error) return { kind: "error", message: s.error };
    const failed = s.report.errors.length;
    if (failed > 0) return { kind: "error", message: `${failed} ${tn("sync.errors", failed)}` };
    const files = [...s.report.downloaded, ...s.report.updated];
    return { kind: "done", added: files.length, files };
  }

  async function loadLinks() {
    links = (await run("status.readingLinks", api.listLinks)) ?? links;
  }

  async function openLink(link: CourseLink) {
    const opened = await attempt("status.openingLink", () => openExternal(link.url));
    if (link.seen || !opened.ok) return;
    await markLinksSeen(link.id);
  }

  async function markLinksSeen(id: number | null) {
    const saved = await attempt("status.saving", () => api.markLinksSeen(id));
    if (!saved.ok) return;
    links = links.map((l) => (id === null || l.id === id ? { ...l, seen: true } : l));
  }

  function openPlan() {
    planFeedback = null;
    planOpen = true;
  }

  /** Imports the plan from the picker or from a dropped file, then refreshes the courses. */
  async function importPlan(source: () => Promise<PlanStatus | null>) {
    planFeedback = null;
    const read = await attempt("status.readingPlan", source);
    if (!read.ok) {
      planFeedback = { error: true, text: read.message };
      return;
    }
    const result = read.value;
    if (!result) return;
    planStatus = result;
    await loadCourses();
    const matched = (courses ?? []).filter(
      (c) => c.current && c.semester?.source === "studyPlan",
    ).length;
    const imported = tn("plan.imported", result.teachings, {
      first: result.first,
      second: result.second,
    });
    planFeedback = { error: false, text: `${imported} ${tn("plan.matched", matched)}` };
  }

  async function clearPlan() {
    const removed = await attempt("status.removingPlan", api.clearStudyPlan);
    if (!removed.ok) {
      planFeedback = { error: true, text: removed.message };
      return;
    }
    planStatus = null;
    planFeedback = { error: false, text: t("plan.removed") };
    await loadCourses();
  }

  async function openCourseFolder(courseId: number) {
    await run("status.openingCourseFolder", () => api.openCourseFolder(courseId));
  }

  async function markRead(id: number | null) {
    await run("status.saving", () => api.markAnnouncementsRead(id));
    announcements = announcements.map((a) => (id === null || a.id === id ? { ...a, read: true } : a));
  }

  async function refreshBoard() {
    announcements = (await run("status.checkingBoard", api.refreshAnnouncements)) ?? announcements;
  }

  async function mute(courseId: number, value: boolean) {
    await run("status.saving", () => api.setCourseMuted(courseId, value));
    muted = value ? [...muted, courseId] : muted.filter((id) => id !== courseId);
  }

  async function loadDeadlines() {
    deadlines = (await run("status.loadingDeadlines", api.listDeadlines)) ?? deadlines;
  }

  const TAB_ORDER = Object.keys(TITLES) as Tab[];

  /** Ctrl+R or F5 syncs (instead of reloading the window), Ctrl+1…5 opens a tab. */
  function shortcut(event: KeyboardEvent) {
    if (!session?.loggedIn || planOpen || (cleanups.length > 0 && !cleanupLater)) return;
    const ctrl = event.ctrlKey || event.metaKey;
    const refresh = event.key === "F5" || (ctrl && !event.shiftKey && event.key.toLowerCase() === "r");
    if (refresh) {
      event.preventDefault();
      if (canEdit && root && courses) void sync();
      return;
    }
    const index = ctrl && !event.altKey ? Number(event.key) - 1 : -1;
    if (index >= 0 && index < TAB_ORDER.length) {
      event.preventDefault();
      open(TAB_ORDER[index]);
    }
  }

  function open(next: Tab) {
    tab = next;
    if (next === "deadlines" && deadlines === null) void loadDeadlines();
  }

  /** "Check for updates" in the settings. */
  async function manualUpdateCheck() {
    checkingUpdate = true;
    updateMessage = null;
    try {
      const update = await checkForUpdate();
      updateMessage = update
        ? t("update.available", { version: update.version })
        : t("update.upToDate");
    } catch (err) {
      updateMessage = t("update.failed", { error: err instanceof Error ? err.message : String(err) });
    } finally {
      checkingUpdate = false;
    }
  }

  onMount(() => {
    refresh();
    const removeSmoothScroll = installSmoothScroll();
    // Only the installed app updates itself (a dev build would offer the release over itself).
    let updateTimer: ReturnType<typeof setInterval> | undefined;
    if (import.meta.env.PROD) {
      const quietCheck = () => void checkForUpdate().catch(() => undefined);
      quietCheck();
      updateTimer = setInterval(quietCheck, RECHECK_MS);
    }
    const stops = [
      listen<SyncProgress>(SYNC_PROGRESS_EVENT, (e) => (progress = e.payload)),
      // Automatic syncs (while the window is open) report here too.
      listen(SYNC_STARTED_EVENT, () => {
        if (!syncRequested) backgroundSync = true;
      }),
      // Every sync ends with one of these two, so the UI never stays "syncing".
      listen<{ message: string }>(SYNC_FAILED_EVENT, (e) => {
        progress = null;
        if (syncRequested) return;
        backgroundSync = false;
        cancelling = false;
        fail(SYNC_LABEL, new Error(e.payload.message));
      }),
      listen<CourseSummary[]>(SYNC_FINISHED_EVENT, (e) => {
        progress = null;
        backgroundSync = false;
        // A sync from the button reports through its own call.
        if (!syncRequested) void synced(e.payload);
      }),
      listen<number>(ANNOUNCEMENTS_EVENT, async () => {
        announcements = await api.listAnnouncements().catch(() => announcements);
      }),
    ];
    return () => {
      removeSmoothScroll();
      clearInterval(updateTimer);
      stops.forEach((p) => p.then((stop) => stop()));
    };
  });
</script>

<svelte:window onkeydown={shortcut} />

{#if session?.loggedIn}
  <div class="app">
    <Sidebar
      items={navItems}
      current={tab}
      fullname={session.fullname}
      {lastSync}
      {progress}
      syncing={syncRunning}
      {cancelling}
      done={doneMessage !== null && !syncRunning}
      canSync={canEdit && !!root && !!courses}
      onopen={open}
      onsync={sync}
      oncancel={cancelSync}
    />

    <main>
      <header class="page-head">
        <h1>{t(TITLES[tab])}</h1>
        {#if tab === "links"}
          <InfoTip text={t("links.hint")} label={t("links.infoLabel")} />
        {:else if tab === "deadlines"}
          <InfoTip text={t("deadlines.hint")} label={t("deadlines.infoLabel")} />
        {/if}
        <div class="actions">
          {#if tab === "courses"}
            <span class="root" class:empty={!root} title={root ?? undefined}>
              <Icon name="folder" size={14} />
              <span class="ellipsis">{root ?? t("courses.noRoot")}</span>
            </span>
            <button onclick={chooseRoot} disabled={!canEdit}>
              {root ? t("common.changeFolder") : t("common.chooseFolder")}
            </button>
          {:else if tab === "links"}
            <button disabled={!idle || newLinks === 0} onclick={() => markLinksSeen(null)}>
              {t("links.markAllSeen")}
            </button>
          {:else if tab === "board"}
            <button disabled={!idle} onclick={refreshBoard}>{t("board.checkNow")}</button>
            <button disabled={!idle || unread === 0} onclick={() => markRead(null)}>
              {t("board.markAllRead")}
            </button>
          {:else if tab === "deadlines"}
            <button disabled={!idle} onclick={loadDeadlines}>{t("board.checkNow")}</button>
          {/if}
        </div>
      </header>

      {#if shownError && (shownError.tab === null || shownError.tab === tab)}
        {@const failure = shownError}
        <div class="error-banner" role="alert">
          <Icon name="alert" size={18} />
          <p><strong>{t(failure.title)}.</strong> {failure.text}</p>
          {#if failure.retry}
            <button
              class="retry"
              disabled={!canEdit && failure.tab === "courses"}
              onclick={() => retry(failure)}
              >{t("common.retry")}</button
            >
          {/if}
          {#if appInfo}
            {@const info = appInfo}
            <button class="link" onclick={() => openExternal(bugReportUrl(info, failure.text))}>
              {t("support.reportShort")}
            </button>
          {/if}
          <button class="link" onclick={() => (shownError = error = null)}>{t("common.close")}</button>
        </div>
      {/if}

      {#if suggestions.length > 0 || showUpdate}
        <div class="banners">
          <UpdateBanner />
          {#each suggestions as s (s.courseId)}
            <section class="card suggestion">
              <p>
                {t("courses.suggestion.before")}
                <strong>{courseTitle(s.name)}</strong>{t("courses.suggestion.after")}
              </p>
              <div class="buttons">
                <button
                  class="primary"
                  disabled={!canEdit}
                  onclick={() => setState(s.teachingKey, "passed")}
                  >{t("courses.suggestion.yes")}</button
                >
                <button disabled={!canEdit} onclick={() => dismiss(s)}>{t("common.no")}</button>
              </div>
            </section>
          {/each}
        </div>
      {/if}

      <!-- Re-created on every tab: each page opens at its top (no entrance animation). -->
      {#key tab}
      <div class="content" class:fill={tab === "board"}>
        {#if tab === "courses"}
          {#if !root || !lastSync}
            <Onboarding loggedIn={true} hasRoot={!!root} hasSynced={!!lastSync} />
          {/if}
          {#if summaries}
            <SyncSummary
              {summaries}
              onclose={() => (summaries = null)}
              onfolder={openCourseFolder}
            />
          {/if}
          {#if courses}
            <CourseList
              {courses}
              disabled={!canEdit}
              ontoggle={toggle}
              onrename={rename}
              onstate={(course, state) => setState(course.teachingKey, state)}
              onsemester={setSemester}
              planLoaded={planStatus !== null}
              onplan={openPlan}
              {results}
              syncingName={progress?.name ?? null}
              onfolder={openCourseFolder}
            />
          {:else}
            <p class="muted">{t("courses.notLoaded")}</p>
          {/if}
        {:else if tab === "links"}
          <Links
            links={visibleLinks}
            courseNames={activeNames}
            upcoming={upcomingIds}
            disabled={!idle}
            onopen={openLink}
            onfolder={openCourseFolder}
          />
        {:else if tab === "board"}
          <Board
            {announcements}
            courses={courses ?? []}
            {muted}
            disabled={!idle}
            onread={markRead}
            onmute={mute}
          />
        {:else if tab === "deadlines"}
          <Deadlines {deadlines} />
        {:else if tab === "settings"}
          <div class="settings">
            <section class="card">
              <h2>{t("settings.account")}</h2>
              <p>{t("settings.loggedInAs")} <strong>{session.fullname}</strong>.</p>
              <button onclick={logout} disabled={!canEdit}>{t("settings.logout")}</button>
            </section>
            <section class="card">
              <h2>{t("settings.folder")}</h2>
              <p>
                {#if root}<code>{root}</code>{:else}{t("settings.folderPrompt")}{/if}
              </p>
              <p class="muted">{t("settings.folderHint")}</p>
              <button onclick={chooseRoot} disabled={!canEdit}>
                {root ? t("common.changeFolder") : t("common.chooseFolder")}
              </button>
            </section>
            {#if settings}
              <section class="card">
                <h2>{t("settings.syncAndNotifications")}</h2>
                <SettingsPanel
                  {settings}
                  disabled={!idle}
                  onchange={saveSettings}
                />
              </section>
            {/if}
            <section class="card">
              <h2>Lingua / Language</h2>
              <select
                value={lang()}
                aria-label="Lingua / Language"
                onchange={(e) => changeLanguage(e.currentTarget.value as Lang)}
              >
                <option value="it">Italiano</option>
                <option value="en">English</option>
              </select>
            </section>
            <section class="card">
              <h2>{t("update.title")}</h2>
              <p>{t("update.current", { version: appInfo?.version ?? "?" })}</p>
              <button disabled={checkingUpdate} onclick={manualUpdateCheck}>
                {t("update.check")}
              </button>
              {#if updateMessage}
                <p class="muted" role="status">{updateMessage}</p>
              {/if}
            </section>
            <section class="card">
              <h2>{t("support.title")}</h2>
              <SupportPanel info={appInfo} disabled={!idle} />
              <p class="muted unofficial">{t("common.unofficial")}</p>
            </section>
            {#if import.meta.env.DEV}
              <!-- Phase 0 dump: development only (the command also refuses in release builds). -->
              <DevTools disabled={!idle} {run} />
            {/if}
          </div>
        {/if}
      </div>
      {/key}

      <footer class="status">
        {#if busy}
          <span class="busy spinner" aria-hidden="true"></span>
          <p class="busy muted" role="status">{t(busy)}</p>
        {:else if syncRunning}
          <span class="spinner" aria-hidden="true"></span>
          <p class="muted" role="status">{cancelling ? t("sync.cancelling") : t(SYNC_LABEL)}</p>
        {:else if notice}
          <p class="notice" role="status">{notice}</p>
          <button class="link" onclick={() => (notice = null)}>{t("common.close")}</button>
        {:else if doneMessage}
          <!-- The confident end of a sync: what happened, then it fades away. -->
          <p class="done" role="status"><Icon name="check" size={14} />{doneMessage}</p>
        {/if}
      </footer>
    </main>
  </div>

  {#if planOpen}
    <StudyPlanDialog
      status={planStatus}
      feedback={planFeedback}
      busy={!canEdit}
      onchoose={() => importPlan(api.chooseStudyPlan)}
      ondrop={(path) => importPlan(() => api.importStudyPlan(path))}
      onpaste={(text) => importPlan(() => api.importStudyPlanText(text))}
      onclear={clearPlan}
      onclose={() => (planOpen = false)}
    />
  {/if}

  {#if cleanups.length > 0 && !cleanupLater && !planOpen}
    <CleanupDialog
      previews={cleanups}
      disabled={!canEdit}
      onconfirm={confirmCleanup}
      ondecline={declineCleanup}
      onlater={() => (cleanupLater = true)}
    />
  {/if}
{:else}
  <div class="welcome">
    <div class="welcome-box">
      <img class="logo" src="/logo.svg" alt="" width="72" height="72" />
      <h1>WeBeep Sync</h1>
      <p class="muted">{t("welcome.tagline")}</p>
      {#if session}
        <Onboarding loggedIn={false} hasRoot={!!root} hasSynced={!!lastSync} />
        <button class="primary big" onclick={login} disabled={!idle}>{t("welcome.login")}</button>
      {/if}
      {#if busy}
        <p class="working muted" role="status">
          <span class="spinner" aria-hidden="true"></span>{t(busy)}
        </p>
      {/if}
      {#if shownError}
        <p class="error" role="alert">{shownError.text}</p>
      {/if}
      <p class="languages">
        <button class="link" class:current={lang() === "it"} onclick={() => changeLanguage("it")}
          >Italiano</button
        >
        ·
        <button class="link" class:current={lang() === "en"} onclick={() => changeLanguage("en")}
          >English</button
        >
      </p>
      <p class="muted small">{t("common.unofficial")}</p>
    </div>
  </div>
{/if}

<style>
  .app {
    display: grid;
    grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
    height: 100vh;
  }

  main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .page-head {
    display: flex;
    align-items: center;
    gap: 16px;
    /* The same height on every tab, with or without buttons. */
    min-height: 56px;
    box-sizing: border-box;
    padding: 0 24px;
    border-bottom: 1px solid var(--border);
    user-select: none;
  }

  .page-head h1 {
    flex: none;
    margin: 0;
    font-size: var(--fs-xl);
    font-weight: 600;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    margin-left: auto;
  }

  .root {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
    max-width: 420px;
    padding: 5px 10px;
    border-radius: var(--r-sm);
    background: rgba(255, 255, 255, 0.04);
    color: var(--muted);
    font-size: var(--fs-sm);
    user-select: text;
  }

  .root.empty {
    color: var(--warn);
  }

  /* An error stays at the top of the page until closed, with a way to report it. */
  .error-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 16px 24px 0;
    padding: 10px 14px;
    border: 1px solid color-mix(in srgb, var(--danger) 45%, transparent);
    border-radius: var(--r-md);
    background: color-mix(in srgb, var(--danger) 10%, var(--card));
    color: var(--danger);
    animation: rise-in var(--t-panel) var(--ease-enter);
  }

  .error-banner .retry {
    flex: none;
    padding: 4px 12px;
    font-size: var(--fs-sm);
  }

  .error-banner p {
    flex: 1;
    margin: 0;
    color: var(--text);
    overflow-wrap: anywhere;
  }

  .banners {
    padding: 16px 24px 0;
    animation: rise-in var(--t-panel) var(--ease-enter);
  }

  .suggestion {
    border-color: var(--accent);
    background: linear-gradient(var(--selected), var(--selected)), var(--card);
  }

  .suggestion p {
    margin-top: 0;
  }

  .buttons {
    display: flex;
    gap: 8px;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 20px 24px 28px;
  }

  /* Pages with their own scrolling panes (the board). */
  .content.fill {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0;
  }

  .settings {
    max-width: 760px;
  }

  .settings .card p {
    max-width: 66ch;
  }

  .settings .card > :global(button) {
    margin-top: 4px;
  }

  .unofficial {
    margin: 14px 0 0;
    font-size: var(--fs-sm);
  }

  .status {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 30px;
    padding: 0 24px;
    background: var(--sidebar);
    border-top: 1px solid var(--border);
    font-size: var(--fs-sm);
    user-select: none;
  }

  .status p {
    flex: 1;
    margin: 0;
    animation: fade-in var(--t-in) var(--ease);
  }

  /* Quick actions finish before their message would flash: it appears only after 150 ms. */
  .status .busy {
    animation: fade-in var(--t-in) var(--ease) 150ms both;
  }

  .notice {
    color: var(--ok);
  }

  .done {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--ok);
  }

  .error {
    color: var(--danger);
  }

  .welcome {
    display: grid;
    place-items: center;
    box-sizing: border-box;
    height: 100vh;
    padding: 24px;
    overflow-y: auto;
    background: var(--bg);
  }

  .welcome-box {
    width: min(520px, 100%);
    text-align: center;
  }

  /* The welcome settles in once, as a whole. */
  .welcome-box {
    animation: rise-in var(--t-panel) var(--ease-enter) both;
  }

  .welcome-box h1 {
    margin: 14px 0 4px;
    font-size: var(--fs-display);
    font-weight: 600;
  }

  .welcome-box :global(.card) {
    margin-top: 24px;
    text-align: left;
  }

  .working {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .languages {
    margin-top: 22px;
    color: var(--muted);
  }

  .languages .current {
    color: var(--text);
    font-weight: 600;
    background-size: 100% 1px;
  }

  button.big {
    padding: 11px 30px;
    font-size: var(--fs-lg);
  }

  .small {
    font-size: var(--fs-sm);
    margin-top: 28px;
  }
</style>
