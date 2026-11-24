// Choices made on a tab that should still be there when the user comes back to it: each tab is
// rebuilt on every visit (so it opens at its top), but its filters and selection are not lost.
export const view = $state({
  /** Board: course filter ("all" or a course id), unread only, open announcement. */
  boardCourse: "all" as number | "all",
  boardUnreadOnly: false,
  boardSelected: null as number | null,
  /** Courses: also list passed, ignored and past courses; course open in the detail pane. */
  showAllCourses: false,
  courseSelected: null as number | null,
});
