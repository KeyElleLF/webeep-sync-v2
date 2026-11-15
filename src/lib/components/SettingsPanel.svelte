<script lang="ts">
  import type { Settings } from "$lib/api";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";

  interface Props {
    settings: Settings;
    disabled: boolean;
    onchange: (settings: Settings) => void;
  }

  let { settings, disabled, onchange }: Props = $props();
  const CONCURRENCY_OPTIONS = [1, 2, 3, 4, 5, 6, 7, 8];
  const INTERVALS: { minutes: number; label: MessageKey }[] = [
    { minutes: 30, label: "settings.interval.30" },
    { minutes: 60, label: "settings.interval.60" },
    { minutes: 120, label: "settings.interval.120" },
    { minutes: 240, label: "settings.interval.240" },
    { minutes: 480, label: "settings.interval.480" },
    { minutes: 1440, label: "settings.interval.1440" },
  ];
  const RECOMMENDED_CONCURRENCY = 4;
</script>

<h3>{t("settings.whileOpen")}</h3>
<label class="row">
  <input
    type="checkbox"
    class="switch"
    checked={settings.autosync}
    {disabled}
    onchange={(e) => onchange({ ...settings, autosync: e.currentTarget.checked })}
  />
  <span>{t("settings.autosync")}</span>
  <select
    value={settings.autosyncMinutes}
    disabled={disabled || !settings.autosync}
    aria-label={t("settings.interval")}
    onchange={(e) => onchange({ ...settings, autosyncMinutes: Number(e.currentTarget.value) })}
  >
    {#each INTERVALS as i (i.minutes)}
      <option value={i.minutes}>{t(i.label)}</option>
    {/each}
  </select>
</label>
<label class="row">
  <input
    type="checkbox"
    class="switch"
    checked={settings.notifications}
    {disabled}
    onchange={(e) => onchange({ ...settings, notifications: e.currentTarget.checked })}
  />
  <span>{t("settings.notifications")}</span>
</label>
<p class="muted">{t("settings.whileOpenHint")}</p>

<h3>{t("settings.download")}</h3>

<label class="row">
  <span>{t("settings.concurrency")}</span>
  <select
    value={settings.downloadConcurrency}
    {disabled}
    onchange={(e) => onchange({ ...settings, downloadConcurrency: Number(e.currentTarget.value) })}
  >
    {#each CONCURRENCY_OPTIONS as n}
      <option value={n}
        >{n === RECOMMENDED_CONCURRENCY ? t("settings.recommended", { value: n }) : n}</option
      >
    {/each}
  </select>
</label>
<p class="muted">{t("settings.concurrencyHint")}</p>

<label class="row">
  <input
    type="checkbox"
    class="switch"
    checked={settings.flattenMaterials}
    {disabled}
    onchange={(e) => onchange({ ...settings, flattenMaterials: e.currentTarget.checked })}
  />
  <span>{t("settings.flatten")}</span>
</label>
<p class="muted">{t("settings.flattenHint")}</p>

<h3>{t("common.passedCourses")}</h3>
<label class="row">
  <span>{t("settings.cleanupWhen")}</span>
  <select
    value={settings.cleanupPolicy}
    {disabled}
    onchange={(e) =>
      onchange({
        ...settings,
        cleanupPolicy: e.currentTarget.value as Settings["cleanupPolicy"],
      })}
  >
    <option value="ask">{t("settings.cleanup.ask")}</option>
    <option value="always">{t("settings.cleanup.always")}</option>
    <option value="never">{t("settings.cleanup.never")}</option>
  </select>
</label>
<p class="muted">{t("settings.cleanupHint")}</p>

<style>
  h3 {
    margin: 22px 0 0;
    padding-top: 16px;
    border-top: 1px solid var(--border);
    color: var(--muted);
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  /* The first section sits right under the card title. */
  h3:first-of-type {
    margin-top: 6px;
    padding-top: 0;
    border-top: none;
  }

  .row {
    display: flex;
    gap: 12px;
    align-items: center;
    margin-top: 12px;
    cursor: pointer;
  }

  .row select {
    padding-block: 5px;
  }

  .muted {
    color: var(--muted);
    margin: 5px 0 0;
    font-size: 0.86em;
    line-height: 1.5;
  }

  /* Hints line up with the text after a switch. */
  .row:has(.switch) + .muted {
    padding-left: 46px;
  }
</style>
