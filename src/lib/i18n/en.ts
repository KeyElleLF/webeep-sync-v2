// English texts of the interface. Same keys as `it.ts`: TypeScript reports missing or extra ones.
// Names of Polimi pages ("Servizi Online", "Carriera didattica", "Piano degli studi") and of
// folders on disk ("Materiali") stay in Italian: that is what the user sees there.
import type { it } from "./it";

export const en: Record<keyof typeof it, string> = {
  "language.label": "Language",
  "language.it": "Italiano",
  "language.en": "English",

  "common.close": "Close",
  "common.retry": "Retry",
  "common.save": "Save",
  "common.on": "On",
  "common.off": "Off",
  "common.cancel": "Cancel",
  "common.change": "Change",
  "common.yes": "Yes",
  "common.no": "No",
  "common.ok": "OK",
  "common.info": "More information",
  "common.course": "Course",
  "common.courseFallback": "Course {id}",
  "common.notSynced": "not synced",
  "common.openOnWebeep": "Open on WeBeep",
  "common.chooseFolder": "Choose folder…",
  "common.changeFolder": "Change folder…",
  "common.passedCourses": "Passed courses",
  "common.unofficial": "Unofficial project, not affiliated with Politecnico di Milano.",

  "nav.label": "Sections",
  "nav.today": "Today",
  "nav.courses": "Courses",
  "nav.links": "Links",
  "nav.board": "Announcements",
  "nav.deadlines": "Deadlines",
  "nav.settings": "Settings",
  "nav.unread.one": "{count} unread",
  "nav.unread.other": "{count} unread",

  "status.checkingSession": "Checking the session…",
  "status.readingSettings": "Loading settings…",
  "status.readingBoard": "Loading announcements…",
  "status.checkingPassed": "Checking passed courses…",
  "status.loadingCourses": "Loading courses from WeBeep…",
  "status.loggingIn": "Complete the login in the WeBeep window…",
  "status.loggingOut": "Logging out…",
  "status.choosingFolder": "Choose the folder…",
  "status.savingSelection": "Saving the selection…",
  "status.changingFolder": "Changing the folder…",
  "status.updatingCourse": "Updating the course…",
  "status.savingSemester": "Saving the semester…",
  "status.saving": "Saving…",
  "status.trashing": "Moving the files to the Recycle Bin…",
  "status.savingSettings": "Saving settings…",
  "status.syncing": "Syncing…",
  "status.readingLinks": "Loading links…",
  "status.readingHistory": "Loading what is new…",
  "status.openingLink": "Opening the link in the browser…",
  "status.readingPlan": "Reading the study plan…",
  "status.removingPlan": "Removing the study plan…",
  "status.openingCourseFolder": "Opening the course folder…",
  "status.checkingBoard": "Checking announcements on WeBeep…",
  "status.loadingDeadlines": "Loading deadlines…",

  "welcome.tagline": "Downloads the material of your WeBeep courses and keeps it up to date.",
  "welcome.login": "Log in to WeBeep",

  "onboarding.title": "Getting started",
  "onboarding.login": "Log in with your Polimi account",
  "onboarding.folder":
    "Choose the folder where courses are saved (the WeBeep Sync v1 folder works too)",
  "onboarding.courses":
    "Check the courses: this year's are already selected, split by semester. " +
    "Untick the ones you don't need",
  "onboarding.sync": "Press “{button}”",

  "courses.noRoot": "No folder chosen",
  "courses.notLoaded": "Courses not loaded yet.",
  "courses.folderMoved": "Folder renamed.",
  "courses.folderRepointed":
    "The course now uses the chosen folder: at the next sync, files already there and " +
    "identical will be adopted without downloading them again.",
  "courses.suggestion.before": "It looks like you passed",
  "courses.suggestion.after":
    ": the course you were assigned to is no longer among your enrolments, while the one you " +
    "enrolled in yourself is still there. Mark it as passed?",
  "courses.suggestion.yes": "Yes, I passed it",
  "courses.selected.one": "course to sync",
  "courses.selected.other": "courses to sync",
  "courses.showAll": "Also show passed, ignored and previous years",
  "courses.selectionHint":
    "The selection follows the state of the course; your manual choices always take " +
    "precedence. “Passed” and “Ignore” apply to all courses of the same subject.",
  "courses.selectionInfoLabel": "How the course selection works",
  "courses.col.semester": "Semester",
  "courses.col.folder": "Folder",
  "courses.section.current.title": "This semester",
  "courses.section.current.hint": "Synced by default.",
  "courses.section.upcoming.title": "Next semester",
  "courses.section.upcoming.hint":
    "Synced by default: the material arrives as soon as it is published.",
  "courses.section.examPending.title": "Exams pending",
  "courses.section.examPending.hint":
    "Courses of last semester: synced until you mark them as passed.",
  "courses.section.retired.title": "Passed and ignored",
  "courses.section.past.title": "Previous years",
  "courses.section.notSynced": "Not synced.",
  "courses.semester1": "1st semester",
  "courses.semester2": "2nd semester",
  "courses.semester1Short": "1st",
  "courses.semester2Short": "2nd",
  "courses.semesterLabel": "{semester} semester ({source})",
  "courses.semesterUnknown": "semester unknown",
  "courses.source.manual": "chosen by you",
  "courses.source.studyPlan": "from the study plan",
  "courses.source.fileDates": "from the file dates",
  "courses.source.startDate": "from the start date of the course",
  "courses.semesterOf": "Semester of {name}",
  "courses.auto": "Automatic",
  "courses.autoWith": "Auto: {semester}",
  "courses.semesterPick": "Unknown: choose",
  "courses.semesterShort": "{semester} · {source}",
  "courses.sourceShort.manual": "chosen by you",
  "courses.sourceShort.studyPlan": "from the plan",
  "courses.sourceShort.fileDates": "from files",
  "courses.sourceShort.startDate": "from start date",
  "courses.folderName": "Folder name",
  "courses.folderSame": "same as the course",
  "courses.moreFiles": "and {count} more",
  "courses.rowSyncing": "Syncing now",
  "courses.rowAdded": "{count} new or updated files",
  "courses.restore": "Restore",
  "courses.passed": "Passed",
  "courses.ignore": "Ignore",
  "courses.confirmPassed.before": "Mark",
  "courses.confirmPassed.after": "as passed",
  "courses.confirmSiblings.one": "(also the other course of the same subject)",
  "courses.confirmSiblings.other": "(also the other {count} courses of the same subject)",
  "courses.syncOf": "Sync {name}",
  "courses.semesterUnknownShort": "Unknown",
  "courses.listLabel": "Course list",
  "courses.section.others": "Passed, ignored and past years",
  "courses.noneSelected": "Choose a course from the list.",
  "courses.syncThis": "Sync",
  "courses.syncThisHint": "Downloads the course material at every sync",
  "courses.code": "code {code}",
  "courses.rename": "Rename…",
  "courses.renameOf": "Rename the folder of {name}",
  "courses.open": "Open",
  "courses.openOf": "Open the folder of {name}",
  "courses.lastSync": "Last sync",
  "courses.lastSyncNothing": "Nothing new",
  "courses.links": "Course links",
  "courses.linksHint": "They open in the browser",
  "courses.markPassed": "Mark as passed",
  "courses.stateNote":
    "“Passed” and “Ignore” apply to all courses of the same subject, in other academic " +
    "years too.",
  "courses.statePassed": "Marked as passed: it is no longer synced.",
  "courses.stateIgnored": "Ignored: it is not synced.",
  "courses.statePast": "Course of a past academic year.",

  "plan.title": "Study plan",
  "plan.importButton": "Import study plan",
  "plan.loadedButton": "Study plan ✓",
  "plan.infoLabel": "What is the study plan",
  "plan.info":
    "From your study plan the app reads whether each subject is in the 1st or 2nd semester and " +
    "sorts the courses automatically. You can use the PDF receipt, the “Piano degli studi” page " +
    "of the Servizi Online saved as PDF (Ctrl+P), or its table copied and pasted. Only the " +
    "codes, names and semesters of the subjects are kept.",
  "plan.intro":
    "From your study plan the app reads whether each subject is in the 1st or 2nd semester and " +
    "sorts the courses accordingly.",
  "plan.loaded.one":
    "Plan loaded on {date}: {count} subject, {first} in the 1st semester and {second} in the 2nd.",
  "plan.loaded.other":
    "Plan loaded on {date}: {count} subjects, {first} in the 1st semester and {second} in the 2nd.",
  "plan.remove": "Remove",
  "plan.reading": "Reading the plan…",
  "plan.dropHere": "Drag the PDF here",
  "plan.dropHint": "the study plan receipt, or the “Piano degli studi” page saved as PDF",
  "plan.dropPdf": "Drag a PDF file.",
  "plan.chooseFile": "Choose file…",
  "plan.pasteLabel": "Or paste the table of subjects",
  "plan.pastePlaceholder":
    "Select the table on the “Piano degli studi” page, copy it (Ctrl+C) and paste it here " +
    "(Ctrl+V)",
  "plan.readTable": "Read the table",
  "plan.noReceipt": "Don't have the receipt?",
  "plan.how.before": "It can no longer be downloaded from the Polimi Servizi Online. Instead, open",
  "plan.how.page": "Carriera didattica → Piano degli studi",
  "plan.how.after": ", then:",
  "plan.how.paste.label": "to paste:",
  "plan.how.paste.text":
    "select the table of subjects with the mouse (from the header to the last row), copy it " +
    "and paste it above: this way the app sees only the table;",
  "plan.how.pdf.label": "for the PDF:",
  "plan.how.pdf.text":
    "Ctrl+P → “Save as PDF”. To leave out your personal data, select the table first and in " +
    "the print window choose “Selection only” (under “More settings”).",
  "plan.note":
    "Only the codes, names and semesters of the subjects are kept: personal data, teachers and " +
    "grades are not saved, and the file never leaves your PC. Semester choices made by hand are " +
    "replaced by the plan; you can always change them again from the course list.",
  "plan.imported.one":
    "Done: read {count} subject ({first} in the 1st semester, {second} in the 2nd).",
  "plan.imported.other":
    "Done: read {count} subjects ({first} in the 1st semester, {second} in the 2nd).",
  "plan.matched.one": "{count} course of this year now follows the plan.",
  "plan.matched.other": "{count} courses of this year now follow the plan.",
  "plan.removed": "Plan removed: semesters are automatic again.",

  "links.markAllSeen": "Mark all as seen",
  "links.hint":
    "Links that teachers put on WeBeep instead of files (OneDrive folders, recordings, " +
    "repositories). They open in the browser: put the files you download from there in the " +
    "course folder, in a subfolder of your own. The app never touches them.",
  "links.infoLabel": "What these links are",
  "links.empty":
    "No links in the synced courses. Links appear after a sync: if a course has only links " +
    "and no files, make sure it is selected in the Courses tab.",
  "links.nextSemester": "Next semester's courses",
  "links.nextSemesterHint":
    "Lectures have not started yet: links (recordings included) are usually still empty.",
  "links.openFolder": "Open course folder",
  "links.group.material": "Material",
  "links.group.recording": "Recordings and online lectures",
  "links.group.other": "Other links",
  "links.new": "new",

  "board.checkNow": "Refresh",
  "board.markAllRead": "Mark all as read",
  "board.listLabel": "List of announcements",
  "board.allCourses": "All courses",
  "board.unreadOnly": "Unread only ({count})",
  "board.mute": "No notifications for this course",
  "board.empty":
    "No announcements saved. The app checks the announcements of the synced courses every 15 " +
    "minutes; you can also press “{button}”.",
  "board.noMatch": "No announcements match these filters.",
  "board.attachments": "Attachments (download them from the announcement page on WeBeep):",
  "board.selectHint": "Select an announcement from the list.",

  "deadlines.refresh": "Refresh",
  "deadlines.hint":
    "Assignments and quizzes for the next 14 days. You get a reminder 24 hours before, if the app is open.",
  "deadlines.infoLabel": "Which deadlines appear",
  "deadlines.notLoaded": "Deadlines not loaded yet.",
  "deadlines.empty": "No deadlines in the next 14 days.",
  "deadlines.col.due": "Due",
  "deadlines.col.activity": "Activity",
  "deadlines.col.kind": "Type",
  "deadlines.overdue": "Overdue",
  "deadlines.group.today": "Today",
  "deadlines.group.tomorrow": "Tomorrow",
  "deadlines.group.week": "This week",
  "deadlines.group.later": "Later",
  "deadlines.kind.assign": "Assignment",
  "deadlines.kind.quiz": "Quiz",
  "deadlines.kind.forum": "Forum",
  "deadlines.kind.workshop": "Workshop",
  "deadlines.kind.lesson": "Lesson",

  "sync.now": "Sync now",
  "sync.running": "Syncing…",
  "sync.progress": "Course {index} of {total}",
  "sync.progressFiles": "{done} of {total} files",
  "sync.cancelling": "Stopping after the files in progress…",
  "sync.doneNothing.one": "All up to date · {count} course checked",
  "sync.doneNothing.other": "All up to date · {count} courses checked",
  "sync.doneNews.one": "Sync complete · {count} new or updated file",
  "sync.doneNews.other": "Sync complete · {count} new or updated files",
  "sync.cancelled.one": "Sync cancelled · {count} course done",
  "sync.cancelled.other": "Sync cancelled · {count} courses done",
  "sync.hideDetails": "Hide",
  "error.sync": "Sync failed",
  "error.board": "Checking the announcements failed",
  "error.deadlines": "Loading the deadlines failed",
  "error.courses": "Loading the courses failed",
  "error.links": "Reading the links failed",
  "error.login": "Sign-in failed",
  "error.generic": "Something went wrong",
  "sync.last": "Last sync: {date}",
  "sync.never": "No sync yet",
  "sync.lastSync": "Last sync",
  "sync.details": "Details",
  "sync.newFiles.one": "new file",
  "sync.newFiles.other": "new files",
  "sync.new.one": "new",
  "sync.new.other": "new",
  "sync.updated.one": "updated",
  "sync.updated.other": "updated",
  "sync.adopted.one": "already there",
  "sync.adopted.other": "already there",
  "sync.conflicts.one": "conflict",
  "sync.conflicts.other": "conflicts",
  "sync.errors.one": "error",
  "sync.errors.other": "errors",
  "sync.noChanges": "nothing new",
  "sync.folder": "Folder: {folder}",
  "sync.list.new": "New",
  "sync.list.updated": "Updated",
  "sync.list.conflicts":
    "Conflicts: the new version was saved next to yours, which was left untouched",
  "sync.list.keptLocal": "Changed by you (left untouched)",
  "sync.list.removed": "Removed from WeBeep (your copy stays)",
  "sync.list.errors": "Errors",
  "sync.list.newLinks": "New links (in the Links tab)",
  "sync.removedCount.one": "removed from WeBeep",
  "sync.removedCount.other": "removed from WeBeep",
  "sync.newLinksCount.one": "new link",
  "sync.newLinksCount.other": "new links",

  "cleanup.passed": "You passed",
  "cleanup.nothing": "there are no downloaded files to delete.",
  "cleanup.question.before.one": "Move the",
  "cleanup.question.before.other": "Move the",
  "cleanup.question.after.one":
    "file downloaded from WeBeep ({size}) to the Recycle Bin, even if it was already there " +
    "from v1 and recognised as identical? You can restore it from the Recycle Bin. Files you " +
    "added to the folder yourself are not touched.",
  "cleanup.question.after.other":
    "files downloaded from WeBeep ({size}) to the Recycle Bin, including those already there " +
    "from v1 and recognised as identical? You can restore them from the Recycle Bin. Files you " +
    "added to the folder yourself are not touched.",
  "cleanup.includeModified.one": "Also delete the file you changed",
  "cleanup.includeModified.other": "Also delete the {count} files you changed",
  "cleanup.confirm": "Yes, move to the Recycle Bin",
  "cleanup.keep": "No, keep them",
  "cleanup.later": "Ask me later",
  "cleanup.trashed.one": "{count} file moved to the Recycle Bin ({size}).",
  "cleanup.trashed.other": "{count} files moved to the Recycle Bin ({size}).",
  "cleanup.keptModified.one": "{count} file you changed was left where it was.",
  "cleanup.keptModified.other": "{count} files you changed were left where they were.",
  "cleanup.skipped.one": "{count} file not moved (maybe open in another program).",
  "cleanup.skipped.other": "{count} files not moved (maybe open in another program).",

  "today.feedLabel": "What happened",
  "today.today": "Today",
  "today.yesterday": "Yesterday",
  "today.tomorrow": "Tomorrow",
  "today.atTime": "{day} at {time}",
  "today.files.one": "{count} new or updated file",
  "today.files.other": "{count} new or updated files",
  "today.inCourses.one": "in 1 course",
  "today.inCourses.other": "in {count} courses",
  "today.openFolder": "Open folder",
  "today.show": "Show",
  "today.showOf": "Show {name} in File Explorer",
  "today.conflict":
    "You had changed “{name}”: your copy stays as it is, the new version is saved next to it.",
  "today.readOnBoard": "Read in Announcements",
  "today.newLink": "new link",
  "today.openInBrowser": "Open in the browser",
  "today.seen": "Already seen: last opened {when}",
  "today.nothingNew": "Nothing new since you last opened the app ({when})",
  "today.empty": "Nothing new in the last 7 days.",
  "today.deadlines": "Deadlines",
  "today.deadlinesLoading": "Loading deadlines…",
  "today.allDeadlines": "All deadlines (14 days)",

  "settings.account": "Account",
  "settings.loggedInAs": "Logged in as",
  "settings.logout": "Log out",
  "settings.loggedInAsName": "Signed in as {name}",
  "settings.accountHint":
    "The access token stays only on this PC, in the Windows Credential Manager.",
  "settings.autosyncHint": "Downloads what is new in the selected courses at regular intervals.",
  "settings.notificationsTitle": "System notifications",
  "settings.notificationsHint": "New files, announcements and deadlines within 24 hours.",
  "settings.languageTitle": "Interface language",
  "settings.languageHint": "WeBeep content stays in Italian, as on the site.",
  "settings.folder": "Destination folder",
  "settings.folderPrompt": "Choose where to save the material of your courses.",
  "settings.folderHint":
    "You can choose the folder used by WeBeep Sync v1: files already there and identical (same " +
    "size and date) are not downloaded again, and files you have changed are never " +
    "overwritten. The folder of each course can be changed from the Courses tab.",
  "settings.syncAndNotifications": "Sync and notifications",
  "settings.whileOpen": "While the app is open",
  "settings.autosync": "Sync automatically",
  "settings.interval": "Interval",
  "settings.interval.30": "every 30 minutes",
  "settings.interval.60": "every hour",
  "settings.interval.120": "every 2 hours (recommended)",
  "settings.interval.240": "every 4 hours",
  "settings.interval.480": "every 8 hours",
  "settings.interval.1440": "once a day",
  "settings.notifications":
    "System notifications (new files, announcements, deadlines within 24 hours)",
  "settings.whileOpenHint":
    "The app only works while it is open: when it opens it checks the announcements right " +
    "away and syncs if the last sync is older than the chosen interval. Closing the window " +
    "closes everything.",
  "settings.download": "Downloads",
  "settings.concurrency": "Files downloaded at the same time",
  "settings.recommended": "{value} (recommended)",
  "settings.concurrencyHint": "Higher values don't speed things up much and load WeBeep more.",
  "settings.flatten": "Don't create the “Materiali” folder (like WeBeep Sync v1)",
  "settings.flattenHint":
    "If you change it, at the next sync the files already downloaded are moved to the new " +
    "location (the ones you have changed stay where they are and a new copy is downloaded).",
  "settings.cleanupWhen": "When I mark a course as passed",
  "settings.cleanup.ask": "ask me whether to delete the files",
  "settings.cleanup.always": "move the files to the Recycle Bin right away",
  "settings.cleanup.never": "never delete the files",
  "settings.cleanupHint":
    "Only files downloaded by the app and not changed by you are moved to the Recycle Bin.",

  "dev.title": "Developer tools · Phase 0",
  "dev.rawHint.before":
    "Saves the raw WeBeep responses (courses, filters, structure of the current courses) in",
  "dev.rawHint.after": ". They contain personal data: do not share them.",
  "dev.langHint":
    "In English WeBeep translates default names and texts written in several languages: both " +
    "dumps are needed to compare them.",
  "dev.dumpIt": "Run dump (Italian)",
  "dev.dumpEn": "Run dump (English)",
  "dev.dumping": "Downloading data from WeBeep…",
  "dev.folder": "Folder:",
  "dev.files": "Saved files ({count}): {files}",
  "dev.errors": "Errors ({count}):",

  "support.title": "Problems and reports",
  "support.intro":
    "If something does not work, report it on GitHub: the form opens in the browser with version " +
    "and system already filled in. Attach the diagnostics file: it helps to understand what happened.",
  "support.report": "Report a problem",
  "support.reportShort": "Report",
  "support.rowTitle": "Something not working?",
  "support.export": "Export diagnostics…",
  "support.previewHint":
    "This is what the file contains: personal folders, tokens and emails are already hidden. " +
    "Check it before attaching it.",
  "support.save": "Save…",
  "support.saved": "Saved to {path}. Attach it to the report on GitHub.",
  "update.title": "Updates",
  "update.current": "Installed version: {version}",
  "update.check": "Check for updates",
  "update.autoHint": "The installed app checks by itself at start and every 6 hours.",
  "update.upToDate": "You already have the latest version.",
  "update.available": "WeBeep Sync {version} is available",
  "update.install": "Update now",
  "update.later": "Later",
  "update.installing": "Downloading and installing the update…",
  "update.installingPercent": "Downloading and installing the update… {percent}%",
  "update.failed": "Update failed: {error}",
};
