<script lang="ts">
  // Settings groups of the sync: while the app is open, downloads, passed courses.
  import type { Settings } from "$lib/api";
  import { t, type MessageKey } from "$lib/i18n/index.svelte";
  import Switch from "./Switch.svelte";

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

<section class="set-group">
  <h2>{t("settings.syncAndNotifications")}</h2>
  <div class="set-box">
    <div class="set-row">
      <div class="set-label">
        <span id="set-autosync">{t("settings.autosync")}</span>
        <span class="set-desc">{t("settings.autosyncHint")}</span>
      </div>
      <div class="set-ctl">
        <select
          value={settings.autosyncMinutes}
          disabled={disabled || !settings.autosync}
          aria-label={t("settings.interval")}
          onchange={(e) =>
            onchange({ ...settings, autosyncMinutes: Number(e.currentTarget.value) })}
        >
          {#each INTERVALS as i (i.minutes)}
            <option value={i.minutes}>{t(i.label)}</option>
          {/each}
        </select>
        <Switch
          checked={settings.autosync}
          {disabled}
          labelledby="set-autosync"
          onchange={(autosync) => onchange({ ...settings, autosync })}
        />
      </div>
    </div>
    <div class="set-row">
      <div class="set-label">
        <span id="set-notifications">{t("settings.notificationsTitle")}</span>
        <span class="set-desc">{t("settings.notificationsHint")}</span>
      </div>
      <div class="set-ctl">
        <Switch
          checked={settings.notifications}
          {disabled}
          labelledby="set-notifications"
          onchange={(notifications) => onchange({ ...settings, notifications })}
        />
      </div>
    </div>
  </div>
  <p class="set-note">{t("settings.whileOpenHint")}</p>
</section>

<section class="set-group">
  <h2>{t("settings.download")}</h2>
  <div class="set-box">
    <div class="set-row">
      <div class="set-label">
        <span id="set-concurrency">{t("settings.concurrency")}</span>
        <span class="set-desc">{t("settings.concurrencyHint")}</span>
      </div>
      <div class="set-ctl">
        <select
          value={settings.downloadConcurrency}
          {disabled}
          aria-labelledby="set-concurrency"
          onchange={(e) =>
            onchange({ ...settings, downloadConcurrency: Number(e.currentTarget.value) })}
        >
          {#each CONCURRENCY_OPTIONS as n (n)}
            <option value={n}
              >{n === RECOMMENDED_CONCURRENCY ? t("settings.recommended", { value: n }) : n}</option
            >
          {/each}
        </select>
      </div>
    </div>
    <div class="set-row">
      <div class="set-label">
        <span id="set-flatten">{t("settings.flatten")}</span>
        <span class="set-desc">{t("settings.flattenHint")}</span>
      </div>
      <div class="set-ctl">
        <Switch
          checked={settings.flattenMaterials}
          {disabled}
          labelledby="set-flatten"
          onchange={(flattenMaterials) => onchange({ ...settings, flattenMaterials })}
        />
      </div>
    </div>
  </div>
</section>

<section class="set-group">
  <h2>{t("common.passedCourses")}</h2>
  <div class="set-box">
    <div class="set-row">
      <div class="set-label">
        <span id="set-cleanup">{t("settings.cleanupWhen")}</span>
        <span class="set-desc">{t("settings.cleanupHint")}</span>
      </div>
      <div class="set-ctl">
        <select
          value={settings.cleanupPolicy}
          {disabled}
          aria-labelledby="set-cleanup"
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
      </div>
    </div>
  </div>
</section>
