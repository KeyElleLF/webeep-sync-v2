<script lang="ts">
  import { trapFocus } from "$lib/focus-trap";
  import type { CleanupPreview } from "$lib/api";
  import { courseTitle } from "$lib/course-title";
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
    use:trapFocus
    aria-modal="true"
    aria-labelledby="cleanup-title"
    use:focusSafeButton
  >
    <h2 id="cleanup-title">{t("common.passedCourses")}</h2>
    {#each previews as p (p.courseId)}
      <div class="course">
        {#if p.files + p.modified.length === 0}
          <p>{t("cleanup.passed")} <strong>{courseTitle(p.name)}</strong>: {t("cleanup.nothing")}</p>
          <div class="buttons">
            <button {disabled} onclick={() => ondecline(p)}>{t("common.ok")}</button>
          </div>
        {:else}
          <p>
            {t("cleanup.passed")} <strong>{courseTitle(p.name)}</strong>. {tn("cleanup.question.before", p.files)}
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
            <button class="danger" {disabled} onclick={() => onconfirm(p, include[p.courseId] ?? false)}
              >{t("cleanup.confirm")}</button
            >
            <button {disabled} onclick={() => ondecline(p)}>{t("cleanup.keep")}</button>
          </div>
        {/if}
      </div>
    {/each}
    <div class="footer">
      <button class="later" {disabled} onclick={onlater}>{t("cleanup.later")}</button>
    </div>
  </div>
</div>

<style>
  /* Overlay, entrance and shadow: .overlay / .dialog in styles.css. */
  .dialog {
    width: min(640px, 100%);
  }

  .dialog h2 {
    font-size: var(--fs-lg);
  }

  .course + .course {
    border-top: 1px solid var(--border);
    padding-top: 10px;
  }

  .course p {
    line-height: 1.55;
  }

  .modified {
    display: flex;
    gap: 10px;
    align-items: center;
    cursor: pointer;
  }

  ul {
    max-height: 160px;
    overflow-y: auto;
    margin: 6px 0 12px;
    padding: 8px 12px 8px 28px;
    border-radius: var(--r-md);
    background: var(--bg);
    color: var(--muted);
    font-size: var(--fs-sm);
    overflow-wrap: anywhere;
  }

  .buttons {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }

  /* "Later" answers for every course at once, so it closes the dialog from its own row. */
  .footer {
    display: flex;
    justify-content: flex-end;
    margin-top: 6px;
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }
</style>
