<script lang="ts" module>
  // Line icons on a 24px grid, drawn with the current text colour. Decorative: the text next to
  // them names the thing, so they are hidden from screen readers.
  const PATHS = {
    courses: "M4 5.5A1.5 1.5 0 0 1 5.5 4H18v13H5.5A1.5 1.5 0 0 0 4 18.5v-13ZM4 18.5A1.5 1.5 0 0 0 5.5 20H18v-3M8 8h6",
    links: "M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1",
    board: "M4 10v4a1 1 0 0 0 1 1h2l5 4V5L7 9H5a1 1 0 0 0-1 1ZM16 9a4 4 0 0 1 0 6M18.5 6.5a7.5 7.5 0 0 1 0 11",
    deadlines: "M5 6h14a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1ZM4 10h16M8 3v4M16 3v4M12 13v3l2 1",
    settings: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM19.4 13.5l1.4 1.1-2 3.4-1.7-.6a7 7 0 0 1-1.9 1.1L15 20.3h-4l-.3-1.8a7 7 0 0 1-1.9-1.1l-1.7.6-2-3.4 1.4-1.1a7 7 0 0 1 0-2.2L5.1 10.2l2-3.4 1.7.6a7 7 0 0 1 1.9-1.1L11 3.7h4l.3 1.8a7 7 0 0 1 1.9 1.1l1.7-.6 2 3.4-1.4 1.1a7 7 0 0 1 0 2.2Z",
    sync: "M20 12a8 8 0 0 1-14.3 4.9M4 12a8 8 0 0 1 14.3-4.9M18.5 3v4.2h-4.2M5.5 21v-4.2h4.2",
    folder: "M4 7a2 2 0 0 1 2-2h3.6l2 2H18a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7Z",
    external: "M14 5h5v5M19 5l-8 8M17 14v4a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1h4",
    pin: "M9 4h6l-1 5 3 3v2H7v-2l3-3-1-5ZM12 14v6",
    check: "M5 12.5l4.5 4.5L19 7.5",
    alert: "M12 4l9 16H3l9-16ZM12 10v4M12 17v.01",
    inbox: "M4 13l2.5-7h11l2.5 7v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-5ZM4 13h4.5l1 2h5l1-2H20",
    paperclip: "M20 11.5l-7.8 7.8a5 5 0 0 1-7.1-7.1L13 4.3a3.3 3.3 0 0 1 4.7 4.7L10 16.7a1.7 1.7 0 0 1-2.4-2.4L14.8 7",
  } as const;

  export type IconName = keyof typeof PATHS;
</script>

<script lang="ts">
  interface Props {
    name: IconName;
    size?: number;
    class?: string;
  }

  let { name, size = 16, class: className = "" }: Props = $props();
</script>

<svg
  class="icon {className}"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.7"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  <path d={PATHS[name]} />
</svg>

<style>
  .icon {
    flex: none;
    display: block;
  }
</style>
