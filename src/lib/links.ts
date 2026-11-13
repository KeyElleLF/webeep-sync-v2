// Links in announcements and deadlines open in the system browser, never inside the app.
import { openUrl } from "@tauri-apps/plugin-opener";

const SITE = "https://webeep.polimi.it";

/** WeBeep page of an announcement (opens in the browser, where the user may be logged in). */
export function discussionUrl(discussionId: number): string {
  return `${SITE}/mod/forum/discuss.php?d=${discussionId}`;
}

/** The project on GitHub (forks: change it to your own repository). */
export const REPO_URL = "https://github.com/KeyElleLF/webeep-sync-v2";
/** GitHub caps URLs: the error text in a pre-filled issue is cut to this length. */
const MAX_ERROR_CHARS = 1500;

/** A new GitHub issue from the bug form, with version, system and an error filled in. */
export function bugReportUrl(
  info: { version: string; os: string; arch: string },
  error?: string,
): string {
  const params = new URLSearchParams({
    template: "bug_report.yml",
    title: error ? `[Bug] ${error.slice(0, 80)}` : "[Bug] ",
    version: info.version,
    os: `${info.os} ${info.arch}`,
  });
  if (error) params.set("error", error.slice(0, MAX_ERROR_CHARS));
  return `${REPO_URL}/issues/new?${params}`;
}

export async function openExternal(url: string): Promise<void> {
  if (/^(https?:|mailto:)/i.test(url)) await openUrl(url);
}

/** Click handler for sanitized HTML: anchors go to the browser. */
export function interceptLinks(event: MouseEvent): void {
  const anchor = (event.target as HTMLElement | null)?.closest("a");
  if (!anchor) return;
  event.preventDefault();
  const href = anchor.getAttribute("href");
  if (href) void openExternal(href);
}
