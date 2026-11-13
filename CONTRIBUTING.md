# Contributing

Thanks for helping! Bug reports, fixes and improvements are welcome. Start from
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for an overview of the code.

## Reporting a bug

Use **Settings → Report a problem** in the app: it opens a pre-filled GitHub issue. Attach the
diagnostic file the app can export (it contains logs without your token or personal paths;
check it before attaching).

## Development setup

Requirements: Rust stable (on Windows the MSVC toolchain and the Visual Studio Build Tools with
the C++ workload), Node 20 or newer, WebView2 (already on Windows 11).

```sh
npm install
npm run tauri dev          # run the app in development mode
```

Checks to run before every commit (CI runs them too):

```sh
cd src-tauri
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cd ..
npm run check
```

Build the Windows installer with `npm run build:installer` (not `npm run tauri build` alone: the
script keeps your home folder out of the binary). The installer ends up in
`src-tauri/target/release/bundle/nsis/`.

## Rules of the codebase

- **Test first** for the pure logic (`sync/`, `courses/`, `notify/`): write the failing test, then
  the code. Tests use redacted real responses in `fixtures/`.
- **Never commit personal data.** Raw responses of the site go in `fixtures/raw/` (ignored by
  git); redact them with `scripts/redact-fixtures.mjs` / `scripts/redact-phase3.mjs`, which fail
  if a name, username or email is left. No tokens, names, student numbers, grades or personal
  paths in code, tests, fixtures or commits.
- **No `unwrap()`/`expect()` in production code**; typed errors with `thiserror` and clear
  messages for the user.
- **Database migrations are append-only** (see the architecture document).
- **Destructive operations** (anything that moves or deletes the student's files) need tests and
  must keep the rules: only files downloaded by the app, never locally modified ones without
  asking, always to the Recycle Bin.
- Small functions and files, organized by domain; identifiers and comments in English, user
  interface and user documentation in Italian and English.
- Conventional commits: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`, `build:`.

## License

By contributing you agree that your contribution is licensed under the
[GPL-3.0](LICENSE), like the rest of the project.
