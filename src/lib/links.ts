// Links in announcements and deadlines open in the system browser, never inside the app.
import { openUrl } from "@tauri-apps/plugin-opener";

const SITE = "https://webeep.polimi.it";

/** WeBeep page of an announcement (opens in the browser, where the user may be logged in). */
export function discussionUrl(discussionId: number): string {
  return `${SITE}/mod/forum/discuss.php?d=${discussionId}`;
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
