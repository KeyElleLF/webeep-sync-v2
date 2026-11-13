// Italian labels and formatting shared by the components.
import type { CourseState, SemesterGuess } from "$lib/api";

const BYTES = new Intl.NumberFormat("it-IT", { maximumFractionDigits: 1 });
const UNITS = ["B", "KB", "MB", "GB", "TB"];

/** 1363148 → "1,3 MB" */
export function formatBytes(bytes: number): string {
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${BYTES.format(value)} ${UNITS[unit]}`;
}

const SOURCES: Record<SemesterGuess["source"], string> = {
  manual: "scelto da te",
  studyPlan: "dal piano di studi",
  fileDates: "dalle date dei file",
  startDate: "dalla data di inizio del corso",
};

/** "1°", "2°" or "?" when unknown. */
export function semesterNumber(guess: SemesterGuess | null): string {
  if (!guess) return "?";
  return guess.semester === "first" ? "1°" : "2°";
}

export function semesterLabel(guess: SemesterGuess | null): string {
  if (!guess) return "semestre non noto";
  return `${semesterNumber(guess)} semestre (${SOURCES[guess.source]})`;
}

/** Courses that are synced and shown in the board and in the links. */
export const ACTIVE_STATES: CourseState[] = ["current", "upcoming", "examPending"];

export function isActive(state: CourseState): boolean {
  return ACTIVE_STATES.includes(state);
}

export interface Section {
  title: string;
  hint: string;
  states: CourseState[];
  /** Shown only when the user asks for all courses. */
  extra: boolean;
}

export const SECTIONS: Section[] = [
  {
    title: "Questo semestre",
    hint: "Sincronizzati di default.",
    states: ["current"],
    extra: false,
  },
  {
    title: "Prossimo semestre",
    hint: "Sincronizzati di default: il materiale arriva appena viene pubblicato.",
    states: ["upcoming"],
    extra: false,
  },
  {
    title: "Esami in sospeso",
    hint: "Corsi del semestre scorso: sincronizzati finché non li segni come superati.",
    states: ["examPending"],
    extra: false,
  },
  {
    title: "Superati e ignorati",
    hint: "Non sincronizzati.",
    states: ["passed", "ignored"],
    extra: true,
  },
  {
    title: "Anni precedenti",
    hint: "Non sincronizzati.",
    states: ["past"],
    extra: true,
  },
];
