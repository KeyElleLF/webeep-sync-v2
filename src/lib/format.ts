// Labels and formatting shared by the components, in the language of the interface.
import type { CourseState, SemesterGuess } from "$lib/api";
import { locale, t, type MessageKey } from "$lib/i18n/index.svelte";

const UNITS = ["B", "KB", "MB", "GB", "TB"];

/** 1363148 → "1,3 MB" (or "1.3 MB" in English) */
export function formatBytes(bytes: number): string {
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const number = new Intl.NumberFormat(locale(), { maximumFractionDigits: 1 }).format(value);
  return `${number} ${UNITS[unit]}`;
}

const SOURCES: Record<SemesterGuess["source"], MessageKey> = {
  manual: "courses.source.manual",
  studyPlan: "courses.source.studyPlan",
  fileDates: "courses.source.fileDates",
  startDate: "courses.source.startDate",
};

/** "1°", "2°" ("1st", "2nd" in English) or "?" when unknown. */
export function semesterNumber(guess: SemesterGuess | null): string {
  if (!guess) return "?";
  return t(guess.semester === "first" ? "courses.semester1Short" : "courses.semester2Short");
}

export function semesterLabel(guess: SemesterGuess | null): string {
  if (!guess) return t("courses.semesterUnknown");
  return t("courses.semesterLabel", {
    semester: semesterNumber(guess),
    source: t(SOURCES[guess.source]),
  });
}

/** Courses that are synced and shown in the board and in the links. */
export const ACTIVE_STATES: CourseState[] = ["current", "upcoming", "examPending"];

export function isActive(state: CourseState): boolean {
  return ACTIVE_STATES.includes(state);
}

/** A group of the course list; `title` and `hint` are translated where they are shown. */
export interface Section {
  title: MessageKey;
  hint: MessageKey;
  states: CourseState[];
  /** Shown only when the user asks for all courses. */
  extra: boolean;
}

export const SECTIONS: Section[] = [
  {
    title: "courses.section.current.title",
    hint: "courses.section.current.hint",
    states: ["current"],
    extra: false,
  },
  {
    title: "courses.section.upcoming.title",
    hint: "courses.section.upcoming.hint",
    states: ["upcoming"],
    extra: false,
  },
  {
    title: "courses.section.examPending.title",
    hint: "courses.section.examPending.hint",
    states: ["examPending"],
    extra: false,
  },
  {
    title: "courses.section.retired.title",
    hint: "courses.section.notSynced",
    states: ["passed", "ignored"],
    extra: true,
  },
  {
    title: "courses.section.past.title",
    hint: "courses.section.notSynced",
    states: ["past"],
    extra: true,
  },
];
