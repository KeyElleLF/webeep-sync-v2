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

  // While syncing, the button itself is the progress bar: square cells fill it as courses are done,
  // roughly left to right with a ragged edge, each popping in after a small random delay.
  const COLS = 28;
  const ROWS = 5;
  const CELLS = COLS * ROWS;
  const MAX_DELAY_MS = 260;
  const order = Array.from({ length: CELLS }, (_, i) => ({
    i,
    key: ((i % COLS) / COLS) * 0.6 + Math.random() * 0.4,
  })).sort((a, b) => a.key - b.key);
  const rank: number[] = [];
  order.forEach((cell, position) => (rank[cell.i] = position));
  const delays = Array.from({ length: CELLS }, () => Math.round(Math.random() * MAX_DELAY_MS));

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
  const lit = $derived(Math.round(fraction * CELLS));
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
      class="primary sync-button"
      title="{t('sync.now')} (Ctrl+R)"
      class:busy={syncing}
      disabled={!canSync || syncing}
      onclick={onsync}
    >
      {#if syncing}
        <span class="px" aria-hidden="true" style:--px-cols={COLS}>
          {#each rank as position, i}
            <i class:on={position < lit} style:--d="{delays[i]}ms"></i>
          {/each}
        </span>
      {/if}
      {#if done}
        <span class="sync-icon settled"><Icon name="check" size={16} /></span>
      {:else}
        <span class="sync-icon" class:spinning={syncing}><Icon name="sync" size={16} /></span>
      {/if}
      <span>{syncing ? t("sync.running") : t("sync.now")}</span>
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
    width: 100%;
    padding-block: 9px;
  }

  /* While syncing the button reads as working, not as switched off: a dim cobalt ground that the
     cells fill with the full colour as the sync advances. */
  .sync-button.busy {
    opacity: 1;
    border-color: color-mix(in srgb, var(--accent-strong) 60%, transparent);
    background: color-mix(in srgb, var(--accent-strong) 22%, var(--sidebar));
    color: var(--on-accent);
    box-shadow: none;
  }

  .px {
    position: absolute;
    inset: 0;
    z-index: -1;
    display: grid;
    grid-template-columns: repeat(var(--px-cols), 1fr);
    grid-auto-rows: 1fr;
    border-radius: inherit;
    overflow: hidden;
    pointer-events: none;
  }

  .px i {
    background: var(--accent-strong);
    /* Covers the hairlines between cells. */
    box-shadow: 0 0 0 0.5px var(--accent-strong);
    opacity: 0;
    transition: opacity 0s linear var(--d);
  }

  .px i.on {
    opacity: 1;
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
