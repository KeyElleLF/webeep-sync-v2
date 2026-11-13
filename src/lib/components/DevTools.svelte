<script lang="ts">
  import { api, type DumpReport } from "$lib/api";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";

  interface Props {
    disabled: boolean;
    run: <T>(label: MessageKey, action: () => Promise<T>) => Promise<T | undefined>;
  }

  let { disabled, run }: Props = $props();
  let report = $state<DumpReport | null>(null);

  async function dump(lang: "it" | "en") {
    report = (await run("dev.dumping", () => api.dumpPhase0(lang))) ?? report;
  }
</script>

<details class="card">
  <summary>{t("dev.title")}</summary>
  <p class="muted">
    {t("dev.rawHint.before")}
    <code>fixtures/raw/</code>{t("dev.rawHint.after")}
  </p>
  <p class="muted">{t("dev.langHint")}</p>
  <div class="buttons">
    <button onclick={() => dump("it")} {disabled}>{t("dev.dumpIt")}</button>
    <button onclick={() => dump("en")} {disabled}>{t("dev.dumpEn")}</button>
  </div>

  {#if report}
    <div class="report">
      <p>{t("dev.folder")} <code>{report.directory}</code></p>
      <p>{t("dev.files", { count: report.files.length, files: report.files.join(", ") })}</p>
      {#if report.errors.length > 0}
        <p class="error">{t("dev.errors", { count: report.errors.length })}</p>
        <ul>
          {#each report.errors as message}
            <li>{message}</li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</details>

<style>
  .buttons {
    display: flex;
    gap: 8px;
  }

  .report {
    margin-top: 12px;
  }

  .error {
    color: var(--danger);
  }
</style>
