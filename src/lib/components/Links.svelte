<script lang="ts">
  import type { CourseLink, LinkKind } from "$lib/api";
  import { displayName, t, type MessageKey } from "$lib/i18n/index.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    /** Links of the courses being synced. */
    links: CourseLink[];
    courseNames: Map<number, string>;
    /** Courses of the next semester: shown apart, closed (their links are usually empty). */
    upcoming: Set<number>;
    disabled: boolean;
    onopen: (link: CourseLink) => void;
    onfolder: (courseId: number) => void;
  }

  let { links, courseNames, upcoming, disabled, onopen, onfolder }: Props = $props();

  const GROUPS: { kind: LinkKind; title: MessageKey; collapsed: boolean }[] = [
    { kind: "material", title: "links.group.material", collapsed: false },
    { kind: "recording", title: "links.group.recording", collapsed: false },
    { kind: "other", title: "links.group.other", collapsed: true },
  ];

  const courses = $derived(
    [...new Set(links.map((l) => l.courseId))]
      .map((id) => ({
        id,
        name: courseNames.get(id) ?? t("common.courseFallback", { id }),
        groups: GROUPS.map((g) => ({
          ...g,
          items: links.filter((l) => l.courseId === id && l.kind === g.kind),
        })).filter((g) => g.items.length > 0),
      }))
      .sort((a, b) => a.name.localeCompare(b.name)),
  );
  const now = $derived(courses.filter((c) => !upcoming.has(c.id)));
  const later = $derived(courses.filter((c) => upcoming.has(c.id)));
  const laterHasNew = $derived(
    later.some((c) => c.groups.some((g) => g.items.some((l) => !l.seen))),
  );

  function host(url: string): string {
    try {
      // The whole host, so a link named "Slides" always shows where it really goes.
      return new URL(url).hostname;
    } catch {
      return "";
    }
  }
</script>

{#if courses.length === 0}
  <p class="empty"><Icon name="links" size={18} />{t("links.empty")}</p>
{/if}

<div class="grid">
  {#each now as course (course.id)}
    {@render card(course)}
  {/each}
</div>

{#if later.length > 0}
  <details class="later" open={laterHasNew}>
    <summary>{t("links.nextSemester")} <span class="count">{later.length}</span></summary>
    <p class="hint">{t("links.nextSemesterHint")}</p>
    <div class="grid">
      {#each later as course (course.id)}
        {@render card(course)}
      {/each}
    </div>
  </details>
{/if}

{#snippet card(course: (typeof courses)[number])}
  <section class="card course">
    <header>
      <h2>{course.name}</h2>
      <button class="small" {disabled} onclick={() => onfolder(course.id)}>
        <Icon name="folder" size={14} />{t("links.openFolder")}
      </button>
    </header>

    {#each course.groups as group (group.kind)}
      <details open={!group.collapsed || group.items.some((l) => !l.seen)}>
        <summary>{t(group.title)} <span class="count">{group.items.length}</span></summary>
        <ul>
          {#each group.items as link (link.id)}
            <li>
              <button class="open" title={link.url} onclick={() => onopen(link)}>
                {#if !link.seen}
                  <span class="new-dot" aria-hidden="true"></span>
                  <span class="sr-only">{t("links.new")}</span>
                {/if}
                <span class="name">{displayName(link.name)}</span>
                <Icon name="external" size={13} class="arrow" />
              </button>
              <span class="where ellipsis" title="{displayName(link.section)} · {link.url}">
                {host(link.url)} · {displayName(link.section)}
              </span>
            </li>
          {/each}
        </ul>
      </details>
    {/each}
  </section>
{/snippet}

<style>
  .hint {
    margin: 0 0 18px;
    color: var(--muted);
    font-size: var(--fs-sm);
    max-width: 66ch;
  }

  .empty {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--muted);
  }

  .later {
    margin-top: 28px;
  }

  .later > summary {
    width: fit-content;
    margin-bottom: 10px;
    color: var(--muted);
    font-weight: 600;
    transition: color var(--t-out) var(--ease);
  }

  .later > summary:hover {
    color: var(--text);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(380px, 100%), 1fr));
    gap: 16px;
    align-items: start;
  }

  .course {
    margin: 0;
    padding: 16px 18px 12px;
    transition: border-color var(--t-out) var(--ease);
  }

  .course:hover {
    border-color: var(--border-strong);
    transition-duration: var(--t-in);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 10px;
  }

  header h2 {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
  }

  button.small {
    flex: none;
    padding: 4px 10px;
    font-size: var(--fs-sm);
  }

  details + details {
    margin-top: 6px;
  }

  summary {
    width: fit-content;
    padding: 4px 0;
    color: var(--muted);
    font-size: var(--fs-sm);
    font-weight: 600;
    transition: color var(--t-out) var(--ease);
  }

  summary:hover {
    color: var(--text);
  }

  /* How many links a group holds matters while it is closed; once open, the list says it. */
  details[open] > summary .count {
    display: none;
  }

  .count {
    display: inline-block;
    min-width: 16px;
    margin-left: 4px;
    padding: 0 5px;
    border-radius: var(--r-md);
    background: rgba(255, 255, 255, 0.07);
    font-size: var(--fs-sm);
    line-height: 16px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  ul {
    list-style: none;
    margin: 2px 0 6px;
    padding: 0;
  }

  li {
    display: flex;
    flex-direction: column;
    gap: 1px;
    margin-left: -8px;
    padding: 6px 8px;
    border-radius: var(--r-md);
    transition: background-color var(--t-out) var(--ease);
  }

  li:hover {
    background: rgba(255, 255, 255, 0.035);
    transition-duration: var(--t-in);
  }

  .open {
    justify-content: flex-start;
    gap: 6px;
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
    color: var(--text);
    text-align: left;
    white-space: normal;
  }

  /* On a page of links, colour marks the one under the pointer, not every title. */
  .open:hover:not(:disabled),
  .open:focus-visible {
    background: none;
    color: var(--accent);
  }

  .open :global(.arrow) {
    color: var(--accent);
    opacity: 0.55;
    transition:
      transform var(--t-out) var(--ease),
      opacity var(--t-out) var(--ease);
  }

  .open:hover :global(.arrow),
  .open:focus-visible :global(.arrow) {
    opacity: 1;
    transform: translate(2px, -2px);
    transition-duration: var(--t-in);
  }

  .name {
    font-weight: 600;
    background-image: linear-gradient(currentColor, currentColor);
    background-repeat: no-repeat;
    background-position: 100% 100%;
    background-size: 0% 1px;
    transition: background-size var(--t-out) var(--ease);
  }

  .open:hover .name {
    background-position: 0 100%;
    background-size: 100% 1px;
    transition-duration: var(--t-in);
  }

  .where {
    color: var(--muted);
    font-size: var(--fs-sm);
  }
</style>
