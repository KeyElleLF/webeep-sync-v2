<script lang="ts">
  // Settings card "Problems and reports": opens a pre-filled GitHub issue and exports the
  // diagnostic file, shown here before it is saved.
  import { api, type AppInfo } from "$lib/api";
  import { t } from "$lib/i18n/index.svelte";
  import { bugReportUrl, openExternal } from "$lib/links";

  interface Props {
    info: AppInfo | null;
    disabled: boolean;
  }

  let { info, disabled }: Props = $props();
  let preview = $state<string | null>(null);
  let message = $state<{ error: boolean; text: string } | null>(null);
  let working = $state(false);

  function errorText(err: unknown): string {
    return err instanceof Error ? err.message : String(err);
  }

  async function report() {
    if (info) await openExternal(bugReportUrl(info));
  }

  async function showPreview() {
    message = null;
    working = true;
    try {
      preview = await api.diagnosticsPreview();
    } catch (err) {
      message = { error: true, text: errorText(err) };
    } finally {
      working = false;
    }
  }

  async function save() {
    working = true;
    try {
      const path = await api.saveDiagnostics();
      if (path) {
        message = { error: false, text: t("support.saved", { path }) };
        preview = null;
      }
    } catch (err) {
      message = { error: true, text: errorText(err) };
    } finally {
      working = false;
    }
  }
</script>

<div class="set-box">
  <div class="set-row">
    <div class="set-label">
      <span>{t("support.rowTitle")}</span>
      <span class="set-desc">{t("support.intro")}</span>
    </div>
    <div class="set-ctl">
      <button disabled={disabled || !info} onclick={report}>{t("support.report")}</button>
      <button disabled={disabled || working} onclick={showPreview}>{t("support.export")}</button>
    </div>
  </div>

  {#if preview !== null}
    <div class="set-row stack">
      <p class="hint">{t("support.previewHint")}</p>
      <pre class="preview">{preview}</pre>
      <div class="buttons">
        <button class="primary" disabled={working} onclick={save}>{t("support.save")}</button>
        <button onclick={() => (preview = null)}>{t("common.close")}</button>
      </div>
    </div>
  {/if}

  {#if message}
    <div class="set-row">
      <p class:error={message.error} class:ok={!message.error} role="status">{message.text}</p>
    </div>
  {/if}
</div>

<style>
  .buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  p {
    margin: 0;
  }

  .hint {
    margin: 0;
    color: var(--muted);
    font-size: var(--fs-sm);
  }

  .preview {
    max-height: 260px;
    overflow: auto;
    margin: 0;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-sm);
    background: var(--bg);
    font-size: var(--fs-sm);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .ok {
    color: var(--ok);
  }

  .error {
    color: var(--danger);
  }
</style>
