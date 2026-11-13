<script lang="ts">
  import type { CleanupPreview } from "$lib/api";
  import { formatBytes } from "$lib/format";

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
    <h2 id="cleanup-title">Corsi superati</h2>
    {#each previews as p (p.courseId)}
      <div class="course">
        {#if p.files + p.modified.length === 0}
          <p>Hai superato <strong>{p.name}</strong>: non ci sono file scaricati da eliminare.</p>
          <div class="buttons">
            <button {disabled} onclick={() => ondecline(p)}>Ok</button>
          </div>
        {:else}
          <p>
            Hai superato <strong>{p.name}</strong>. Vuoi spostare nel Cestino i
            <strong>{p.files}</strong> file scaricati da WeBeep ({formatBytes(p.bytes)}), compresi
            quelli già presenti dalla v1 e riconosciuti come identici? Puoi recuperarli dal Cestino. I
            file che hai aggiunto tu nella cartella non vengono toccati.
          </p>
          {#if p.modified.length > 0}
            <label class="modified">
              <input type="checkbox" bind:checked={include[p.courseId]} />
              Elimina anche i {p.modified.length} file che hai modificato
            </label>
            <ul>
              {#each p.modified as file (file.path)}
                <li>{file.path}</li>
              {/each}
            </ul>
          {/if}
          <div class="buttons">
            <button class="primary" {disabled} onclick={() => onconfirm(p, include[p.courseId] ?? false)}
              >Sì, sposta nel Cestino</button
            >
            <button {disabled} onclick={() => ondecline(p)}>No, tienili</button>
          </div>
        {/if}
      </div>
    {/each}
    <button class="later" {disabled} onclick={onlater}>Chiedimelo più tardi</button>
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
