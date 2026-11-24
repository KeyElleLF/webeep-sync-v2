<script lang="ts">
  // An on/off setting, as Windows draws it: the word ("Attivato") next to the switch.
  import { t } from "$lib/i18n/index.svelte";

  interface Props {
    checked: boolean;
    disabled?: boolean;
    /** Id of the visible text that names the setting. */
    labelledby: string;
    onchange: (checked: boolean) => void;
  }

  let { checked, disabled = false, labelledby, onchange }: Props = $props();
</script>

<span class="toggle">
  <span class="word" aria-hidden="true">{checked ? t("common.on") : t("common.off")}</span>
  <input
    type="checkbox"
    class="switch"
    {checked}
    {disabled}
    aria-labelledby={labelledby}
    onchange={(e) => onchange(e.currentTarget.checked)}
  />
</span>

<style>
  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }

  .word {
    min-width: 72px;
    color: var(--muted);
    font-size: var(--fs-sm);
    text-align: right;
  }
</style>
