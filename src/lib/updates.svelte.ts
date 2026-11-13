// Automatic updates from the GitHub releases (tauri-plugin-updater). The update is signed: the
// app checks the signature with the public key in tauri.conf.json before installing it.
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";

/** While the app is open it looks again every few hours. */
export const RECHECK_MS = 6 * 60 * 60 * 1000;

let available = $state<Update | null>(null);
let dismissed = $state(false);
/** Download progress in percent while installing, `null` when unknown or idle. */
let progress = $state<number | null>(null);
let installing = $state(false);

export function availableUpdate(): Update | null {
  return available;
}

export function updateDismissed(): boolean {
  return dismissed;
}

export function updateProgress(): { installing: boolean; percent: number | null } {
  return { installing, percent: progress };
}

/** Asks GitHub for a newer version; `null` when this is the latest. */
export async function checkForUpdate(): Promise<Update | null> {
  const update = await check();
  available = update;
  dismissed = false;
  return update;
}

/** "Later": hidden until the next check. */
export function dismissUpdate(): void {
  dismissed = true;
}

/** Downloads, verifies and installs the update, then restarts the app. */
export async function installUpdate(): Promise<void> {
  if (!available || installing) return;
  installing = true;
  let total = 0;
  let received = 0;
  try {
    await available.downloadAndInstall((event) => {
      if (event.event === "Started") total = event.data.contentLength ?? 0;
      if (event.event === "Progress") {
        received += event.data.chunkLength;
        progress = total > 0 ? Math.round((received / total) * 100) : null;
      }
    });
    // On Windows the installer closes the app by itself; elsewhere restart it.
    await relaunch();
  } finally {
    installing = false;
    progress = null;
  }
}
