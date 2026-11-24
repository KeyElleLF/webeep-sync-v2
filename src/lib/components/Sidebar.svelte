<script lang="ts" module>
  import type { IconName } from "./Icon.svelte";

  export interface NavItem<Id extends string> {
    id: Id;
    label: string;
    icon?: IconName;
    badge?: number;
    /** An error raised on that tab, shown while the user is elsewhere. */
    alert?: boolean;
  }
</script>

<script lang="ts" generics="T extends string">
  import type { SyncProgress } from "$lib/api";
  import { locale, t, tn } from "$lib/i18n/index.svelte";
  import { courseTitle } from "$lib/course-title";
  import Icon from "./Icon.svelte";

  // "Sincronizza ora" (prototype .impeccable/mockups/sync-button.html, chosen on 2026-11-24).
  // Hover: a white copy of the button is revealed by a mask whose right edge is a staircase of
  // pixel cells; only the mask position moves, in one transition. Click: the button turns deep
  // blue and fills from the bottom with cobalt water as the sync advances; full = done.
  const MASK_ROWS = 5;
  /** Cells of the staircase edge, left to right (1 = opaque); lower rows reach further right. */
  const EDGE = [0.5, 1, 0, 1, 0.25, 0, 0.5];

  /** The hover mask at the exact size of the button: opaque, then the pixel edge. */
  function pixelMask(width: number, height: number): string {
    const dpr = window.devicePixelRatio || 1;
    const cell = height / MASK_ROWS;
    const cols = Math.ceil(width / cell);
    const canvas = document.createElement("canvas");
    canvas.width = Math.ceil(width * dpr);
    canvas.height = Math.ceil(height * dpr);
    const g = canvas.getContext("2d");
    if (!g) return "none";
    g.scale(dpr, dpr);
    const start = Math.floor(((width / 1.4) * 1.04) / cell);
    for (let r = 0; r < MASK_ROWS; r++) {
      for (let k = 0; k < cols; k++) {
        const rel = k - start - r;
        const alpha = rel < 0 ? 1 : (EDGE[rel] ?? 0);
        if (alpha <= 0) continue;
        g.fillStyle = `rgba(0, 0, 0, ${alpha})`;
        g.fillRect(Math.floor(k * cell), Math.floor(r * cell), Math.ceil(cell), Math.ceil(cell));
      }
    }
    return `url(${canvas.toDataURL()})`;
  }

  interface Props {
    items: NavItem<T>[];
    current: T;
    fullname: string | null;
    lastSync: number | null;
    progress: SyncProgress | null;
    syncing: boolean;
    /** "Annulla" was pressed: the sync ends after the files in progress. */
    cancelling: boolean;
    /** A sync just ended: the button shows a tick for a moment. */
    done: boolean;
    canSync: boolean;
    onopen: (id: T) => void;
    onsync: () => void;
    oncancel: () => void;
  }

  let {
    items,
    current,
    fullname,
    lastSync,
    progress,
    syncing,
    cancelling,
    done,
    canSync,
    onopen,
    onsync,
    oncancel,
  }: Props = $props();
  const DATE = $derived(
    new Intl.DateTimeFormat(locale(), { dateStyle: "medium", timeStyle: "short" }),
  );
  // Courses done plus the share of files done in the current one, so a big course still moves.
  const fraction = $derived.by(() => {
    if (!progress || progress.total === 0) return 0;
    const inCourse = progress.filesTotal > 0 ? progress.filesDone / progress.filesTotal : 0;
    return Math.min((progress.index + inCourse) / progress.total, 1);
  });
  const percent = $derived(Math.round(fraction * 100));
  /** How full of water the button is: the progress while syncing, full once done. */
  const fill = $derived(done && !syncing ? 1 : fraction);

  let syncButton = $state<HTMLButtonElement>();
  let mask = $state("none");
  // Redrawn whenever the button changes size (the sidebar narrows under 1050 px).
  $effect(() => {
    const button = syncButton;
    if (!button) return;
    const paint = () => {
      mask = pixelMask((button.offsetWidth + 6) * 1.4, button.offsetHeight);
    };
    paint();
    const observer = new ResizeObserver(paint);
    observer.observe(button);
    return () => observer.disconnect();
  });
  const initials = $derived(
    (fullname ?? "")
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((word) => word[0]?.toUpperCase() ?? "")
      .join(""),
  );

  // The highlight slides to the current entry instead of jumping.
  const buttons: Partial<Record<string, HTMLButtonElement>> = $state({});
  const marker = $derived.by(() => {
    const active = buttons[current];
    return active ? { y: active.offsetTop, h: active.offsetHeight } : null;
  });
</script>

<aside>
  <div class="brand">
    <img src="/logo.svg" alt="" width="26" height="26" />
    <span>WeBeep Sync</span>
  </div>

  <nav aria-label={t("nav.label")}>
    {#if marker}
      <span
        class="marker"
        aria-hidden="true"
        style:transform="translateY({marker.y}px)"
        style:height="{marker.h}px"
      ></span>
    {/if}
    {#each items as item, i (item.id)}
      <button
        title="{item.label} (Ctrl+{i + 1})"
        bind:this={buttons[item.id]}
        class="nav-item"
        class:active={current === item.id}
        aria-current={current === item.id ? "page" : undefined}
        onclick={() => onopen(item.id)}
      >
        {#if item.icon}<Icon name={item.icon} size={17} />{/if}
        <span class="label">{item.label}</span>
        {#if item.alert}
          <span class="alert-dot" aria-label={t("error.generic")}></span>
        {/if}
        {#if item.badge}
          <span class="badge" aria-label={tn("nav.unread", item.badge)}>{item.badge}</span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="sync">
    <!-- Also off during an automatic sync: the backend runs one sync at a time. -->
    <button
      bind:this={syncButton}
      class="primary sync-button"
      title="{t('sync.now')} (Ctrl+R)"
      class:busy={syncing}
      class:done={done && !syncing}
      disabled={!canSync || syncing}
      onclick={onsync}
      style:--fill={fill}
      style:--px-mask={mask}
    >
      {#if syncing || done}
        <span class="water" aria-hidden="true">
          <svg viewBox="0 0 400 44" preserveAspectRatio="none">
            <path
              class="w1"
              d="M0 6 Q25 0 50 6 T100 6 T150 6 T200 6 T250 6 T300 6 T350 6 T400 6 V44 H0Z"
            />
            <path
              class="w2"
              d="M0 6 Q25 12 50 6 T100 6 T150 6 T200 6 T250 6 T300 6 T350 6 T400 6 V44 H0Z"
            />
          </svg>
        </span>
      {/if}
      <span class="label">
        {#if done}
          <span class="sync-icon settled"><Icon name="check" size={16} /></span>
        {:else}
          <span class="sync-icon" class:spinning={syncing}><Icon name="sync" size={16} /></span>
        {/if}
        <span>{syncing ? t("sync.running") : t("sync.now")}</span>
      </span>
      <span class="hover-copy" aria-hidden="true">
        <Icon name="sync" size={16} /><span>{t("sync.now")}</span>
      </span>
    </button>
    {#if progress}
      <div
        class="sr-only"
        role="progressbar"
        aria-label={t("sync.running")}
        aria-valuenow={percent}
        aria-valuemin={0}
        aria-valuemax={100}
      ></div>
      <p class="small">
        {t("sync.progress", { index: progress.index + 1, total: progress.total })}
        {#if progress.filesTotal > 0}
          · {t("sync.progressFiles", { done: progress.filesDone, total: progress.filesTotal })}
        {/if}
      </p>
      <p class="small ellipsis current" title={courseTitle(progress.name)}>{courseTitle(progress.name)}</p>
    {/if}
    {#if syncing}
      <button class="link cancel" disabled={cancelling} onclick={oncancel}>
        {cancelling ? t("sync.cancelling") : t("common.cancel")}
      </button>
    {:else if lastSync}
      <p class="small">{t("sync.last", { date: DATE.format(new Date(lastSync * 1000)) })}</p>
    {:else}
      <p class="small">{t("sync.never")}</p>
    {/if}
  </div>

  <footer>
    {#if fullname}
      <div class="user">
        <span class="avatar" aria-hidden="true">{initials}</span>
        <span class="ellipsis" title={fullname}>{fullname}</span>
      </div>
    {/if}
  </footer>
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    gap: 18px;
    width: var(--sidebar-width);
    box-sizing: border-box;
    padding: 16px 10px 14px;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
    overflow-y: auto;
    user-select: none;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 2px 8px;
    font-size: var(--fs-lg);
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  nav {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .marker {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    border-radius: var(--r-sm);
    background: var(--selected);
    transition: transform var(--t-panel) var(--ease);
    pointer-events: none;
  }

  .nav-item {
    justify-content: flex-start;
    gap: 10px;
    border: none;
    background: none;
    box-shadow: none;
    padding: 8px 10px 8px 12px;
    border-radius: var(--r-sm);
    color: var(--muted);
    font-weight: 500;
    text-align: left;
    transition:
      background-color var(--t-out) var(--ease),
      color var(--t-out) var(--ease),
      transform var(--t-press) var(--ease);
  }

  .nav-item:hover:not(.active) {
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
    transition-duration: var(--t-in);
  }

  .nav-item.active {
    color: var(--text);
  }

  .nav-item.active :global(.icon) {
    color: var(--accent);
  }

  .nav-item:active {
    transform: scale(0.985);
  }

  .label {
    flex: 1;
  }

  .sync {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 0 2px;
  }

  .sync-button {
    overflow: hidden;
    width: 100%;
    height: 36px;
    padding-block: 0;
    border: none;
  }

  .label {
    position: relative;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 8px;
    pointer-events: none;
  }

  /* Hover: the white copy, uncovered left to right by the pixel-edged mask in 0.6 s and covered
     again on the way out. Not while syncing or just after (the button is showing progress). */
  .hover-copy {
    position: absolute;
    z-index: 3;
    top: 0;
    left: -3px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: calc(100% + 6px);
    height: 100%;
    background: #ffffff;
    color: var(--accent-strong);
    pointer-events: none;
    -webkit-mask-image: var(--px-mask);
    mask-image: var(--px-mask);
    -webkit-mask-repeat: no-repeat;
    mask-repeat: no-repeat;
    -webkit-mask-size: 140% 100%;
    mask-size: 140% 100%;
    -webkit-mask-position: 350% 0;
    mask-position: 350% 0;
    transition:
      -webkit-mask-position 0.6s ease-in-out,
      mask-position 0.6s ease-in-out;
  }

  .sync-button:not(.busy, .done, :disabled):hover .hover-copy,
  .sync-button:not(.busy, .done, :disabled):focus-visible .hover-copy {
    -webkit-mask-position: 0 0;
    mask-position: 0 0;
  }

  /* While syncing the button reads as working, not as switched off: deep blue, filling up with
     cobalt water as courses and files are done. */
  .sync-button.busy {
    opacity: 1;
    background: #1b3670;
    color: var(--on-accent);
  }

  .water {
    position: absolute;
    z-index: 1;
    left: 0;
    right: 0;
    bottom: 0;
    height: calc(100% + 8px);
    transform: translateY(calc((1 - var(--fill, 0)) * 40px - 9px));
    transition: transform 0.8s cubic-bezier(0.22, 1, 0.36, 1);
    pointer-events: none;
  }

  .water svg {
    position: absolute;
    top: 0;
    left: 0;
    width: 200%;
    height: 100%;
  }

  /* Two waves drifting at different speeds and in opposite directions. */
  .w1 {
    fill: var(--accent-strong);
    animation: drift 2.6s linear infinite;
  }

  .w2 {
    fill: var(--accent-strong);
    opacity: 0.55;
    animation: drift 4.1s linear infinite reverse;
  }

  @keyframes drift {
    to {
      transform: translateX(-50%);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .w1,
    .w2 {
      animation: none;
    }
  }

  .sync-icon {
    display: inline-flex;
  }

  .sync-icon.spinning {
    animation: spin 1.1s linear infinite;
  }

  .sync-icon.settled {
    animation: fade-in var(--t-in) var(--ease);
  }

  .cancel {
    align-self: flex-start;
    white-space: normal;
    text-align: left;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .cancel:hover:not(:disabled) {
    color: var(--text);
  }

  .alert-dot {
    flex: none;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--danger);
    animation: fade-in var(--t-in) var(--ease);
  }

  p {
    margin: 0;
  }

  .small {
    color: var(--muted);
    font-size: var(--fs-sm);
    font-variant-numeric: tabular-nums;
  }

  .current {
    color: var(--text);
    opacity: 0.8;
  }

  footer {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 6px 0;
    border-top: 1px solid var(--border);
  }

  .user {
    display: flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
    font-weight: 500;
  }

  .avatar {
    flex: none;
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--selected);
    color: var(--accent);
    font-size: var(--fs-sm);
    font-weight: 600;
  }

</style>
