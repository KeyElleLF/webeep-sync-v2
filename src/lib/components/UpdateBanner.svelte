<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import {
    availableUpdate,
    dismissUpdate,
    installUpdate,
    updateDismissed,
    updateProgress,
  } from "$lib/updates.svelte";

  let error = $state<string | null>(null);
  const update = $derived(availableUpdate());
  const status = $derived(updateProgress());

  async function install() {
    error = null;
    try {
      await installUpdate();
    } catch (err) {
      error = t("update.failed", { error: err instanceof Error ? err.message : String(err) });
    }
  }
</script>

{#if update && !updateDismissed()}
  <section class="card update" role="status">
    <p class="title">{t("update.available", { version: update.version })}</p>
    {#if update.body}
      <p class="notes">{update.body}</p>
    {/if}
    {#if status.installing}
      <p class="muted">
        {status.percent === null
          ? t("update.installing")
          : t("update.installingPercent", { percent: status.percent })}
      </p>
    {:else}
      <div class="buttons">
        <button class="primary" onclick={install}>{t("update.install")}</button>
        <button onclick={dismissUpdate}>{t("update.later")}</button>
      </div>
    {/if}
    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}
  </section>
{/if}

<style>
  .update {
    border-color: var(--accent);
  }

  .title {
    margin-top: 0;
    font-weight: 600;
  }

  .notes {
    max-height: 120px;
    overflow-y: auto;
    color: var(--muted);
    font-size: var(--fs-sm);
    white-space: pre-wrap;
  }

  .buttons {
    display: flex;
    gap: 8px;
  }

  .error {
    color: var(--danger);
  }
</style>
