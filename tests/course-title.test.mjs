// Unit tests of the frontend's pure helpers: `npm test` (Node's own runner, no dependencies).
import { test } from "node:test";
import assert from "node:assert/strict";
import { courseTitle } from "../src/lib/course-title.ts";

test("capitals become title case, small words stay lower case", () => {
  assert.equal(courseTitle("DATA MINING"), "Data Mining");
  assert.equal(courseTitle("DATA AND INFORMATION QUALITY"), "Data and Information Quality");
  assert.equal(
    courseTitle("TECHNOLOGIES FOR INFORMATION SYSTEMS"),
    "Technologies for Information Systems",
  );
  assert.equal(courseTitle("BASI DI DATI"), "Basi di Dati");
});

test("the first word is capitalised even when it is a small word", () => {
  assert.equal(courseTitle("IN SILICO MEDICINE"), "In Silico Medicine");
  assert.equal(courseTitle("A FIRST COURSE"), "A First Course");
});

test("numbers, ordinals and roman numerals keep their form", () => {
  assert.equal(courseTitle("SOFTWARE ENGINEERING 2"), "Software Engineering 2");
  assert.equal(courseTitle("ANALISI MATEMATICA II"), "Analisi Matematica II");
  assert.equal(courseTitle("FISICA I"), "Fisica I");
  assert.equal(courseTitle("STORIA DEI I SECOLI"), "Storia dei i Secoli");
  assert.equal(
    courseTitle("MODEL IDENTIFICATION AND DATA ANALYSIS - 2ND MODULE"),
    "Model Identification and Data Analysis - 2nd Module",
  );
});

test("hyphens, apostrophes and brackets", () => {
  assert.equal(courseTitle("NON-LINEAR SYSTEMS"), "Non-Linear Systems");
  assert.equal(
    courseTitle("SISTEMI INFORMATIVI (PER IL SETTORE DELL'INFORMAZIONE)"),
    "Sistemi Informativi (per il Settore dell'Informazione)",
  );
  assert.equal(courseTitle("L'INGEGNERIA DEL SOFTWARE"), "L'Ingegneria del Software");
});

test("known acronyms stay in capitals", () => {
  assert.equal(courseTitle("AI AND ICT FOR SOCIETY"), "AI and ICT for Society");
  assert.equal(courseTitle("IOT SYSTEMS"), "IoT Systems");
});

test("names already in mixed case are left alone", () => {
  assert.equal(courseTitle("Ingegneria Informatica"), "Ingegneria Informatica");
  assert.equal(courseTitle(""), "");
  assert.equal(courseTitle("  DATA  MINING "), "Data Mining");
});
