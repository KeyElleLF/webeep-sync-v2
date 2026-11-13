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
    type SessionInfo,
    type Settings,
    type StateOutcome,
    type SyncProgress,
  } from "$lib/api";
  import { formatBytes, isActive } from "$lib/format";
  import { openExternal } from "$lib/links";
  import Board from "$lib/components/Board.svelte";
  import CleanupDialog from "$lib/components/CleanupDialog.svelte";
  import CourseList from "$lib/components/CourseList.svelte";
  import Deadlines from "$lib/components/Deadlines.svelte";
  import Links from "$lib/components/Links.svelte";
  import DevTools from "$lib/components/DevTools.svelte";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import StudyPlanDialog, { type Feedback } from "$lib/components/StudyPlanDialog.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import SyncSummary from "$lib/components/SyncSummary.svelte";

  type Tab = "courses" | "links" | "board" | "deadlines" | "settings";
  const TITLES: Record<Tab, string> = {
    courses: "Corsi",
    links: "Link",
    board: "Bacheca",
    deadlines: "Scadenze",
    settings: "Impostazioni",
  };
  const SYNC_LABEL = "Sincronizzazione in corso…";
  const FOLDER_MESSAGES = {
    same: null,
    moved: "Cartella rinominata.",
    repointed:
      "Il corso ora usa la cartella indicata: al prossimo sync i file già presenti e identici " +
      "verranno adottati senza riscaricarli.",
  } as const;

  let tab = $state<Tab>("courses");
  let session = $state<SessionInfo | null>(null);
  let busy = $state<string | null>(null);
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
      label: TITLES[id],
      badge: id === "board" ? unread : id === "links" ? newLinks : undefined,
    })),
  );

  async function run<T>(label: string, action: () => Promise<T>): Promise<T | undefined> {
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

  async function refresh() {
    session = (await run("Verifico la sessione…", api.authStatus)) ?? session;
    if (session?.loggedIn) await loadLibrary();
  }

  async function loadLibrary() {
    root = (await run("Leggo le impostazioni…", api.syncRoot)) ?? root;
    settings = (await run("Leggo le impostazioni…", api.getSettings)) ?? settings;
    lastSync = (await run("Leggo le impostazioni…", api.lastSync)) ?? lastSync;
    announcements = (await run("Leggo la bacheca…", api.listAnnouncements)) ?? announcements;
    await loadLinks();
    planStatus = (await run("Leggo le impostazioni…", api.studyPlanStatus)) ?? null;
    muted = (await run("Leggo la bacheca…", api.mutedCourses)) ?? muted;
    await loadCourses();
    cleanups = (await run("Controllo i corsi superati…", api.pendingCleanups)) ?? cleanups;
  }

  async function loadCourses() {
    const listing = await run("Carico i corsi da WeBeep…", api.listCourses);
    if (!listing) return;
    courses = listing.courses;
    suggestions = listing.suggestions;
  }

  async function login() {
    session = (await run("Completa l'accesso nella finestra di WeBeep…", api.login)) ?? session;
    if (session?.loggedIn) await loadLibrary();
  }

  async function logout() {
    await run("Esco…", api.logout);
    session = { loggedIn: false, fullname: null, moodleRelease: null };
    courses = null;
    summaries = null;
    suggestions = [];
    tab = "courses";
  }

  async function chooseRoot() {
    const chosen = await run("Scegli la cartella…", api.chooseSyncRoot);
    if (chosen) {
      root = chosen;
      await loadCourses();
    }
  }

  async function toggle(course: CourseRow, selected: boolean) {
    const update = (value: boolean) =>
      (courses = courses?.map((c) => (c.id === course.id ? { ...c, selected: value } : c)) ?? null);
    update(selected);
    await run("Salvo la selezione…", () => api.setCourseSelected(course.id, selected));
    if (error) update(!selected);
  }

  async function rename(course: CourseRow, folder: string) {
    const change = await run("Cambio la cartella…", () => api.setCourseFolder(course.id, folder));
    if (change === undefined) return;
    await loadCourses();
    notice = FOLDER_MESSAGES[change];
  }

  async function setState(teachingKey: string, state: ManualState | null) {
    const outcome = await run("Aggiorno il corso…", () => api.setTeachingState(teachingKey, state));
    if (!outcome) return;
    await loadCourses();
    cleanups = (await run("Controllo i corsi superati…", api.pendingCleanups)) ?? cleanups;
    cleanupLater = false;
    notice = stateNotice(outcome);
  }

  function stateNotice(outcome: StateOutcome): string | null {
    if (outcome.cleaned.length === 0) return null;
    const trashed = outcome.cleaned.reduce((n, r) => n + r.trashed, 0);
    const freed = outcome.cleaned.reduce((n, r) => n + r.freed, 0);
    const kept = outcome.cleaned.reduce((n, r) => n + r.kept, 0);
    const keptText = kept > 0 ? ` ${kept} file modificati da te sono rimasti al loro posto.` : "";
    return `${trashed} file spostati nel Cestino (${formatBytes(freed)}).${keptText}`;
  }

  async function setSemester(course: CourseRow, semester: 1 | 2 | null) {
    await run("Salvo il semestre…", () => api.setTeachingSemester(course.teachingKey, semester));
    await loadCourses();
  }

  async function dismiss(suggestion: PassedSuggestion) {
    await run("Salvo…", () => api.dismissSuggestion(suggestion.courseId));
    suggestions = suggestions.filter((s) => s.courseId !== suggestion.courseId);
  }

  async function confirmCleanup(preview: CleanupPreview, includeModified: boolean) {
    const report = await run("Sposto i file nel Cestino…", () =>
      api.confirmCleanup(preview.courseId, includeModified),
    );
    if (!report) return;
    cleanups = cleanups.filter((c) => c.courseId !== preview.courseId);
    const skipped = report.skipped.length
      ? ` ${report.skipped.length} file non spostati (forse aperti in un altro programma).`
      : "";
    notice = `${preview.name}: ${report.trashed} file spostati nel Cestino (${formatBytes(report.freed)}).${skipped}`;
  }

  async function declineCleanup(preview: CleanupPreview) {
    await run("Salvo…", () => api.declineCleanup(preview.courseId));
    cleanups = cleanups.filter((c) => c.courseId !== preview.courseId);
  }

  async function saveSettings(next: Settings) {
    settings = (await run("Salvo le impostazioni…", () => api.setSettings(next))) ?? settings;
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
    links = (await run("Leggo i link…", api.listLinks)) ?? links;
  }

  async function openLink(link: CourseLink) {
    await run("Apro il link nel browser…", () => openExternal(link.url));
    if (link.seen || error) return;
    await markLinksSeen(link.id);
  }

  async function markLinksSeen(id: number | null) {
    await run("Salvo…", () => api.markLinksSeen(id));
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
    const result = await run("Leggo il piano di studi…", source);
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
    planFeedback = {
      error: false,
      text:
        `Fatto: letti ${result.teachings} insegnamenti (${result.first} del 1° semestre, ` +
        `${result.second} del 2°). ${matched} ${matched === 1 ? "corso di quest'anno ora segue" : "corsi di quest'anno ora seguono"} il piano.`,
    };
  }

  async function clearPlan() {
    await run("Rimuovo il piano di studi…", api.clearStudyPlan);
    if (error) {
      planFeedback = { error: true, text: error };
      return;
    }
    planStatus = null;
    planFeedback = { error: false, text: "Piano rimosso: i semestri tornano automatici." };
    await loadCourses();
  }

  async function openCourseFolder(courseId: number) {
    await run("Apro la cartella del corso…", () => api.openCourseFolder(courseId));
  }

  async function markRead(id: number | null) {
    await run("Salvo…", () => api.markAnnouncementsRead(id));
    announcements = announcements.map((a) => (id === null || a.id === id ? { ...a, read: true } : a));
  }

  async function refreshBoard() {
    announcements = (await run("Controllo gli avvisi su WeBeep…", api.refreshAnnouncements)) ?? announcements;
  }

  async function mute(courseId: number, value: boolean) {
    await run("Salvo…", () => api.setCourseMuted(courseId, value));
    muted = value ? [...muted, courseId] : muted.filter((id) => id !== courseId);
  }

  async function loadDeadlines() {
    deadlines = (await run("Carico le scadenze…", api.listDeadlines)) ?? deadlines;
  }

  function open(next: Tab) {
    tab = next;
    if (next === "deadlines" && deadlines === null) void loadDeadlines();
  }

  onMount(() => {
    refresh();
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
    return () => stops.forEach((p) => p.then((stop) => stop()));
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
        <h1>{TITLES[tab]}</h1>
        <div class="actions">
          {#if tab === "courses"}
            <span class="root ellipsis" title={root ?? undefined}>
              {root ?? "Nessuna cartella scelta"}
            </span>
            <button onclick={chooseRoot} disabled={!idle}>
              {root ? "Cambia cartella…" : "Scegli cartella…"}
            </button>
          {:else if tab === "links"}
            <button disabled={!idle || newLinks === 0} onclick={() => markLinksSeen(null)}>
              Segna tutti come visti
            </button>
          {:else if tab === "board"}
            <button disabled={!idle} onclick={refreshBoard}>Controlla ora</button>
            <button disabled={!idle || unread === 0} onclick={() => markRead(null)}>
              Segna tutti come letti
            </button>
          {:else if tab === "deadlines"}
            <button disabled={!idle} onclick={loadDeadlines}>Aggiorna</button>
          {/if}
        </div>
      </header>

      {#if suggestions.length > 0}
        <div class="banners">
          {#each suggestions as s (s.courseId)}
            <section class="card suggestion">
              <p>
                Sembra che tu abbia superato <strong>{s.name}</strong>: il corso d'ufficio non è più
                tra le tue iscrizioni, mentre resta quello a cui ti eri iscritto tu. Segnarlo come
                superato?
              </p>
              <div class="buttons">
                <button
                  class="primary"
                  disabled={!idle}
                  onclick={() => setState(s.teachingKey, "passed")}>Sì, l'ho superato</button
                >
                <button disabled={!idle} onclick={() => dismiss(s)}>No</button>
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
                <h2>Ultima sincronizzazione</h2>
                <button class="link" onclick={() => (summaries = null)}>Chiudi</button>
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
            <p class="muted">Corsi non ancora caricati.</p>
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
              <h2>Account</h2>
              <p>Accesso effettuato come <strong>{session.fullname}</strong>.</p>
              <button onclick={logout} disabled={!idle}>Esci</button>
            </section>
            <section class="card">
              <h2>Cartella di destinazione</h2>
              <p>
                {#if root}<code>{root}</code>{:else}Scegli dove salvare il materiale dei corsi.{/if}
              </p>
              <p class="muted">
                Puoi scegliere la cartella usata da WeBeep Sync v1: i file già presenti e identici
                (stessa dimensione e data) non vengono riscaricati e i file che hai modificato non
                vengono mai sovrascritti. La cartella di ogni corso si cambia dalla scheda Corsi.
              </p>
              <button onclick={chooseRoot} disabled={!idle}>
                {root ? "Cambia cartella…" : "Scegli cartella…"}
              </button>
            </section>
            {#if settings}
              <section class="card">
                <h2>Sincronizzazione e notifiche</h2>
                <SettingsPanel {settings} disabled={!idle} onchange={saveSettings} />
              </section>
            {/if}
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
          <button class="link" onclick={() => (error = null)}>Chiudi</button>
        {:else if busy}
          <p class="muted" role="status">{busy}</p>
        {:else if notice}
          <p class="notice" role="status">{notice}</p>
          <button class="link" onclick={() => (notice = null)}>Chiudi</button>
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
      <p class="muted">Scarica e tiene aggiornato il materiale dei tuoi corsi di WeBeep.</p>
      {#if session}
        <Onboarding loggedIn={false} hasRoot={!!root} hasSynced={!!lastSync} />
        <button class="primary big" onclick={login} disabled={!idle}>Accedi a WeBeep</button>
      {/if}
      {#if busy}
        <p class="muted" role="status">{busy}</p>
      {/if}
      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}
      <p class="muted small">Progetto non ufficiale, non affiliato al Politecnico di Milano.</p>
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

  button.big {
    padding: 10px 28px;
    font-size: 1.05rem;
  }

  .small {
    font-size: 0.85em;
    margin-top: 24px;
  }
</style>
