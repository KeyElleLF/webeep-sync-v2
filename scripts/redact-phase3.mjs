#!/usr/bin/env node
// Builds redacted Phase 3 fixtures (announcements and deadlines) from a raw WeBeep dump.
//
//   node scripts/redact-phase3.mjs fixtures/raw/<stamp>
//
// Announcements keep their HTML *structure* (tags, link count, dates, flags) but every text,
// link target, author and id of a person is replaced. Deadlines keep type, dates and course
// code/name; the teacher in the course name is replaced. The output is checked for leaks.
import fs from "node:fs";
import path from "node:path";

const OUT_DIR = path.join("fixtures", "notify");
const TEXT = "Testo dell'avviso";
const LINK = "https://example.com/link";
const MAX_DISCUSSIONS = 3;

const rawDir = process.argv[2];
if (!rawDir || !fs.existsSync(rawDir)) {
  console.error("Uso: node scripts/redact-phase3.mjs fixtures/raw/<stamp>");
  process.exit(1);
}
const readRaw = (file) => JSON.parse(fs.readFileSync(path.join(rawDir, file), "utf8"));

/** Replaces text between tags and every href, keeping the tags. */
function redactHtml(html) {
  return html
    .replace(/href\s*=\s*"[^"]*"/gi, `href="${LINK}"`)
    .replace(/href\s*=\s*'[^']*'/gi, `href="${LINK}"`)
    .replace(/\s(style|class|id|title|target|rel)\s*=\s*"[^"]*"/gi, "")
    .replace(/>([^<]+)</g, (_, text) => (text.trim() ? `>${TEXT}<` : `>${text}<`));
}

function redactDiscussion(d, index) {
  return {
    ...d,
    name: `Avviso ${index + 1}`,
    subject: `Avviso ${index + 1}`,
    message: redactHtml(d.message ?? ""),
    userfullname: "DOCENTE A",
    usermodifiedfullname: "DOCENTE A",
    userid: 0,
    usermodified: 0,
    userpictureurl: "",
    usermodifiedpictureurl: "",
  };
}

/** `056892 - DATA MINING (TEACHER) [2026-27]` → `… (DOCENTE) [2026-27]` */
function redactCourseName(fullname) {
  return fullname.replace(/\(([^()]*)\)(\s*\[[^\]]*\])?\s*$/, (_, _t, year) => `(DOCENTE)${year ?? ""}`);
}

function redactEvent(e) {
  const course = {
    id: e.course.id,
    fullname: redactCourseName(e.course.fullname),
    coursecategory: e.course.coursecategory,
    viewurl: e.course.viewurl,
  };
  return {
    id: e.id,
    name: e.name,
    activityname: e.activityname,
    activitystr: e.activitystr,
    modulename: e.modulename,
    eventtype: e.eventtype,
    component: e.component,
    instance: e.instance,
    timestart: e.timestart,
    timesort: e.timesort,
    timeduration: e.timeduration,
    overdue: e.overdue,
    url: e.url,
    course,
    action: { name: e.action?.name, actionable: e.action?.actionable, url: e.action?.url },
  };
}

function collectPeople() {
  const people = new Set();
  for (const file of fs.readdirSync(rawDir).filter((f) => f.startsWith("discussions_"))) {
    for (const d of readRaw(file).discussions) {
      people.add(d.userfullname);
      people.add(d.usermodifiedfullname);
    }
  }
  const site = readRaw("site_info.json");
  [site.fullname, site.firstname, site.lastname].forEach((p) => people.add(p));
  for (const e of readRaw("action_events.json").events) {
    const teacher = e.course.fullname.match(/\(([^()]*)\)(\s*\[[^\]]*\])?\s*$/)?.[1];
    teacher?.split(",").forEach((t) => people.add(t));
  }
  return [...people]
    .filter(Boolean)
    .flatMap((p) => p.split(/[\s,]+/))
    .filter((t) => t.length >= 4 && t !== "DOCENTE");
}

function writeOut(name, value) {
  const text = JSON.stringify(value, null, 2) + "\n";
  fs.mkdirSync(OUT_DIR, { recursive: true });
  fs.writeFileSync(path.join(OUT_DIR, name), text);
  return text;
}

const discussionFile = fs
  .readdirSync(rawDir)
  .filter((f) => f.startsWith("discussions_"))
  .map((f) => ({ f, n: readRaw(f).discussions.length }))
  .sort((a, b) => b.n - a.n)[0].f;
const discussions = readRaw(discussionFile).discussions.slice(0, MAX_DISCUSSIONS).map(redactDiscussion);
const events = readRaw("action_events.json").events.map(redactEvent);
const outputs = [
  writeOut("discussions.json", { discussions, warnings: [] }),
  writeOut("action_events.json", { events, firstid: 0, lastid: 0 }),
];

const leaks = collectPeople().filter((token) =>
  outputs.some((text) => text.toUpperCase().includes(token.toUpperCase())),
);
if (leaks.length > 0) {
  console.error(`Dati personali ancora presenti (${leaks.length}): controlla lo script.`);
  process.exit(1);
}
console.log(`Scritti ${OUT_DIR}/discussions.json (${discussions.length}) e action_events.json (${events.length}).`);
