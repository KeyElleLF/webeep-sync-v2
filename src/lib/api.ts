// Typed wrappers around the Tauri commands defined in src-tauri/src/commands/.
import { invoke } from "@tauri-apps/api/core";

export interface SessionInfo {
  loggedIn: boolean;
  fullname: string | null;
  moodleRelease: string | null;
}

export interface DumpReport {
  directory: string;
  files: string[];
  errors: string[];
  moodleRelease: string | null;
  referenceCourseIds: number[];
}

export type CourseState = "current" | "upcoming" | "examPending" | "passed" | "past" | "ignored";
export type ManualState = "passed" | "ignored";
export type SemesterSource = "manual" | "studyPlan" | "fileDates" | "startDate";

/** The imported study plan (only teaching codes, names and semesters are kept). */
export interface PlanStatus {
  teachings: number;
  first: number;
  second: number;
  importedAt: number;
}

export interface SemesterGuess {
  semester: "first" | "second";
  source: SemesterSource;
}

export interface CourseRow {
  id: number;
  fullname: string;
  name: string;
  code: string | null;
  teachingCode: string | null;
  /** Key of the choices that apply to the whole teaching (all its courses). */
  teachingKey: string;
  state: CourseState;
  semester: SemesterGuess | null;
  parentName: string | null;
  section: string | null;
  teachers: string[];
  academicYear: string;
  /** Course of the current academic year (the default filter). */
  current: boolean;
  selected: boolean;
  viewurl: string;
  folder: string;
}

export interface PassedSuggestion {
  courseId: number;
  name: string;
  teachingKey: string;
}

export interface CourseListing {
  courses: CourseRow[];
  suggestions: PassedSuggestion[];
}

export interface FileError {
  path: string;
  message: string;
}

export interface CleanupFile {
  path: string;
  size: number;
}

export interface CleanupPreview {
  courseId: number;
  name: string;
  files: number;
  bytes: number;
  modified: CleanupFile[];
}

export interface CleanupReport {
  trashed: number;
  freed: number;
  kept: number;
  skipped: FileError[];
}

export interface StateOutcome {
  queued: number;
  cleaned: CleanupReport[];
}

export interface CourseReport {
  downloaded: string[];
  updated: string[];
  adopted: number;
  renamed: number;
  keptLocal: string[];
  conflicts: string[];
  removedRemotely: string[];
  errors: FileError[];
}

export interface ExternalLink {
  moduleId: number;
  section: string;
  name: string;
  url: string;
}

export interface CourseSummary {
  id: number;
  name: string;
  folder: string;
  report: CourseReport;
  externalLinks: ExternalLink[];
  /** Links added since the previous sync of the course. */
  newLinks: ExternalLink[];
  error: string | null;
}

export type LinkKind = "material" | "recording" | "other";

/** An external link of a course, as saved by the syncs (name given by the teacher). */
export interface CourseLink {
  id: number;
  courseId: number;
  moduleId: number;
  name: string;
  section: string;
  url: string;
  kind: LinkKind;
  firstSeen: number;
  /** False for a link added after the first sync of its course and not opened yet. */
  seen: boolean;
}

/** Version and system, for pre-filled bug reports. */
export interface AppInfo {
  version: string;
  os: string;
  arch: string;
}

export interface SyncProgress {
  index: number;
  total: number;
  name: string;
}

export const SYNC_PROGRESS_EVENT = "sync-progress";
export const SYNC_FINISHED_EVENT = "sync-finished";
export const ANNOUNCEMENTS_EVENT = "announcements-updated";

export interface Attachment {
  filename: string;
  filesize: number;
  fileurl: string;
}

export interface Announcement {
  /** Moodle discussion id. */
  id: number;
  courseId: number;
  subject: string;
  /** Sanitized HTML (done by the backend). */
  message: string;
  author: string;
  created: number;
  modified: number;
  pinned: boolean;
  attachments: Attachment[];
  read: boolean;
}

export interface Deadline {
  id: number;
  name: string;
  activity: string;
  kind: string;
  courseId: number;
  courseName: string;
  due: number;
  overdue: boolean;
  url: string;
}

export interface Settings {
  /** Files downloaded at the same time (1–8). */
  downloadConcurrency: number;
  /** No folder for the "Materiali" section. */
  flattenMaterials: boolean;
  /** What to do with the files of a passed course. */
  cleanupPolicy: "ask" | "always" | "never";
  autosync: boolean;
  /** 30–1440. */
  autosyncMinutes: number;
  notifications: boolean;
}

/** Same: nothing changed; Moved: renamed on disk; Repointed: now uses an existing folder. */
export type FolderChange = "same" | "moved" | "repointed";

interface CommandError {
  message: string;
}

/** Converts any rejection from `invoke` into an Error with a readable message. */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    const message =
      typeof err === "object" && err !== null && "message" in err
        ? (err as CommandError).message
        : String(err);
    throw new Error(message);
  }
}

export const api = {
  authStatus: () => call<SessionInfo>("auth_status"),
  login: () => call<SessionInfo>("login"),
  logout: () => call<void>("logout"),
  /** Interface language saved in the backend, or null if never chosen (first run). */
  appInfo: () => call<AppInfo>("app_info"),
  /** The diagnostic text (personal folders, tokens and emails hidden), shown before saving. */
  diagnosticsPreview: () => call<string>("diagnostics_preview"),
  /** Saves the diagnostic file where the user chooses; null if cancelled. */
  saveDiagnostics: () => call<string | null>("save_diagnostics"),
  getLanguage: () => call<"it" | "en" | null>("get_language"),
  setLanguage: (lang: "it" | "en") => call<void>("set_language", { lang }),
  /** Development only: raw WeBeep responses, with content in Italian or English. */
  dumpPhase0: (lang: "it" | "en") => call<DumpReport>("dump_phase0", { lang }),
  listCourses: () => call<CourseListing>("list_courses"),
  setTeachingState: (teachingKey: string, teachingState: ManualState | null) =>
    call<StateOutcome>("set_teaching_state", { teachingKey, teachingState }),
  setTeachingSemester: (teachingKey: string, semester: 1 | 2 | null) =>
    call<void>("set_teaching_semester", { teachingKey, semester }),
  dismissSuggestion: (courseId: number) => call<void>("dismiss_suggestion", { courseId }),
  pendingCleanups: () => call<CleanupPreview[]>("pending_cleanups"),
  confirmCleanup: (courseId: number, includeModified: boolean) =>
    call<CleanupReport>("confirm_cleanup", { courseId, includeModified }),
  declineCleanup: (courseId: number) => call<void>("decline_cleanup", { courseId }),
  setCourseSelected: (courseId: number, selected: boolean) =>
    call<void>("set_course_selected", { courseId, selected }),
  syncRoot: () => call<string | null>("sync_root"),
  chooseSyncRoot: () => call<string | null>("choose_sync_root"),
  setCourseFolder: (courseId: number, folder: string) =>
    call<FolderChange>("set_course_folder", { courseId, folder }),
  getSettings: () => call<Settings>("get_settings"),
  lastSync: () => call<number | null>("last_sync"),
  setSettings: (settings: Settings) => call<Settings>("set_settings", { settings }),
  syncNow: () => call<CourseSummary[]>("sync_now"),
  listAnnouncements: () => call<Announcement[]>("list_announcements"),
  refreshAnnouncements: () => call<Announcement[]>("refresh_announcements"),
  markAnnouncementsRead: (id: number | null) => call<void>("mark_announcements_read", { id }),
  listDeadlines: () => call<Deadline[]>("list_deadlines"),
  mutedCourses: () => call<number[]>("muted_courses"),
  setCourseMuted: (courseId: number, muted: boolean) =>
    call<void>("set_course_muted", { courseId, muted }),
  listLinks: () => call<CourseLink[]>("list_links"),
  markLinksSeen: (id: number | null) => call<void>("mark_links_seen", { id }),
  openCourseFolder: (courseId: number) => call<void>("open_course_folder", { courseId }),
  studyPlanStatus: () => call<PlanStatus | null>("study_plan_status"),
  importStudyPlan: (path: string) => call<PlanStatus>("import_study_plan", { path }),
  chooseStudyPlan: () => call<PlanStatus | null>("choose_study_plan"),
  importStudyPlanText: (text: string) => call<PlanStatus>("import_study_plan_text", { text }),
  clearStudyPlan: () => call<void>("clear_study_plan"),};
