// Italian texts of the interface: the source of truth. Every key must also exist in `en.ts`.
// Keys are grouped by screen; `<key>.one` / `<key>.other` are singular and plural (see `tn`).
// No markup here: texts around a <strong> are split into `.before` / `.after` keys.
export const it = {
  "language.label": "Lingua",
  "language.it": "Italiano",
  "language.en": "English",

  "common.close": "Chiudi",
  "common.save": "Salva",
  "common.cancel": "Annulla",
  "common.change": "Cambia",
  "common.yes": "Sì",
  "common.no": "No",
  "common.ok": "Ok",
  "common.info": "Informazioni",
  "common.course": "Corso",
  "common.courseFallback": "Corso {id}",
  "common.notSynced": "non sincronizzato",
  "common.openOnWebeep": "Apri su WeBeep",
  "common.chooseFolder": "Scegli cartella…",
  "common.changeFolder": "Cambia cartella…",
  "common.passedCourses": "Corsi superati",
  "common.unofficial": "Progetto non ufficiale, non affiliato al Politecnico di Milano.",

  "nav.label": "Sezioni",
  "nav.courses": "Corsi",
  "nav.links": "Link",
  "nav.board": "Bacheca",
  "nav.deadlines": "Scadenze",
  "nav.settings": "Impostazioni",
  "nav.unread.one": "{count} non letto",
  "nav.unread.other": "{count} non letti",

  "status.checkingSession": "Verifico la sessione…",
  "status.readingSettings": "Leggo le impostazioni…",
  "status.readingBoard": "Leggo la bacheca…",
  "status.checkingPassed": "Controllo i corsi superati…",
  "status.loadingCourses": "Carico i corsi da WeBeep…",
  "status.loggingIn": "Completa l'accesso nella finestra di WeBeep…",
  "status.loggingOut": "Esco…",
  "status.choosingFolder": "Scegli la cartella…",
  "status.savingSelection": "Salvo la selezione…",
  "status.changingFolder": "Cambio la cartella…",
  "status.updatingCourse": "Aggiorno il corso…",
  "status.savingSemester": "Salvo il semestre…",
  "status.saving": "Salvo…",
  "status.trashing": "Sposto i file nel Cestino…",
  "status.savingSettings": "Salvo le impostazioni…",
  "status.syncing": "Sincronizzazione in corso…",
  "status.readingLinks": "Leggo i link…",
  "status.openingLink": "Apro il link nel browser…",
  "status.readingPlan": "Leggo il piano di studi…",
  "status.removingPlan": "Rimuovo il piano di studi…",
  "status.openingCourseFolder": "Apro la cartella del corso…",
  "status.checkingBoard": "Controllo gli avvisi su WeBeep…",
  "status.loadingDeadlines": "Carico le scadenze…",

  "welcome.tagline": "Scarica e tiene aggiornato il materiale dei tuoi corsi di WeBeep.",
  "welcome.login": "Accedi a WeBeep",

  "onboarding.title": "Primo avvio",
  "onboarding.login": "Accedi con il tuo account Polimi",
  "onboarding.folder": "Scegli la cartella dove salvare i corsi (anche quella di WeBeep Sync v1)",
  "onboarding.courses":
    "Controlla i corsi: quelli dell'anno in corso sono già selezionati, divisi per semestre. " +
    "Togli quelli che non ti interessano",
  "onboarding.sync": "Premi «{button}»",

  "courses.noRoot": "Nessuna cartella scelta",
  "courses.notLoaded": "Corsi non ancora caricati.",
  "courses.folderMoved": "Cartella rinominata.",
  "courses.folderRepointed":
    "Il corso ora usa la cartella indicata: al prossimo sync i file già presenti e identici " +
    "verranno adottati senza riscaricarli.",
  "courses.suggestion.before": "Sembra che tu abbia superato",
  "courses.suggestion.after":
    ": il corso d'ufficio non è più tra le tue iscrizioni, mentre resta quello a cui ti eri " +
    "iscritto tu. Segnarlo come superato?",
  "courses.suggestion.yes": "Sì, l'ho superato",
  "courses.selected.one": "corso sincronizzato",
  "courses.selected.other": "corsi sincronizzati",
  "courses.showAll": "Mostra anche superati, ignorati e anni precedenti",
  "courses.selectionHint":
    "La selezione segue lo stato del corso; le tue scelte manuali hanno sempre la precedenza. " +
    "«Superato» e «Ignora» valgono per tutti i corsi dello stesso insegnamento.",
  "courses.col.semester": "Semestre",
  "courses.col.folder": "Cartella",
  "courses.section.current.title": "Questo semestre",
  "courses.section.current.hint": "Sincronizzati di default.",
  "courses.section.upcoming.title": "Prossimo semestre",
  "courses.section.upcoming.hint":
    "Sincronizzati di default: il materiale arriva appena viene pubblicato.",
  "courses.section.examPending.title": "Esami in sospeso",
  "courses.section.examPending.hint":
    "Corsi del semestre scorso: sincronizzati finché non li segni come superati.",
  "courses.section.retired.title": "Superati e ignorati",
  "courses.section.past.title": "Anni precedenti",
  "courses.section.notSynced": "Non sincronizzati.",
  "courses.semester1": "1° semestre",
  "courses.semester2": "2° semestre",
  "courses.semester1Short": "1°",
  "courses.semester2Short": "2°",
  "courses.semesterLabel": "{semester} semestre ({source})",
  "courses.semesterUnknown": "semestre non noto",
  "courses.source.manual": "scelto da te",
  "courses.source.studyPlan": "dal piano di studi",
  "courses.source.fileDates": "dalle date dei file",
  "courses.source.startDate": "dalla data di inizio del corso",
  "courses.semesterOf": "Semestre di {name}",
  "courses.auto": "Automatico",
  "courses.autoWith": "Auto: {semester}",
  "courses.folderName": "Nome della cartella",
  "courses.restore": "Ripristina",
  "courses.passed": "Superato",
  "courses.ignore": "Ignora",
  "courses.confirmPassed.before": "Segnare",
  "courses.confirmPassed.after": "come superato",
  "courses.confirmSiblings.one": "(anche l'altro corso dello stesso insegnamento)",
  "courses.confirmSiblings.other": "(anche gli altri {count} corsi dello stesso insegnamento)",

  "plan.title": "Piano di studi",
  "plan.importButton": "Importa piano di studi",
  "plan.loadedButton": "Piano di studi ✓",
  "plan.infoLabel": "Che cos'è il piano di studi",
  "plan.info":
    "Dal tuo piano degli studi l'app legge se ogni insegnamento è del 1° o del 2° semestre e " +
    "divide i corsi in automatico. Va bene la ricevuta in PDF, la pagina «Piano degli studi» dei " +
    "Servizi Online salvata in PDF (Ctrl+P) oppure la sua tabella copiata e incollata. Si " +
    "tengono solo codici, nomi e semestri degli insegnamenti.",
  "plan.intro":
    "Dal tuo piano degli studi l'app legge se ogni insegnamento è del 1° o del 2° semestre e " +
    "divide i corsi di conseguenza.",
  "plan.loaded.one":
    "Piano caricato il {date}: {count} insegnamento, {first} del 1° semestre e {second} del 2°.",
  "plan.loaded.other":
    "Piano caricato il {date}: {count} insegnamenti, {first} del 1° semestre e {second} del 2°.",
  "plan.remove": "Rimuovi",
  "plan.reading": "Leggo il piano…",
  "plan.dropHere": "Trascina qui il PDF",
  "plan.dropHint":
    "la ricevuta del piano degli studi, oppure la pagina «Piano degli studi» salvata in PDF",
  "plan.dropPdf": "Trascina un file PDF.",
  "plan.chooseFile": "Scegli file…",
  "plan.pasteLabel": "Oppure incolla la tabella degli insegnamenti",
  "plan.pastePlaceholder":
    "Seleziona la tabella nella pagina «Piano degli studi», copiala (Ctrl+C) e incollala qui " +
    "(Ctrl+V)",
  "plan.readTable": "Leggi la tabella",
  "plan.noReceipt": "Non hai la ricevuta?",
  "plan.how.before": "Dai Servizi Online del Polimi non si può più scaricare. Apri invece",
  "plan.how.page": "Carriera didattica → Piano degli studi",
  "plan.how.after": ", poi:",
  "plan.how.paste.label": "per incollare:",
  "plan.how.paste.text":
    "seleziona con il mouse la tabella degli insegnamenti (dall'intestazione all'ultima riga), " +
    "copiala e incollala qui sopra: così l'app vede solo la tabella;",
  "plan.how.pdf.label": "per il PDF:",
  "plan.how.pdf.text":
    "Ctrl+P → «Salva come PDF». Per non includere i tuoi dati, seleziona prima la tabella e " +
    "nella finestra di stampa scegli «Solo selezione» (in «Altre impostazioni»).",
  "plan.note":
    "Si tengono solo codici, nomi e semestri degli insegnamenti: dati personali, docenti e voti " +
    "non vengono salvati, e il file non lascia il tuo PC. Le scelte del semestre fatte a mano " +
    "vengono sostituite dal piano; puoi sempre cambiarle di nuovo dalla lista dei corsi.",
  "plan.imported.one":
    "Fatto: letto {count} insegnamento ({first} del 1° semestre, {second} del 2°).",
  "plan.imported.other":
    "Fatto: letti {count} insegnamenti ({first} del 1° semestre, {second} del 2°).",
  "plan.matched.one": "{count} corso di quest'anno ora segue il piano.",
  "plan.matched.other": "{count} corsi di quest'anno ora seguono il piano.",
  "plan.removed": "Piano rimosso: i semestri tornano automatici.",

  "links.markAllSeen": "Segna tutti come visti",
  "links.hint":
    "Link che i docenti mettono su WeBeep al posto dei file (cartelle OneDrive, registrazioni, " +
    "repository). Si aprono nel browser: i file scaricati da lì mettili nella cartella del " +
    "corso, in una sottocartella tua. L'app non li tocca mai.",
  "links.empty":
    "Nessun link nei corsi sincronizzati. I link compaiono dopo un sync: se un corso ha solo " +
    "link e nessun file, assicurati che sia selezionato nella scheda Corsi.",
  "links.nextSemester": "Corsi del prossimo semestre",
  "links.nextSemesterHint":
    "Le lezioni non sono ancora iniziate: di solito i link (registrazioni comprese) sono ancora " +
    "vuoti.",
  "links.openFolder": "Apri cartella del corso",
  "links.group.material": "Materiale",
  "links.group.recording": "Registrazioni e lezioni online",
  "links.group.other": "Altri link",
  "links.new": "nuovo",

  "board.checkNow": "Controlla ora",
  "board.markAllRead": "Segna tutti come letti",
  "board.listLabel": "Elenco degli avvisi",
  "board.allCourses": "Tutti i corsi",
  "board.unreadOnly": "Solo non letti ({count})",
  "board.mute": "Niente notifiche per questo corso",
  "board.empty":
    "Nessun avviso salvato. L'app controlla gli avvisi dei corsi sincronizzati ogni 15 minuti; " +
    "puoi anche premere «{button}».",
  "board.noMatch": "Nessun avviso con questi filtri.",
  "board.attachments": "Allegati (si scaricano dalla pagina dell'avviso su WeBeep):",
  "board.selectHint": "Seleziona un avviso dall'elenco.",

  "deadlines.refresh": "Aggiorna",
  "deadlines.hint":
    "Consegne e quiz dei prossimi 14 giorni. Ricevi un promemoria 24 ore prima.",
  "deadlines.notLoaded": "Scadenze non ancora caricate.",
  "deadlines.empty": "Nessuna scadenza nei prossimi 14 giorni.",
  "deadlines.col.due": "Scadenza",
  "deadlines.col.activity": "Attività",
  "deadlines.col.kind": "Tipo",
  "deadlines.group.today": "Oggi",
  "deadlines.group.tomorrow": "Domani",
  "deadlines.group.week": "Questa settimana",
  "deadlines.group.later": "Più avanti",
  "deadlines.kind.assign": "Consegna",
  "deadlines.kind.quiz": "Quiz",
  "deadlines.kind.forum": "Forum",
  "deadlines.kind.workshop": "Workshop",
  "deadlines.kind.lesson": "Lezione",

  "sync.now": "Sincronizza ora",
  "sync.running": "Sincronizzazione…",
  "sync.progress": "Corso {index} di {total}",
  "sync.last": "Ultimo sync: {date}",
  "sync.never": "Nessun sync ancora",
  "sync.lastSync": "Ultima sincronizzazione",
  "sync.newFiles.one": "file nuovo",
  "sync.newFiles.other": "file nuovi",
  "sync.new.one": "nuovo",
  "sync.new.other": "nuovi",
  "sync.updated.one": "aggiornato",
  "sync.updated.other": "aggiornati",
  "sync.adopted.one": "già presente",
  "sync.adopted.other": "già presenti",
  "sync.conflicts.one": "conflitto",
  "sync.conflicts.other": "conflitti",
  "sync.errors.one": "errore",
  "sync.errors.other": "errori",
  "sync.noChanges": "nessuna novità",
  "sync.folder": "Cartella: {folder}",
  "sync.list.new": "Nuovi",
  "sync.list.updated": "Aggiornati",
  "sync.list.conflicts":
    "Conflitti: la versione nuova è stata salvata accanto alla tua, che non è stata toccata",
  "sync.list.keptLocal": "Modificati da te (non toccati)",
  "sync.list.removed": "Rimossi da WeBeep (la tua copia resta)",
  "sync.list.errors": "Errori",
  "sync.list.external": "Contenuti esterni (da aprire su WeBeep)",

  "cleanup.passed": "Hai superato",
  "cleanup.nothing": "non ci sono file scaricati da eliminare.",
  "cleanup.question.before.one": "Vuoi spostare nel Cestino il",
  "cleanup.question.before.other": "Vuoi spostare nel Cestino i",
  "cleanup.question.after.one":
    "file scaricato da WeBeep ({size}), anche se era già presente dalla v1 ed è stato " +
    "riconosciuto come identico? Puoi recuperarlo dal Cestino. I file che hai aggiunto tu nella " +
    "cartella non vengono toccati.",
  "cleanup.question.after.other":
    "file scaricati da WeBeep ({size}), compresi quelli già presenti dalla v1 e riconosciuti " +
    "come identici? Puoi recuperarli dal Cestino. I file che hai aggiunto tu nella cartella non " +
    "vengono toccati.",
  "cleanup.includeModified.one": "Elimina anche il file che hai modificato",
  "cleanup.includeModified.other": "Elimina anche i {count} file che hai modificato",
  "cleanup.confirm": "Sì, sposta nel Cestino",
  "cleanup.keep": "No, tienili",
  "cleanup.later": "Chiedimelo più tardi",
  "cleanup.trashed.one": "{count} file spostato nel Cestino ({size}).",
  "cleanup.trashed.other": "{count} file spostati nel Cestino ({size}).",
  "cleanup.keptModified.one": "{count} file modificato da te è rimasto al suo posto.",
  "cleanup.keptModified.other": "{count} file modificati da te sono rimasti al loro posto.",
  "cleanup.skipped.one": "{count} file non spostato (forse aperto in un altro programma).",
  "cleanup.skipped.other": "{count} file non spostati (forse aperti in un altro programma).",

  "settings.account": "Account",
  "settings.loggedInAs": "Accesso effettuato come",
  "settings.logout": "Esci",
  "settings.folder": "Cartella di destinazione",
  "settings.folderPrompt": "Scegli dove salvare il materiale dei corsi.",
  "settings.folderHint":
    "Puoi scegliere la cartella usata da WeBeep Sync v1: i file già presenti e identici (stessa " +
    "dimensione e data) non vengono riscaricati e i file che hai modificato non vengono mai " +
    "sovrascritti. La cartella di ogni corso si cambia dalla scheda Corsi.",
  "settings.syncAndNotifications": "Sincronizzazione e notifiche",
  "settings.whileOpen": "Mentre l'app è aperta",
  "settings.autosync": "Sincronizza automaticamente",
  "settings.interval": "Intervallo",
  "settings.interval.30": "ogni 30 minuti",
  "settings.interval.60": "ogni ora",
  "settings.interval.120": "ogni 2 ore (consigliato)",
  "settings.interval.240": "ogni 4 ore",
  "settings.interval.480": "ogni 8 ore",
  "settings.interval.1440": "una volta al giorno",
  "settings.notifications": "Notifiche di sistema (nuovi file, avvisi, scadenze tra 24 ore)",
  "settings.whileOpenHint":
    "L'app lavora solo mentre è aperta: all'apertura controlla subito gli avvisi e, se l'ultimo " +
    "sync è più vecchio dell'intervallo scelto, sincronizza. Chiudendo la finestra si chiude " +
    "tutto.",
  "settings.download": "Download",
  "settings.concurrency": "File scaricati contemporaneamente",
  "settings.recommended": "{value} (consigliato)",
  "settings.concurrencyHint": "Valori alti non velocizzano molto e caricano di più WeBeep.",
  "settings.flatten": "Non creare la cartella «Materiali» (come WeBeep Sync v1)",
  "settings.flattenHint":
    "Se la cambi, al prossimo sync i file già scaricati vengono spostati nella nuova posizione " +
    "(quelli che hai modificato restano dove sono e ne viene scaricata una copia nuova).",
  "settings.cleanupWhen": "Quando segno un corso come superato",
  "settings.cleanup.ask": "chiedimi se eliminare i file",
  "settings.cleanup.always": "sposta subito i file nel Cestino",
  "settings.cleanup.never": "non eliminare mai i file",
  "settings.cleanupHint":
    "Vengono spostati nel Cestino solo i file scaricati dall'app e non modificati da te.",

  "dev.title": "Strumenti di sviluppo · Fase 0",
  "dev.rawHint.before":
    "Salva le risposte grezze di WeBeep (corsi, filtri, struttura dei corsi in corso) in",
  "dev.rawHint.after": ". Contengono dati personali: non vanno condivise.",
  "dev.langHint":
    "In inglese WeBeep traduce i nomi predefiniti e i testi scritti in più lingue: servono " +
    "entrambi i dump per confrontarli.",
  "dev.dumpIt": "Esegui dump (italiano)",
  "dev.dumpEn": "Esegui dump (inglese)",
  "dev.dumping": "Scarico i dati da WeBeep…",
  "dev.folder": "Cartella:",
  "dev.files": "File salvati ({count}): {files}",
  "dev.errors": "Errori ({count}):",

  "support.title": "Problemi e segnalazioni",
  "support.intro":
    "Se qualcosa non funziona, segnalalo su GitHub: il modulo si apre nel browser con versione e " +
    "sistema già compilati. Allega il file di diagnostica: aiuta a capire cosa è successo.",
  "support.report": "Segnala un problema",
  "support.reportShort": "Segnala",
  "support.export": "Esporta diagnostica…",
  "support.previewHint":
    "Ecco cosa contiene il file: cartelle personali, token ed email sono già nascosti. " +
    "Controllalo prima di allegarlo.",
  "support.save": "Salva…",
  "support.saved": "Salvato in {path}. Allegalo alla segnalazione su GitHub.",
  "update.title": "Aggiornamenti",
  "update.current": "Versione installata: {version}",
  "update.check": "Controlla aggiornamenti",
  "update.upToDate": "Hai già l'ultima versione.",
  "update.available": "È disponibile WeBeep Sync {version}",
  "update.install": "Aggiorna ora",
  "update.later": "Più tardi",
  "update.installing": "Scarico e installo l'aggiornamento…",
  "update.installingPercent": "Scarico e installo l'aggiornamento… {percent}%",
  "update.failed": "Aggiornamento non riuscito: {error}",
} as const;
