<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import Icon from "./Icon.svelte";

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
    {#each steps as step, i}
      <li class:done={step.done}>
        <span class="mark" aria-hidden="true">
          {#if step.done}<Icon name="check" size={12} />{:else}{i + 1}{/if}
        </span>
        <span>{step.text}{#if step.done}<span class="sr-only"> ✓</span>{/if}</span>
      </li>
    {/each}
  </ol>
</section>

<style>
  ol {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }

  .mark {
    flex: none;
    display: grid;
    place-items: center;
    width: 20px;
    height: 20px;
    margin-top: -1px;
    border: 1.5px solid var(--border-strong);
    border-radius: 50%;
    color: var(--muted);
    font-size: 0.75rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    box-sizing: border-box;
  }

  .done {
    color: var(--muted);
  }

  .done .mark {
    border-color: var(--accent-strong);
    background: var(--accent-strong);
    color: var(--on-accent);
  }

  /* The first step still to do. */
  li:not(.done):first-of-type .mark,
  .done + li:not(.done) .mark {
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
