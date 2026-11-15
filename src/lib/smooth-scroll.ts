// Smooth scrolling for the mouse wheel: each notch glides to its place instead of jumping, in
// whichever pane is under the pointer (course list, board, dialogs...). Touchpads already scroll
// smoothly and keep their native behaviour, as do Ctrl+wheel (zoom), horizontal scrolling and
// people who ask Windows for less motion. Keyboard and programmatic scrolls are smoothed by CSS
// (`scroll-behavior: smooth` in styles.css).

/** Share of the remaining distance covered every 16.7 ms: higher is snappier. */
const EASE_PER_FRAME = 0.2;
/** Below this many pixels from the target the glide stops. */
const SETTLE_PX = 0.5;
/** A wheel notch arrives as a big pixel step (or in lines/pages); smaller steps come from touchpads. */
const WHEEL_STEP_MIN = 50;
const LINE_PX = 40;
const FRAME_MS = 1000 / 60;

interface Glide {
  target: number;
  /** Exact position of the glide: the pane rounds scrollTop, which would stall the last pixels. */
  pos: number;
  /** Where the glide last put the pane: any other position means something else moved it. */
  last: number;
  frame: number;
  time: number;
}

const glides = new WeakMap<HTMLElement, Glide>();

/** A real wheel notch reports multiples of 120 in Chromium's legacy field; touchpads do not. */
const LEGACY_NOTCH = 120;

function isWheelNotch(event: WheelEvent): boolean {
  if (event.deltaMode !== WheelEvent.DOM_DELTA_PIXEL) return true;
  const legacy = (event as WheelEvent & { wheelDeltaY?: number }).wheelDeltaY;
  if (typeof legacy === "number" && legacy !== 0) return legacy % LEGACY_NOTCH === 0;
  return Math.abs(event.deltaY) >= WHEEL_STEP_MIN;
}

function pixels(event: WheelEvent, pane: HTMLElement): number {
  if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) return event.deltaY * LINE_PX;
  if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) return event.deltaY * pane.clientHeight;
  return event.deltaY;
}

/** The nearest pane under the pointer that can still scroll in that direction (like native chaining). */
function paneFor(start: EventTarget | null, down: boolean): HTMLElement | null {
  let node = start instanceof Element ? start : null;
  while (node && node !== document.documentElement) {
    if (node instanceof HTMLElement && node.scrollHeight > node.clientHeight + 1) {
      const overflow = getComputedStyle(node).overflowY;
      if (overflow === "auto" || overflow === "scroll") {
        const room = down
          ? node.scrollTop < node.scrollHeight - node.clientHeight - 1
          : node.scrollTop > 0;
        if (room) return node;
      }
    }
    node = node.parentElement;
  }
  return null;
}

function step(pane: HTMLElement, glide: Glide, now: number) {
  if (Math.abs(pane.scrollTop - glide.last) > 2) {
    // Dragged by the scrollbar or moved by the keyboard meanwhile: that wins.
    glides.delete(pane);
    return;
  }
  const elapsed = Math.min(now - glide.time, 100);
  glide.time = now;
  const ease = 1 - Math.pow(1 - EASE_PER_FRAME, elapsed / FRAME_MS);
  const remaining = glide.target - glide.pos;
  glide.pos = Math.abs(remaining) <= SETTLE_PX ? glide.target : glide.pos + remaining * ease;
  pane.scrollTo({ top: glide.pos, behavior: "instant" });
  glide.last = pane.scrollTop;
  if (glide.pos === glide.target) {
    glides.delete(pane);
    return;
  }
  glide.frame = requestAnimationFrame((t) => step(pane, glide, t));
}

/** Installs the smoothing on the whole document; returns the function that removes it. */
export function installSmoothScroll(doc: Document = document): () => void {
  const motion = window.matchMedia("(prefers-reduced-motion: reduce)");

  function wheel(event: WheelEvent) {
    if (event.defaultPrevented || event.ctrlKey || event.shiftKey || motion.matches) return;
    if (event.deltaY === 0 || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
    if (!isWheelNotch(event)) return;
    const pane = paneFor(event.target, event.deltaY > 0);
    if (!pane) return;
    event.preventDefault();
    const max = pane.scrollHeight - pane.clientHeight;
    const current = glides.get(pane);
    const from = current ? current.target : pane.scrollTop;
    const target = Math.min(Math.max(from + pixels(event, pane), 0), max);
    if (current) {
      current.target = target;
      return;
    }
    const start = pane.scrollTop;
    const glide: Glide = { target, pos: start, last: start, frame: 0, time: performance.now() };
    glides.set(pane, glide);
    glide.frame = requestAnimationFrame((t) => step(pane, glide, t));
  }

  doc.addEventListener("wheel", wheel, { passive: false });
  return () => doc.removeEventListener("wheel", wheel);
}
