<script lang="ts">
  import type { CleanupPreview } from "$lib/api";
  import { formatBytes } from "$lib/format";
  import { t, tn } from "$lib/i18n/index.svelte";

  interface Props {
    previews: CleanupPreview[];
    disabled: boolean;
    onconfirm: (preview: CleanupPreview, includeModified: boolean) => void;
    ondecline: (preview: CleanupPreview) => void;
    onlater: () => void;
  }

  let { previews, disabled, onconfirm, ondecline, onlater }: Props = $props();
  let include = $state<Record<number, boolean>>({});

  function focusSafeButton(node: HTMLElement) {
    // The safe choice, never the button that moves files to the Trash.
    node.querySelector<HTMLButtonElement>("button.later")?.focus();
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !disabled && onlater()} />

<div class="overlay">
  <div
    class="card dialog"
    role="dialog"
    aria-modal="true"
    aria-labelledby="cleanup-title"
    use:focusSafeButton
  >
    <h2 id="cleanup-title">{t("common.passedCourses")}</h2>
    {#each previews as p (p.courseId)}
      <div class="course">
        {#if p.files + p.modified.length === 0}
          <p>{t("cleanup.passed")} <strong>{p.name}</strong>: {t("cleanup.nothing")}</p>
          <div class="buttons">
            <button {disabled} onclick={() => ondecline(p)}>{t("common.ok")}</button>
          </div>
        {:else}
          <p>
            {t("cleanup.passed")} <strong>{p.name}</strong>. {tn("cleanup.question.before", p.files)}
            <strong>{p.files}</strong>
            {tn("cleanup.question.after", p.files, { size: formatBytes(p.bytes) })}
          </p>
          {#if p.modified.length > 0}
            <label class="modified">
              <input type="checkbox" bind:checked={include[p.courseId]} />
              {tn("cleanup.includeModified", p.modified.length)}
            </label>
            <ul>
              {#each p.modified as file (file.path)}
                <li>{file.path}</li>
              {/each}
            </ul>
          {/if}
          <div class="buttons">
            <button class="primary" {disabled} onclick={() => onconfirm(p, include[p.courseId] ?? false)}
              >{t("cleanup.confirm")}</button
            >
            <button {disabled} onclick={() => ondecline(p)}>{t("cleanup.keep")}</button>
          </div>
        {/if}
      </div>
    {/each}
    <button class="later" {disabled} onclick={onlater}>{t("cleanup.later")}</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(0, 0, 0, 0.4);
  }

  .dialog {
    width: min(640px, 100%);
    max-height: calc(100vh - 48px);
    overflow-y: auto;
    box-sizing: border-box;
    margin: 0;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3);
  }

  .course + .course {
    border-top: 1px solid var(--border);
    padding-top: 8px;
  }

  .modified {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  ul {
    margin: 4px 0 8px;
    color: var(--muted);
    overflow-wrap: anywhere;
  }

  .buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }

  .later {
    margin-top: 4px;
  }
</style>
