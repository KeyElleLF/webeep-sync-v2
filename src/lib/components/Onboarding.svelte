<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";

  interface Props {
    loggedIn: boolean;
    hasRoot: boolean;
    hasSynced: boolean;
  }

  let { loggedIn, hasRoot, hasSynced }: Props = $props();

  const steps = $derived([
    { text: t("onboarding.login"), done: loggedIn },
    { text: t("onboarding.folder"), done: hasRoot },
    { text: t("onboarding.courses"), done: hasRoot && hasSynced },
    { text: t("onboarding.sync", { button: t("sync.now") }), done: hasSynced },
  ]);
</script>

<section class="card">
  <h2>{t("onboarding.title")}</h2>
  <ol>
    {#each steps as step}
      <li class:done={step.done}>{step.text}{step.done ? " ✓" : ""}</li>
    {/each}
  </ol>
</section>

<style>
  ol {
    margin: 0;
    padding-left: 20px;
  }

  li {
    margin: 4px 0;
  }

  .done {
    color: var(--muted);
  }
</style>
