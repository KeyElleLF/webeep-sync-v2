// Builds the release app and the installer without the developer's home folder inside the exe.
//
// Rust embeds the source paths of the libraries (e.g. C:\Users\<name>\.cargo\registry\...) for
// panic messages, which would reveal the Windows user name of whoever builds the installer.
// --remap-path-prefix rewrites the home folder to "~". CARGO_ENCODED_RUSTFLAGS (separator 0x1f)
// is used instead of RUSTFLAGS so that a home folder with spaces still works.
// C libraries compiled by build scripts (aws-lc for HTTPS) put the same paths in __FILE__: the
// `cc` crate passes CFLAGS to the C compiler, with the MSVC or GCC/Clang equivalent option.
//
// Usage: npm run build:installer
import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";

const SEPARATOR = "\x1f";

/** C compiler option that strips the home folder from __FILE__. */
function cTrimFlag(home) {
  if (process.platform === "win32") return `/d1trimfile:${home}${path.sep}`;
  return `-ffile-prefix-map=${home}=~`;
}

function withFlag(variable, flag) {
  return [process.env[variable], flag].filter(Boolean).join(" ");
}

function encodedRustflags(home) {
  const existing = process.env.CARGO_ENCODED_RUSTFLAGS
    ? process.env.CARGO_ENCODED_RUSTFLAGS.split(SEPARATOR)
    : (process.env.RUSTFLAGS ?? "").split(/\s+/);
  return [...existing.filter(Boolean), `--remap-path-prefix=${home}=~`].join(SEPARATOR);
}

const home = os.homedir();
if (!home) {
  console.error("Cartella utente non trovata: impossibile togliere i percorsi dall'exe.");
  process.exit(1);
}

const env = { ...process.env, CARGO_ENCODED_RUSTFLAGS: encodedRustflags(home) };
delete env.RUSTFLAGS;
// CFLAGS is split on spaces, so it only works for a home folder without spaces.
if (/\s/.test(home)) {
  console.warn("La cartella utente contiene spazi: i percorsi delle librerie C restano nell'exe.");
} else {
  env.CFLAGS = withFlag("CFLAGS", cTrimFlag(home));
  env.CXXFLAGS = withFlag("CXXFLAGS", cTrimFlag(home));
}

// shell: true lets Windows find npm.cmd; the arguments are fixed, nothing comes from input.
const result = spawnSync("npm", ["run", "tauri", "build"], { stdio: "inherit", shell: true, env });
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
