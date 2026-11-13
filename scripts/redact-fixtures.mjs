#!/usr/bin/env node
// Builds redacted test fixtures from a raw WeBeep dump (fixtures/raw/<stamp>/).
//
//   node scripts/redact-fixtures.mjs fixtures/raw/<stamp>
//
// - Teacher names are mapped to stable placeholders ("DOCENTE A", "DOCENTE B", ...), so tests can
//   still tell different teachers apart.
// - The student's own data (name, username, ids, keys) and every e-mail address are removed.
// - The output is checked for leaks: the script fails if any collected name token survives.
import fs from "node:fs";
import path from "node:path";

const OUT_DIR = "fixtures";
const REDACTED = "REDACTED";
const REDACTED_EMAIL = "redacted@example.com";
const MIN_TOKEN_LENGTH = 4;
const EMAIL_RE = /[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi;
/** Keys whose values are dropped entirely: images and contact lists. */
const DROPPED_KEYS = new Set(["courseimage", "overviewfiles", "contacts", "userpictureurl"]);
/** Keys holding user ids (teachers or the student). */
const USER_ID_KEYS = new Set(["userid", "usermodified", "useridfrom", "useridto"]);

/** Courses kept as `get_contents` fixtures, chosen for the structure they exercise. */
const CONTENT_FIXTURES = {
  24741: "data_mining", // nested folders, ZIPs without extension, external url
  24951: "data_information_quality", // all material inside a `page`
  24477: "formal_languages_compilers", // page with ZIPs, many folders, resource
  24390: "technologies_information_systems", // single-file resources
  24561: "recommender_systems", // labels, many external urls, resources
  23497: "uncertainty_ai", // quiz, assign, folder, resource
  23056: "machine_learning", // course without material yet
};
const TIMELINES = ["inprogress", "past", "all"];

const rawDir = process.argv[2];
if (!rawDir || !fs.existsSync(rawDir)) {
  console.error("Uso: node scripts/redact-fixtures.mjs fixtures/raw/<stamp>");
  process.exit(1);
}
const readRaw = (file) => JSON.parse(fs.readFileSync(path.join(rawDir, file), "utf8"));

/** Last parenthesised group of a course fullname: the teacher list. */
function teacherGroup(fullname) {
  const groups = [...fullname.matchAll(/\(([^()]*)\)/g)].map((m) => m[1]);
  return groups.at(-1) ?? null;
}

function collectSecrets() {
  const site = readRaw("site_info.json");
  const timeline = readRaw("timeline_all.json").courses;
  const teacherGroups = [...new Set(timeline.map((c) => teacherGroup(c.fullname)).filter(Boolean))]
    // "(PER IL SETTORE DELL'INFORMAZIONE)" is a course qualifier, not a teacher list.
    .filter((g) => g === g.toUpperCase() && !g.startsWith("PER "));
  const people = [
    site.fullname, site.firstname, site.lastname,
    ...teacherGroups.flatMap((g) => g.split(",")),
    ...readRaw("courses_by_field.json").courses.flatMap((c) => (c.contacts ?? []).map((p) => p.fullname)),
    ...collectAuthors(),
  ].filter(Boolean);
  const tokens = new Set(
    people.flatMap((p) => p.split(/[\s,]+/)).filter((t) => t.length >= MIN_TOKEN_LENGTH),
  );
  const literals = [site.username, site.userprivateaccesskey, String(site.userid)].filter(Boolean);
  return { teacherGroups, tokens: [...tokens], literals };
}

function collectAuthors() {
  return fs
    .readdirSync(rawDir)
    .filter((f) => f.startsWith("contents_"))
    .flatMap((f) => readRaw(f).flatMap((s) => s.modules).flatMap((m) => m.contents ?? []))
    .map((c) => c.author);
}

const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

function buildRedactor({ teacherGroups, tokens, literals }) {
  const labels = new Map(teacherGroups.map((g, i) => [g, `DOCENTE ${String.fromCharCode(65 + (i % 26))}${i >= 26 ? Math.floor(i / 26) : ""}`]));
  const tokenRe = new RegExp(`(?<![\\p{L}\\d])(${tokens.map(escapeRe).join("|")})(?![\\p{L}\\d])`, "giu");
  const literalRe = literals.length ? new RegExp(literals.map(escapeRe).join("|"), "g") : null;
  const redactString = (s) => {
    let out = s;
    for (const [group, label] of labels) out = out.split(`(${group})`).join(`(${label})`);
    out = out.replace(EMAIL_RE, REDACTED_EMAIL);
    if (literalRe) out = out.replace(literalRe, REDACTED);
    return out.replace(tokenRe, REDACTED);
  };
  const redact = (value, key = "") => {
    if (Array.isArray(value)) return value.map((v) => redact(v));
    if (value && typeof value === "object") {
      return Object.fromEntries(
        Object.entries(value)
          .filter(([k]) => !DROPPED_KEYS.has(k))
          .map(([k, v]) => [k, redact(v, k)]),
      );
    }
    if (USER_ID_KEYS.has(key) && typeof value === "number") return 0;
    if (key === "author" && typeof value === "string") return REDACTED;
    return typeof value === "string" ? redactString(value) : value;
  };
  return { redact, teacherCount: labels.size };
}

function assertNoLeaks(text, file, { tokens, literals }) {
  const lower = text.toLowerCase();
  const wordRe = (t) => new RegExp(`(?<![\\p{L}\\d])${escapeRe(t.toLowerCase())}(?![\\p{L}\\d])`, "u");
  const leaked = [
    ...tokens.filter((t) => wordRe(t).test(lower)),
    ...literals.filter((l) => text.includes(l)),
    ...(text.match(EMAIL_RE) ?? []).filter((e) => e !== REDACTED_EMAIL),
  ];
  if (leaked.length) throw new Error(`Dati personali rimasti in ${file}: ${leaked.length} occorrenze`);
}

function writeFixture(relPath, data, secrets, redact) {
  const text = JSON.stringify(redact(data), null, 2) + "\n";
  assertNoLeaks(text, relPath, secrets);
  const target = path.join(OUT_DIR, relPath);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, text);
  console.log(`ok  ${relPath}`);
}

const secrets = collectSecrets();
const { redact, teacherCount } = buildRedactor(secrets);
for (const name of TIMELINES) {
  writeFixture(`courses/timeline_${name}.json`, readRaw(`timeline_${name}.json`), secrets, redact);
}
for (const [id, slug] of Object.entries(CONTENT_FIXTURES)) {
  writeFixture(`contents/${slug}.json`, readRaw(`contents_${id}.json`), secrets, redact);
}
writeFixture("assignments/inprogress.json", readRaw("assignments_inprogress.json"), secrets, redact);
console.log(`Docenti mappati: ${teacherCount}; token personali controllati: ${secrets.tokens.length}`);
