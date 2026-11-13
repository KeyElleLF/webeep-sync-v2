// Interface language (Italian or English). Texts live in `it.ts` (source of truth) and `en.ts`,
// which must have exactly the same keys: TypeScript reports any missing or extra key.
// Content from WeBeep (course names, announcements…) is never translated: the app always asks
// WeBeep for Italian content, so folder names never change (see CLAUDE.md, "Verifiche").
import { en } from "./en";
import { it } from "./it";

export type Lang = "it" | "en";
export type MessageKey = keyof typeof it;
type Params = Record<string, string | number>;

const DICTIONARIES: Record<Lang, Record<MessageKey, string>> = { it, en };
const LOCALES: Record<Lang, string> = { it: "it-IT", en: "en-GB" };

let current = $state<Lang>("it");

export function lang(): Lang {
  return current;
}

/** Changes the language of the interface (the caller also saves it in the backend). */
export function setLang(next: Lang): void {
  current = next;
  document.documentElement.lang = next;
}

/** Italian if Windows (the webview) is in Italian, English otherwise. */
export function systemLang(): Lang {
  return navigator.language?.toLowerCase().startsWith("it") ? "it" : "en";
}

/** Locale for `Intl` formatters (dates, numbers) in the current language. */
export function locale(): string {
  return LOCALES[current];
}

/** Text of `key`, with `{name}` placeholders replaced by `params`. */
export function t(key: MessageKey, params?: Params): string {
  const text = DICTIONARIES[current][key] ?? it[key];
  if (!params) return text;
  return text.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

/** Keys with a singular and a plural form: `<base>.one` and `<base>.other`. */
type PluralBase = MessageKey extends infer K
  ? K extends `${infer Base}.one`
    ? Base
    : never
  : never;

/** Singular or plural text for `count` (`{count}` is available as a placeholder). */
export function tn(base: PluralBase, count: number, params?: Params): string {
  const key = `${base}.${count === 1 ? "one" : "other"}` as MessageKey;
  return t(key, { count, ...params });
}

/**
 * Standard names of the Polimi course template and Moodle defaults, shown translated in English.
 * Display only: folders on disk always keep the Italian names.
 */
const STANDARD_NAMES: Record<string, string> = {
  Materiali: "Materials",
  Registrazioni: "Recordings",
  Consegne: "Assignments",
  Bacheca: "Notice board",
  Annunci: "Announcements",
  "Archivio registrazioni": "Recordings archive",
  Introduzione: "General",
  Generale: "General",
  "Nuova sezione": "New section",
};

export function displayName(name: string): string {
  return current === "en" ? (STANDARD_NAMES[name] ?? name) : name;
}
