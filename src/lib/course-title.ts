// WeBeep sends course names in capitals ("DATA MINING"). The interface shows them in title case
// ("Data Mining"); folders on disk keep the name as WeBeep sends it. Pure, without imports, so
// `npm test` can run it directly with Node.

/** Articles, conjunctions and prepositions (English and Italian) left in lower case. */
const SMALL = new Set([
  // English
  "a", "an", "and", "as", "at", "but", "by", "for", "from", "in", "into", "nor", "of", "on",
  "or", "over", "per", "the", "to", "via", "vs", "with",
  // Italian
  "ad", "agli", "ai", "al", "all", "alla", "alle", "col", "con", "da", "dagli", "dai", "dal",
  "dall", "dalla", "dalle", "degli", "dei", "del", "dell", "della", "delle", "di", "e", "ed",
  "fra", "gli", "i", "il", "la", "le", "lo", "negli", "nei", "nel", "nell", "nella", "nelle",
  "o", "su", "sugli", "sui", "sul", "sull", "sulla", "sulle", "tra", "un", "una", "uno",
]);

/** Acronyms found in course names, in their usual form. */
const ACRONYMS = new Map(
  ["AI", "API", "BIM", "CAD", "CPU", "DSP", "FPGA", "GIS", "GPU", "HCI", "ICT", "IoT", "NLP",
    "SQL", "UI", "UX", "VLSI"].map((a) => [a.toUpperCase(), a]),
);

const ROMAN = /^(?:I{1,3}|IV|VI{0,3}|IX|X)$/;
const ORDINAL = /^(\d+)(ST|ND|RD|TH)$/;

function capitalize(word: string): string {
  return word.charAt(0).toUpperCase() + word.slice(1);
}

/** One word without surrounding punctuation, as it appears in capitals. */
function convertWord(word: string, first: boolean, last: boolean): string {
  const acronym = ACRONYMS.get(word);
  if (acronym) return acronym;
  // A lone "I" or "V" is a numeral only at the end ("Fisica I"); elsewhere "i" is an article.
  if (ROMAN.test(word) && (word.length > 1 || last)) return word;
  const ordinal = ORDINAL.exec(word);
  if (ordinal) return ordinal[1] + ordinal[2].toLowerCase();
  if (/\d/.test(word)) return word;
  // "DELL'INFORMAZIONE" → "dell'Informazione", "L'INGEGNERIA" → "L'Ingegneria".
  const apostrophe = word.indexOf("'");
  if (apostrophe > 0) {
    const head = convertWord(word.slice(0, apostrophe), first, false);
    return `${head}'${capitalize(word.slice(apostrophe + 1).toLowerCase())}`;
  }
  const lower = word.toLowerCase();
  if (!first && SMALL.has(lower)) return lower;
  return capitalize(lower);
}

/** Converts one space-separated token, keeping brackets and hyphenated parts. */
function convertToken(token: string, first: boolean, last: boolean): string {
  const match = /^([^\p{L}\p{N}']*)(.*?)([^\p{L}\p{N}']*)$/u.exec(token);
  if (!match || !match[2]) return token;
  const [, before, core, after] = match;
  const parts = core.split("-");
  const converted = parts.map((part, i) =>
    convertWord(part, first && i === 0, last && i === parts.length - 1),
  );
  return before + converted.join("-") + after;
}

/** The course name as shown in the interface. Names that already have lower-case letters
 *  (written by hand on WeBeep) are left as they are. */
export function courseTitle(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return "";
  const text = words.join(" ");
  if (text !== text.toUpperCase()) return text;
  return words.map((w, i) => convertToken(w, i === 0, i === words.length - 1)).join(" ");
}
