# Architecture

WeBeep Sync is a desktop client that keeps a local copy of the course material of
[WeBeep](https://webeep.polimi.it), the Moodle site of Politecnico di Milano. It is a
[Tauri 2](https://v2.tauri.app) app: a Rust backend does all the work (login, Moodle API,
sync, SQLite, notifications) and a small Svelte 5 + TypeScript frontend shows it.

This document is for whoever picks the project up. The full design notes, with the reasoning
behind every decision and the checks done on the real site, are in [`CLAUDE.md`](../CLAUDE.md)
(in Italian).

## Principles

- **Never lose the student's work.** A file changed locally (annotated slides) is never
  overwritten; files are deleted only when the student confirms it, only if the app downloaded
  them, and always to the Recycle Bin.
- **Pure logic, tested on real data.** Deciding *what* to download and *where* is done by pure
  functions without I/O (`sync/tree.rs`, `sync/plan.rs`, `courses/`), tested with redacted
  responses of the real site (`fixtures/`). I/O lives in separate modules.
- **One course failing never stops the others.** Errors are collected per course and shown in
  the summary; only an invalid token stops a sync.
- **Nothing leaves the PC except calls to WeBeep.** No server, no telemetry. The Moodle token
  stays in the OS credential store and is sent only to `webeep.polimi.it`.
- **No background mode.** Closing the window quits the app; periodic checks run only while it is
  open (a decision of the original author).

## Backend (`src-tauri/src`)

| Module | Role |
|---|---|
| `auth/` | Login window (SSO in an embedded webview, restricted navigation), parsing and signature check of the mobile-app token (`launch.php` → `moodlemobile://token=…`), token in the OS keyring. |
| `moodle/` | REST client for `webservice/rest/server.php` (errors come as HTTP 200 with an `exception` field), response types. |
| `courses/` | Course list: parsing of the Polimi `fullname` (code, name, teachers, parent teaching, section, academic year), course states (current, upcoming, exam pending, passed, past, ignored), semester guess, study plan import (receipt PDF, printed "Piano degli studi" page or pasted table), folder names. |
| `sync/` | Pure: `tree` (Moodle contents → files and external links), `extract` (file links in HTML), `assign` (assignment attachments), `paths` (Windows-safe names), `plan` (three-way sync), `cleanup` (what to remove for a passed course), `links` (classification of external links). I/O: `download` (atomic `.part` + rename, retries), `execute`, `run` (one sync), `folders`, `retire`, `local` (`safe_join`). |
| `notify/` | Announcements (forum "news"), deadlines (calendar), HTML sanitizing (`ammonia`), notification texts. |
| `store/` | SQLite (`rusqlite`, bundled): baseline of downloaded files, choices, settings, announcements, links, study plan. |
| `commands/` | Thin Tauri commands: validate input, call the modules above, map errors to user messages. |
| `background.rs` | Periodic checks while the window is open (announcements, deadlines, automatic sync). |

### Sync in one paragraph

For each selected course, `core_course_get_contents` (plus assignment attachments) becomes a
list of remote files with stable keys (`tree.rs`). `plan.rs` compares, for each file, the
**baseline** (what was written last time: remote date and size, local size and mtime), the
**local** file and the **remote** one: new remote file → download; only remote changed →
replace; only local changed → leave it; both changed → save the remote copy next to it as
`name (WeBeep aggiornato YYYY-MM-DD).ext`. A file already on disk with the same size and
modification time as the remote one is adopted without downloading (this is how a WeBeep Sync
v1 folder is taken over). Files removed by the teacher are reported, never deleted.

### Database

Schema changes are **append-only** migrations in `store/mod.rs` (`MIGRATIONS`, tracked with
`PRAGMA user_version`). Never edit a released migration: add a new one at the end. An app that
finds a newer schema refuses to open the database, so after adding a migration every user needs
the new version.

Data locations on Windows: database `%APPDATA%\it.webeepsync.app\webeep-sync.sqlite`, logs
`%LOCALAPPDATA%\it.webeepsync.app\logs` (7 days), token in Credential Manager (service
`it.webeepsync.app`). `tauri dev` and the installed app share them.

## Frontend (`src`)

SvelteKit in SPA mode (`adapter-static`). `routes/+page.svelte` is the app shell and holds the
state; `lib/api.ts` has typed wrappers for every Tauri command; `lib/components/` has one
component per page or dialog (courses table, links, board, deadlines, settings, study plan
dialog, cleanup dialog). The interface is always dark (palette in `lib/styles.css`). Links from
WeBeep always open in the system browser (`lib/links.ts`).

The frontend can be tried without an account: start the Vite dev server and define a fake
`window.__TAURI_INTERNALS__` whose `invoke` answers the command names of `api.ts` with sample
data (see `CLAUDE.md`, "Provare l'interfaccia senza account").

## Security notes

- The login webview gets no Tauri capability; the main window only has `core:default` and
  `opener:default`.
- The token is never logged and never put in error messages; downloads add it only to
  `/webservice/pluginfile.php/` URLs of WeBeep itself.
- HTML from WeBeep is sanitized with an allow-list before being shown.
- Study plan PDFs are parsed on a separate thread with a large stack, a timeout and panics
  caught (release builds use `panic = "unwind"` for this); at most 5 MB, no network paths.
- Release builds remap the home folder out of the binary (`scripts/build-installer.mjs`).

## Forking

The project is GPL-3.0: you can take it, change it and share your version, as long as you share
it under the same license with its source code. If you publish your own builds:

- change `identifier` in `src-tauri/tauri.conf.json` if your app should not share data with the
  original;
- generate your own update signing key and point the updater to your own releases (otherwise
  your builds will not update, or will try to update from the original project).
