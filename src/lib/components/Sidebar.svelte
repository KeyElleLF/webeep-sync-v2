<script lang="ts" module>
  export interface NavItem<Id extends string> {
    id: Id;
    label: string;
    badge?: number;
  }
</script>

<script lang="ts" generics="T extends string">
  import type { SyncProgress } from "$lib/api";

  interface Props {
    items: NavItem<T>[];
    current: T;
    fullname: string | null;
    lastSync: number | null;
    progress: SyncProgress | null;
    syncing: boolean;
    canSync: boolean;
    onopen: (id: T) => void;
    onsync: () => void;
  }

  let { items, current, fullname, lastSync, progress, syncing, canSync, onopen, onsync }: Props =
    $props();
  const DATE = new Intl.DateTimeFormat("it-IT", { dateStyle: "medium", timeStyle: "short" });
  const percent = $derived(progress ? Math.round((progress.index / progress.total) * 100) : 0);
</script>

<aside>
  <div class="brand">
    <img src="/logo.svg" alt="" width="28" height="28" />
    <span>WeBeep Sync</span>
  </div>

  <nav aria-label="Sezioni">
    {#each items as item (item.id)}
      <button
        class="nav-item"
        class:active={current === item.id}
        aria-current={current === item.id ? "page" : undefined}
        onclick={() => onopen(item.id)}
      >
        <span>{item.label}</span>
        {#if item.badge}
          <span class="badge" aria-label="{item.badge} non letti">{item.badge}</span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="sync">
    <button class="primary" disabled={!canSync} onclick={onsync}>
      {syncing ? "Sincronizzazione…" : "Sincronizza ora"}
    </button>
    {#if progress}
      <div class="bar" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
        <div style="width: {percent}%"></div>
      </div>
      <p class="small">Corso {progress.index + 1} di {progress.total}</p>
      <p class="small ellipsis" title={progress.name}>{progress.name}</p>
    {:else if lastSync}
      <p class="small">Ultimo sync: {DATE.format(new Date(lastSync * 1000))}</p>
    {:else}
      <p class="small">Nessun sync ancora</p>
    {/if}
  </div>

  <footer>
    {#if fullname}
      <p class="ellipsis" title={fullname}>{fullname}</p>
    {/if}
    <p class="small">Progetto non ufficiale, non affiliato al Politecnico di Milano.</p>
  </footer>
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    gap: 16px;
    width: var(--sidebar-width);
    box-sizing: border-box;
    padding: 16px 12px;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
    overflow-y: auto;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 6px;
    font-size: 1.1rem;
    font-weight: 700;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border: none;
    background: none;
    text-align: left;
    padding: 8px 10px;
  }

  .nav-item:hover:not(.active) {
    background: var(--hover);
  }

  .nav-item.active {
    background: var(--selected);
    color: var(--accent);
    font-weight: 600;
  }

  .sync {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .bar {
    height: 4px;
    border-radius: 2px;
    background: var(--border);
    overflow: hidden;
  }

  .bar div {
    height: 100%;
    background: var(--accent);
    transition: width 0.3s;
  }

  p {
    margin: 0;
  }

  .small {
    color: var(--muted);
    font-size: 0.85em;
  }

  footer {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 6px 0;
    border-top: 1px solid var(--border);
  }
</style>
