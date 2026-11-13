<script lang="ts">
  interface Props {
    loggedIn: boolean;
    hasRoot: boolean;
    hasSynced: boolean;
  }

  let { loggedIn, hasRoot, hasSynced }: Props = $props();

  const steps = $derived([
    { text: "Accedi con il tuo account Polimi", done: loggedIn },
    {
      text: "Scegli la cartella dove salvare i corsi (anche quella di WeBeep Sync v1)",
      done: hasRoot,
    },
    {
      text:
        "Controlla i corsi: quelli dell'anno in corso sono già selezionati, divisi per semestre. " +
        "Togli quelli che non ti interessano",
      done: hasRoot && hasSynced,
    },
    { text: "Premi «Sincronizza ora»", done: hasSynced },
  ]);
</script>

<section class="card">
  <h2>Primo avvio</h2>
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
