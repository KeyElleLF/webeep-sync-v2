// Publishes a new version: sets it in every manifest, commits, tags and pushes. The tag starts
// the "Release" workflow on GitHub, which builds and signs the installer and publishes it; the
// installed apps then update themselves.
//
// Usage: npm run release -- 0.2.1 "What changed (shown in the update notice)"
//        npm run release -- 0.2.1 --notes-file notes.md
import { execFileSync } from "node:child_process";
import fs from "node:fs";

const SEMVER = /^\d+\.\d+\.\d+$/;

function fail(message) {
  console.error(`release: ${message}`);
  process.exit(1);
}

function git(...args) {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

function readNotes(args) {
  const fileFlag = args.indexOf("--notes-file");
  if (fileFlag !== -1) return fs.readFileSync(args[fileFlag + 1], "utf8").trim();
  return (args[0] ?? "").trim();
}

function newer(next, current) {
  const a = next.split(".").map(Number);
  const b = current.split(".").map(Number);
  for (let i = 0; i < 3; i++) if (a[i] !== b[i]) return a[i] > b[i];
  return false;
}

function updateJson(path, change) {
  const data = JSON.parse(fs.readFileSync(path, "utf8"));
  change(data);
  fs.writeFileSync(path, JSON.stringify(data, null, 2) + "\n");
}

function updateText(path, pattern, replacement) {
  const text = fs.readFileSync(path, "utf8");
  if (!pattern.test(text)) fail(`version not found in ${path}`);
  fs.writeFileSync(path, text.replace(pattern, replacement));
}

const [version, ...rest] = process.argv.slice(2);
const notes = readNotes(rest);
if (!SEMVER.test(version ?? "")) fail("give the new version, e.g. npm run release -- 0.2.1 \"notes\"");
if (!notes) fail("give the release notes (shown to users in the update notice)");
if (git("status", "--porcelain")) fail("commit or stash your changes first");
if (git("rev-parse", "--abbrev-ref", "HEAD") !== "main") fail("releases are made from main");

const current = JSON.parse(fs.readFileSync("package.json", "utf8")).version;
if (!newer(version, current)) fail(`${version} is not newer than ${current}`);

updateJson("package.json", (p) => (p.version = version));
updateJson("package-lock.json", (p) => {
  p.version = version;
  p.packages[""].version = version;
});
updateJson("src-tauri/tauri.conf.json", (c) => (c.version = version));
updateText("src-tauri/Cargo.toml", /^version = "[^"]+"/m, `version = "${version}"`);
updateText(
  "src-tauri/Cargo.lock",
  /(name = "webeep-sync"\nversion = )"[^"]+"/,
  `$1"${version}"`,
);

const files = [
  "package.json",
  "package-lock.json",
  "src-tauri/tauri.conf.json",
  "src-tauri/Cargo.toml",
  "src-tauri/Cargo.lock",
];
git("add", ...files);
git("commit", "-m", `chore: release v${version}`);
git("tag", "-a", `v${version}`, "-m", notes);
git("push", "origin", "main");
git("push", "origin", `v${version}`);
console.log(`v${version} pushed: follow the build in the "Actions" tab of the repository.`);
