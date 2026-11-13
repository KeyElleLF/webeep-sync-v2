<script lang="ts">
  import "$lib/styles.css";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import {
    api,
    ANNOUNCEMENTS_EVENT,
    SYNC_FINISHED_EVENT,
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
  import { formatBytes, isActive } from "$lib/format";
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
  import SyncSummary from "$lib/components/SyncSummary.svelte";

  type Tab = "courses" | "links" | "board" | "deadlines" | "settings";
  const TITLES: Record<Tab, MessageKey> = {
    courses: "nav.courses",
    links: "nav.links",
    board: "nav.board",
    deadlines: "nav.deadlines",
    settings: "nav.settings",
  };
  const SYNC_LABEL: MessageKey = "status.syncing";
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
  const unread = $derived(announcements.filter((a) => !a.read).length);
  // Links of passed, ignored or past courses stay saved but are not shown.
  const activeNames = $derived(
    new Map((courses ?? []).filter((c) => isActive(c.state)).map((c) => [c.id, c.name])),
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
      badge: id === "board" ? unread : id === "links" ? newLinks : undefined,
    })),
  );

  async function run<T>(label: MessageKey, action: () => Promise<T>): Promise<T | undefined> {
    busy = label;
    error = null;
    notice = null;
    try {
      return await action();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      return undefined;
    } finally {
      busy = null;
    }
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
    if (session?.loggedIn) await loadLibrary();
  }

  async function logout() {
    await run("status.loggingOut", api.logout);
    session = { loggedIn: false, fullname: null, moodleRelease: null };
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
    await run("status.savingSelection", () => api.setCourseSelected(course.id, selected));
    if (error) update(!selected);
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
    notice = `${preview.name}: ${trashed}${skipped}`;
  }

  async function declineCleanup(preview: CleanupPreview) {
    await run("status.saving", () => api.declineCleanup(preview.courseId));
    cleanups = cleanups.filter((c) => c.courseId !== preview.courseId);
  }

  async function saveSettings(next: Settings) {
    settings = (await run("status.savingSettings", () => api.setSettings(next))) ?? settings;
  }

  async function sync() {
    summaries = null;
    const result = await run(SYNC_LABEL, api.syncNow);
    progress = null;
    if (result) {
      await synced(result);
      return;
    }
    // A failed sync may mean the session expired: update the account card, keep the error.
    try {
      session = await api.authStatus();
    } catch {
      // The error of the sync is the one worth showing.
    }
  }

  async function synced(result: CourseSummary[]) {
    summaries = result;
    lastSync = Math.floor(Date.now() / 1000);
    // File dates just learned refine the semester guesses.
    await loadCourses();
    await loadLinks();
  }

  async function loadLinks() {
    links = (await run("status.readingLinks", api.listLinks)) ?? links;
  }

  async function openLink(link: CourseLink) {
    await run("status.openingLink", () => openExternal(link.url));
    if (link.seen || error) return;
    await markLinksSeen(link.id);
  }

  async function markLinksSeen(id: number | null) {
    await run("status.saving", () => api.markLinksSeen(id));
    if (error) return;
    links = links.map((l) => (id === null || l.id === id ? { ...l, seen: true } : l));
  }

  function openPlan() {
    planFeedback = null;
    planOpen = true;
  }

  /** Imports the plan from the picker or from a dropped file, then refreshes the courses. */
  async function importPlan(source: () => Promise<PlanStatus | null>) {
    planFeedback = null;
    const result = await run("status.readingPlan", source);
    if (error) {
      planFeedback = { error: true, text: error };
      return;
    }
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
    await run("status.removingPlan", api.clearStudyPlan);
    if (error) {
      planFeedback = { error: true, text: error };
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
      listen<CourseSummary[]>(SYNC_FINISHED_EVENT, (e) => {
        progress = null;
        if (idle) void synced(e.payload);
      }),
      listen<number>(ANNOUNCEMENTS_EVENT, async () => {
        announcements = await api.listAnnouncements().catch(() => announcements);
      }),
    ];
    return () => {
      clearInterval(updateTimer);
      stops.forEach((p) => p.then((stop) => stop()));
    };
  });
</script>

{#if session?.loggedIn}
  <div class="app">
    <Sidebar
      items={navItems}
      current={tab}
      fullname={session.fullname}
      {lastSync}
      {progress}
      syncing={busy === SYNC_LABEL || progress !== null}
      canSync={idle && !!root && !!courses}
      onopen={open}
      onsync={sync}
    />

    <main>
      <header class="page-head">
        <h1>{t(TITLES[tab])}</h1>
        <div class="actions">
          {#if tab === "courses"}
            <span class="root ellipsis" title={root ?? undefined}>
              {root ?? t("courses.noRoot")}
            </span>
            <button onclick={chooseRoot} disabled={!idle}>
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
            <button disabled={!idle} onclick={loadDeadlines}>{t("deadlines.refresh")}</button>
          {/if}
        </div>
      </header>

      {#if suggestions.length > 0 || showUpdate}
        <div class="banners">
          <UpdateBanner />
          {#each suggestions as s (s.courseId)}
            <section class="card suggestion">
              <p>
                {t("courses.suggestion.before")}
                <strong>{s.name}</strong>{t("courses.suggestion.after")}
              </p>
              <div class="buttons">
                <button
                  class="primary"
                  disabled={!idle}
                  onclick={() => setState(s.teachingKey, "passed")}
                  >{t("courses.suggestion.yes")}</button
                >
                <button disabled={!idle} onclick={() => dismiss(s)}>{t("common.no")}</button>
              </div>
            </section>
          {/each}
        </div>
      {/if}

      <div class="content" class:fill={tab === "board"}>
        {#if tab === "courses"}
          {#if !root || !lastSync}
            <Onboarding loggedIn={true} hasRoot={!!root} hasSynced={!!lastSync} />
          {/if}
          {#if summaries}
            <section class="card">
              <div class="card-head">
                <h2>{t("sync.lastSync")}</h2>
                <button class="link" onclick={() => (summaries = null)}>{t("common.close")}</button>
              </div>
              <SyncSummary {summaries} />
            </section>
          {/if}
          {#if courses}
            <CourseList
              {courses}
              disabled={!idle}
              ontoggle={toggle}
              onrename={rename}
              onstate={(course, state) => setState(course.teachingKey, state)}
              onsemester={setSemester}
              planLoaded={planStatus !== null}
              onplan={openPlan}
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
              <button onclick={logout} disabled={!idle}>{t("settings.logout")}</button>
            </section>
            <section class="card">
              <h2>{t("settings.folder")}</h2>
              <p>
                {#if root}<code>{root}</code>{:else}{t("settings.folderPrompt")}{/if}
              </p>
              <p class="muted">{t("settings.folderHint")}</p>
              <button onclick={chooseRoot} disabled={!idle}>
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
                  language={lang()}
                  onlanguage={changeLanguage}
                />
              </section>
            {/if}
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
            </section>
            {#if import.meta.env.DEV}
              <!-- Phase 0 dump: development only (the command also refuses in release builds). -->
              <DevTools disabled={!idle} {run} />
            {/if}
          </div>
        {/if}
      </div>

      <footer class="status">
        {#if error}
          <p class="error" role="alert">{error}</p>
          {#if appInfo}
            {@const info = appInfo}
            {@const text = error}
            <button class="link" onclick={() => openExternal(bugReportUrl(info, text))}>
              {t("support.reportShort")}
            </button>
          {/if}
          <button class="link" onclick={() => (error = null)}>{t("common.close")}</button>
        {:else if busy}
          <p class="muted" role="status">{t(busy)}</p>
        {:else if notice}
          <p class="notice" role="status">{notice}</p>
          <button class="link" onclick={() => (notice = null)}>{t("common.close")}</button>
        {/if}
      </footer>
    </main>
  </div>

  {#if planOpen}
    <StudyPlanDialog
      status={planStatus}
      feedback={planFeedback}
      busy={!idle}
      onchoose={() => importPlan(api.chooseStudyPlan)}
      ondrop={(path) => importPlan(() => api.importStudyPlan(path))}
      onpaste={(text) => importPlan(() => api.importStudyPlanText(text))}
      onclear={clearPlan}
      onclose={() => (planOpen = false)}
    />
  {/if}

  {#if cleanups.length > 0 && !cleanupLater}
    <CleanupDialog
      previews={cleanups}
      disabled={!idle}
      onconfirm={confirmCleanup}
      ondecline={declineCleanup}
      onlater={() => (cleanupLater = true)}
    />
  {/if}
{:else}
  <div class="welcome">
    <div class="welcome-box">
      <img src="/logo.svg" alt="" width="72" height="72" />
      <h1>WeBeep Sync</h1>
      <p class="muted">{t("welcome.tagline")}</p>
      {#if session}
        <Onboarding loggedIn={false} hasRoot={!!root} hasSynced={!!lastSync} />
        <button class="primary big" onclick={login} disabled={!idle}>{t("welcome.login")}</button>
      {/if}
      {#if busy}
        <p class="muted" role="status">{t(busy)}</p>
      {/if}
      {#if error}
        <p class="error" role="alert">{error}</p>
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
    padding: 12px 24px;
    background: var(--card);
    border-bottom: 1px solid var(--border);
  }

  .page-head h1 {
    flex: none;
    margin: 0;
    font-size: 1.3rem;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    margin-left: auto;
  }

  .root {
    color: var(--muted);
    font-size: 0.9em;
  }

  .banners {
    padding: 16px 24px 0;
  }

  .suggestion {
    border-color: var(--accent);
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
    padding: 20px 24px;
  }

  /* Pages with their own scrolling panes (the board). */
  .content.fill {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0;
  }

  .card-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .card-head h2 {
    margin-bottom: 8px;
  }

  .settings {
    max-width: 760px;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 28px;
    padding: 0 24px;
    background: var(--card);
    border-top: 1px solid var(--border);
    font-size: 0.9em;
  }

  .status p {
    flex: 1;
    margin: 0;
  }

  .notice {
    color: var(--accent);
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
  }

  .welcome-box {
    width: min(520px, 100%);
    text-align: center;
  }

  .welcome-box h1 {
    margin: 8px 0 4px;
  }

  .welcome-box :global(.card) {
    margin-top: 20px;
    text-align: left;
  }

  .languages {
    margin-top: 20px;
    color: var(--muted);
  }

  .languages .current {
    font-weight: 700;
    text-decoration: underline;
  }

  button.big {
    padding: 10px 28px;
    font-size: 1.05rem;
  }

  .small {
    font-size: 0.85em;
    margin-top: 24px;
  }
</style>
