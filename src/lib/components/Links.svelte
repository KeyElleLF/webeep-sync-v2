<script lang="ts">
  import type { CourseLink, LinkKind } from "$lib/api";
  import { displayName, t, type MessageKey } from "$lib/i18n/index.svelte";

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

<p class="hint">{t("links.hint")}</p>

{#if courses.length === 0}
  <p class="muted">{t("links.empty")}</p>
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
        {t("links.openFolder")}
      </button>
    </header>

    {#each course.groups as group (group.kind)}
      <details open={!group.collapsed || group.items.some((l) => !l.seen)}>
        <summary>{t(group.title)} <span class="count">{group.items.length}</span></summary>
        <ul>
          {#each group.items as link (link.id)}
            <li>
              <button class="open" title={link.url} onclick={() => onopen(link)}>
                <span class="name">{displayName(link.name)}</span>
                {#if !link.seen}<span class="badge">{t("links.new")}</span>{/if}
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
    margin: 0 0 16px;
    color: var(--muted);
    font-size: 0.9em;
    max-width: 90ch;
  }

  .later {
    margin-top: 24px;
  }

  .later > summary {
    cursor: pointer;
    margin-bottom: 8px;
    color: var(--muted);
    font-weight: 600;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(380px, 100%), 1fr));
    gap: 16px;
    align-items: start;
  }

  .course {
    margin: 0;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
  }

  header h2 {
    margin: 0;
  }

  button.small {
    flex: none;
    padding: 3px 10px;
    font-size: 0.85em;
  }

  details + details {
    margin-top: 6px;
  }

  summary {
    cursor: pointer;
    padding: 4px 0;
    color: var(--muted);
    font-size: 0.9em;
    font-weight: 600;
  }

  .count {
    font-weight: 400;
  }

  ul {
    list-style: none;
    margin: 0 0 4px;
    padding: 0;
  }

  li {
    display: flex;
    flex-direction: column;
    padding: 5px 0 5px 12px;
    border-left: 2px solid var(--border);
  }

  li:hover {
    border-left-color: var(--accent);
  }

  .open {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0;
    border: none;
    background: none;
    color: var(--accent);
    text-align: left;
  }

  .open:hover .name {
    text-decoration: underline;
  }

  .name {
    font-weight: 600;
  }

  .where {
    color: var(--muted);
    font-size: 0.85em;
  }
</style>
