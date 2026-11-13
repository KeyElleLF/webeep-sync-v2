# WeBeep Sync V2 — Documento di progetto

> Questo file è il brief completo per sviluppare il progetto. Viene caricato automaticamente
> da Claude Code in ogni sessione aperta in questa cartella. Tienilo aggiornato: quando una
> verifica della Fase 0 conferma o smentisce un'ipotesi qui sotto, correggi la sezione relativa.

---

## 0. Stato attuale e note per la prossima sessione (aggiornato 2026-11-09)

**Leggere per primo.** Le sezioni 1–14 sono il brief originale, con le decisioni e le verifiche
annotate in corsivo nel piano di lavoro (§11).

### A che punto siamo
| Fase | Stato |
|---|---|
| 0 – setup, login, dump, fixture | ✅ fatta |
| 1 – MVP sync | ✅ fatta e **provata sul sito reale**: 436 file, 0 errori; il secondo sync non scarica nulla |
| 2 – semestri, stati, pulizia dei corsi superati | ✅ fatta, provata dall'utente (la divisione per semestre funziona) |
| 3 – bacheca, scadenze, notifiche | ✅ fatta, provata in `tauri dev`. **Niente background** (vedi sotto) |
| 5 – app installabile | ✅ installer NSIS creato; **non ancora provato installato** dall'utente |
| 4 – CI, release, updater, Mac/Linux, diagnostica, i18n EN | ⏳ **da fare: è il prossimo passo** |

Storia riscritta il 2026-11-13 in un unico commit pubblico (`99e1428`, versione 0.1.3) in vista della
pubblicazione; identità git anonima (`…@users.noreply.github.com`). Guida per sviluppatori in
inglese: `docs/ARCHITECTURE.md`, `CONTRIBUTING.md`. Test (allora 199) tutti verdi; clippy
`-D warnings` pulito; `npm run check` pulito. Copertura circa 81% complessiva, 93–100% sulla
logica di sync e corsi.

### Decisioni dell'utente da rispettare
- **Niente background:** chiudere la finestra chiude l'app. Niente tray né avvio con Windows. I
  controlli periodici (avvisi ogni 15 min, scadenze ogni ora, sync automatico) girano **solo a
  finestra aperta**. Non reintrodurli.
- **Ogni commit va pubblicato** con `git push` sul repo **pubblico**
  `github.com/KeyElleLF/webeep-sync-v2` (remote `origin`, branch `main`; pubblicato il
  2026-11-13). **Il repository è pubblico:** prima di ogni commit controllare che non ci siano
  `fixtures/raw/`, token, dati personali, nomi di docenti o percorsi personali (vanno in
  `CLAUDE.local.md`, ignorato da git). Commit con l'identità anonima già impostata nel repository.
  Il vecchio repository privato (`webeep-sync-v2-private`, remote `private`) è solo un archivio:
  niente più push lì.
- **Il materiale non va mai perso:** nessuna sovrascrittura dei file modificati, eliminazioni solo
  via Cestino e solo dei file scaricati dall'app (§6.5, §7).
- **«Materiali» appiattita di default**, come la v1, per compatibilità con `Desktop\polimi`.
- **Contenuti di WeBeep sempre in italiano** (verificato il 2026-11-13, vedi «Verifiche»): in
  inglese cambiano i nomi delle sezioni e di alcune cartelle, quindi i percorsi dei file. La lingua
  dell'interfaccia traduce solo i testi dell'app; i nomi standard del modello Polimi (Materiali,
  Registrazioni, Consegne, Bacheca, Annunci, Archivio registrazioni) sono tradotti **solo in
  visualizzazione** (`displayName` in `src/lib/i18n/index.svelte.ts`).
- **Interfaccia in italiano e inglese** (2026-11-13): impostazione `language` nel database
  (comandi `get_language`/`set_language`; al primo avvio la lingua di Windows). Frontend:
  `src/lib/i18n/` con `it.ts` (testo di riferimento) ed `en.ts` (stesse chiavi, controllate da
  TypeScript), `t()`, `tn()` per singolare/plurale, `locale()` per date e numeri. Backend:
  `src-tauri/src/i18n.rs`, lingua globale caricata all'avvio; i messaggi restano scritti in
  italiano nel codice e la tabella `EN` dà l'inglese (con `{}` per le parti variabili), applicata a
  tutti gli errori dei comandi (`CommandError`), agli errori dei singoli file e ai titoli delle
  finestre; le notifiche usano `pick()`. **Quando si aggiunge un messaggio**: chiave in `it.ts` e
  `en.ts`, oppure riga nella tabella `EN` (un test controlla che ogni riga si traduca). I nomi
  delle copie di conflitto sul disco («WeBeep aggiornato») restano in italiano.
- **Interfaccia sempre scura con accento viola** (2026-11-12), indipendente dal tema di Windows:
  palette in `src/lib/styles.css` (`--accent` viola chiaro per testo e link, `--accent-strong` per
  i pulsanti pieni con testo bianco, contrasto ≥ 5:1), `color-scheme: dark`, finestra con
  `"theme": "Dark"` e `backgroundColor` in `tauri.conf.json`. Niente tema chiaro.
- La v1 e i dati dell'utente: cartella di sync della v1 in `CLAUDE.local.md`. La prima prova reale
  è stata fatta in una cartella nuova; l'adozione dei file della v1 è verificata solo offline
  (76 file su 76 hanno dimensione e mtime uguali a WeBeep).

### Come si lavora qui (comandi e trappole)
- Sviluppo: `npm run tauri dev`. Se la porta 1420 è occupata, terminare il processo `node`/vite
  rimasto e `webeep-sync.exe`.
- Installer: **`npm run build:installer`** (non `npm run tauri build` da solo: lo script
  `scripts/build-installer.mjs` aggiunge `--remap-path-prefix=<home>=~`, altrimenti l'exe contiene
  il nome utente di Windows nei percorsi di `.cargo\registry`; verifica: `grep -a -c <nomeutente>`
  sull'exe deve dare 0) →
  `src-tauri/target/release/bundle/nsis/WeBeep Sync_<versione>_x64-setup.exe` (5,7 MB dalla 0.1.3, prima 4,3; l'exe
  installato pesa circa 21 MB dalla 0.1.3: +8 MB per la lettura dei PDF e `panic = "unwind"`). La versione si cambia in `src-tauri/tauri.conf.json`, `Cargo.toml` e
  `package.json`.
- Test e qualità: `cd src-tauri; cargo test; cargo clippy --all-targets -- -D warnings; cargo fmt`,
  poi `npm run check`. Copertura: `cargo llvm-cov --summary-only`.
- **Shell = Windows PowerShell 5.1.** Non riscrivere i file con `Get-Content`/`Set-Content`:
  corrompono l'UTF-8 (è → Ã¨) e aggiungono il BOM. Usare Edit/Write; per gli script node scriverli
  nello scratchpad (evitare `node -e` con virgolette annidate). Messaggi di commit con virgolette:
  `git commit -F <file>`. Dopo modifiche estese, cercare `Ã`, `Â`, `â€` e BOM nei sorgenti.
- `cargo add tauri --no-default-features` **rompe** tauri: la riga giusta è
  `tauri = { version = "2", features = [] }`.
- **Migrazioni SQLite** (`store/mod.rs`, `MIGRATIONS`): ce ne sono 5 (4ª `course_link` e
  `course_link_import`, 5ª `study_plan`, entrambe del 2026-11-12). **Mai modificarne una esistente**: solo aggiungerne di nuove
  in coda. Attenzione: una migrazione nuova applicata da `tauri dev` fa rifiutare il DB alle versioni
  installate più vecchie («database di una versione più recente»): dopo serve un installer nuovo.
- Dati dell'app: DB `%APPDATA%\it.webeepsync.app\webeep-sync.sqlite`; log
  `%LOCALAPPDATA%\it.webeepsync.app\logs`; token nel Gestore credenziali (servizio
  `it.webeepsync.app`). `tauri dev` e l'app installata **condividono** DB e token (stesso identifier).
- Dump della Fase 0 (solo in dev, nella scheda Impostazioni): salva in `fixtures/raw/<data>/`
  (gitignored, contiene dati personali). Fixture redatte: `node scripts/redact-fixtures.mjs <dir>`
  (corsi, contenuti, consegne) e `node scripts/redact-phase3.mjs <dir>` (avvisi, scadenze). Entrambi
  falliscono se trovano nomi rimasti.
- Prima di commit che toccano auth, rete o cancellazione di file: far fare una review di sicurezza
  (agente `ecc:security-reviewer`). Quelle già fatte hanno portato a: ricontrollo del file prima di
  sostituirlo, verifica della dimensione scaricata, token solo verso `/webservice/pluginfile.php/`,
  pulizia confermata solo se c'è una richiesta in sospeso.

### Mappa del codice (reale, sostituisce in parte §10)
- `src-tauri/src/`
  - `auth/`: finestra di login (`window.rs`, navigazione ristretta), parsing e firma del token
    (`token.rs`), keyring (`store.rs`).
  - `moodle/`: client REST (`client.rs`), errori, tipi (`types.rs`, `notify_types.rs`).
  - `courses/`: `name.rs` (parsing del fullname), `semester.rs` (stima del semestre), `state.rs`
    (stati), `archived.rs` (suggerimento «superato»), `mod.rs` (lista, selezione, nomi cartella),
    `study_plan.rs` (testo del piano di studi → insegnamenti, abbinamento per codice o nome),
    `career_page.rs` (pagina «Piano degli studi» stampata in PDF o tabella incollata),
    `plan_file.rs` (lettura del PDF con `pdf-extract`, limite 5 MB, panico intercettato).
  - `sync/`:
    - logica pura: `tree.rs`, `extract.rs`, `assign.rs`, `paths.rs`, `plan.rs`, `cleanup.rs`,
      `links.rs` (link esterni: materiale / registrazioni / altro, per parole e host);
    - I/O: `local.rs` (`safe_join`), `download.rs`, `execute.rs`, `run.rs` (orchestrazione),
      `folders.rs` (cambio cartella), `retire.rs` (superato/ignorato e pulizia).
  - `notify/`: `html.rs` (pulizia HTML con ammonia), `announcements.rs`, `deadlines.rs`,
    `links.rs` (testo della notifica dei link nuovi).
  - `store/`: SQLite (`mod.rs` con migrazioni e baseline; `courses.rs`, `links.rs`, `notify.rs`,
    `settings.rs`, `study_plan.rs`).
  - `commands/`: comandi Tauri sottili (`session`, `courses`, `settings`, `sync`, `teaching`,
    `notify`, `links`, `study_plan`, `dump`, `shared`).
  - `background.rs`: controlli periodici a finestra aperta. `logging.rs`: log su file.
    `window.rs`: istanza singola. `state.rs`: `AppState` (Store + guard «un sync alla volta»).
- `src/`: `routes/+page.svelte` (guscio dell'app e stato: schede Corsi, Link, Bacheca, Scadenze,
  Impostazioni), `lib/api.ts` (wrapper tipizzati di tutti i comandi), `lib/components/*`,
  `lib/format.ts`, `lib/links.ts` (link sempre nel browser), `lib/styles.css`.
  **Layout da desktop** (2026-11-12): barra laterale (`Sidebar.svelte`, con «Sincronizza ora»),
  contenuto a tutta larghezza, barra di stato in basso; corsi e scadenze in righe a colonne
  (`--course-cols` in `CourseList.svelte`), bacheca a due pannelli (elenco + lettura, frecce ↑↓),
  pulizia dei corsi superati in un modale (fuoco su «Chiedimelo più tardi»). Finestra 1180×760,
  minimo 960×560; sotto i 1050 px barra laterale e colonne si stringono. Niente layout a colonna
  stretta: non tornare a `max-width` sul contenuto.
- Provare l'interfaccia **senza account**: `.claude/launch.json` (config `vite`) nel browser
  integrato, poi da JavaScript si definisce un finto `window.__TAURI_INTERNALS__` (`invoke` che
  risponde ai nomi dei comandi di `api.ts` con dati inventati) e si monta a mano
  `/src/routes/+page.svelte` con `mount` di `/node_modules/.vite/deps/svelte.js?v=<hash>`.

- **Scheda Link** (2026-11-12, richiesta dell'utente): i moduli `url` esterni (cartelle OneDrive,
  registrazioni, GitHub, Kaggle…) salvati a ogni sync (`store/links.rs`), con il nome che il docente
  ha dato su WeBeep, raggruppati per corso in Materiale / Registrazioni e lezioni online / Altri link
  (chiuso). Solo corsi attivi; solo link http(s). Al primo sync di un corso nessun link è «nuovo»;
  dopo, i link aggiunti dal docente hanno l'etichetta «nuovo» (badge nella barra laterale) finché non
  si aprono, e il sync automatico manda una notifica (non per i corsi silenziati). «Apri cartella del
  corso» crea la cartella se manca e la apre in Esplora file: lì l'utente mette i file scaricati a
  mano, che l'app non tocca mai (sono fuori dalla baseline).

- **Piano di studi** (2026-11-12, richiesta dell'utente): pulsante «Importa piano di studi» con una
  «i» (`InfoTip.svelte`) sopra la lista dei corsi; finestra (`StudyPlanDialog.svelte`) con
  trascinamento (eventi `onDragDropEvent` di Tauri, attivi solo a finestra aperta) o «Scegli file…».
  Si legge la «Ricevuta di presentazione piano degli studi» (PDF dei Servizi Online, stesso formato per
  tutti): righe `2026 - 2° anno 1° semestre 056892 DATA MINING 5 CFU`, nome anche sulla riga dopo.
  Si salvano solo codice, nome, anno e semestre (niente anagrafica). Ordine del semestre: scelta
  manuale > piano di studi > date dei file > `startdate`; l'importazione azzera le scelte manuali del
  semestre. Abbinamento: codice del corso o del padre, poi nome normalizzato (MIDA 2° modulo ha
  codice diverso su WeBeep). Sul piano reale: 20 insegnamenti, tutti i 16 corsi dell'AA abbinati;
  Machine Learning risulta del **2°** semestre. Fixture redatta in `fixtures/study_plan/`; il PDF vero
  si prova a mano con il test `real_plan_pdf` (ignorato, `WEBEEP_PLAN_PDF=<percorso>`).
  Release con `panic = "unwind"` (non più `abort`) perché un PDF strano non chiuda l'app.
  Nella scheda Link i corsi del prossimo semestre stanno in una sezione chiusa in fondo.
  **La ricevuta non si può più scaricare** dai Servizi Online (verificato il 2026-11-12), quindi si
  accettano anche: la pagina **Carriera didattica → Piano degli studi** salvata in PDF con Ctrl+P
  (layout «a schede»: etichetta prima di ogni valore, nomi a capo, intestazione e piè di pagina
  del browser) e la sua **tabella copiata e incollata** (una cella per riga, celle vuote assenti;
  comando `import_study_plan_text`, max 200 KB). `career_page.rs` unisce le righe e legge
  codice → nome → tre celle (corso di laurea, PSPA, Posins) → «1° Sem»/«2° Sem», solo **dopo**
  l'intestazione «Codice ins.» (sopra, nel PDF, c'è la matricola a 6 cifre). Sui file veri: 20
  insegnamenti su 20, stessi semestri della ricevuta. Fixture redatte (senza docenti, voti, date):
  `fixtures/study_plan/career_page_{pasted,pdf}.txt`. Immagini/OCR scartate dall'utente.

- **Segnalazione dei bug** (2026-11-13): modulo di Issue `.github/ISSUE_TEMPLATE/bug_report.yml`
  (bilingue; campi con id `what`, `steps`, `expected`, `error`, `version`, `os`, `diagnostics`,
  precompilabili dall'URL). Nell'app: Impostazioni → «Problemi e segnalazioni» (`SupportPanel`)
  con «Segnala un problema» e «Esporta diagnostica…» (anteprima, poi salvataggio in `.txt`), e
  link «Segnala» accanto agli errori nella barra di stato (`bugReportUrl` in `src/lib/links.ts`,
  `REPO_URL` da cambiare nei fork). La diagnostica (`diagnostics/report.rs`, comando in
  `commands/support.rs`): versione, sistema, impostazioni, conteggi, ultime 1500 righe di log;
  cartella utente → `~`, token ed email oscurati. Niente invio diretto dall'app (servirebbe un
  token nell'exe). Le Issue funzionano solo quando il repository è pubblico.

- **Aggiornamenti automatici e rilasci** (2026-11-13): `tauri-plugin-updater` +
  `tauri-plugin-process`; endpoint `…/releases/latest/download/latest.json` del repository
  pubblico, `installMode: passive`, chiave pubblica in `plugins.updater.pubkey` di
  `tauri.conf.json`. L'app installata controlla all'avvio e ogni 6 ore (solo build di release:
  `import.meta.env.PROD`), mostra un avviso con le note (`UpdateBanner`, `src/lib/updates.svelte.ts`);
  Impostazioni → «Aggiornamenti» con versione e «Controlla aggiornamenti». Rilascio:
  `npm run release -- X.Y.Z "note"` (`scripts/release.mjs`: versione nei 5 file, commit, tag
  annotato con le note, push) → workflow `.github/workflows/release.yml` (tauri-action, Windows,
  firma con i secrets `TAURI_SIGNING_PRIVATE_KEY`/`_PASSWORD`, `tauri.release.conf.json` attiva
  `createUpdaterArtifacts` solo in CI, così `npm run build:installer` in locale non ha bisogno della
  chiave). CI su ogni push: `.github/workflows/ci.yml` (check, build del frontend, fmt, clippy,
  test). **La chiave privata la tiene solo l'utente** (file fuori dal repo + secrets di GitHub):
  mai leggerla né copiarla. Le versioni ≤ 0.1.3 non hanno l'updater: la 0.2.0 va installata a mano.

### Limiti noti e idee rimaste aperte
- Le cartelle OneDrive/SharePoint di Polimi non si possono scaricare dall'app (servono il login
  Microsoft del Politecnico e un'app registrata con il consenso dell'amministratore): la scheda Link
  dà solo il link. Non si vede quando il docente aggiunge file dentro una cartella OneDrive. I link
  scritti nel testo delle sezioni o delle etichette (non moduli `url`) non sono raccolti.
- Gli allegati degli avvisi non si scaricano dall'app: «Apri su WeBeep» nel browser. Il clic sulla
  notifica non apre l'avviso specifico.
- Junction e symlink dentro la cartella di sync non sono controllati. Nessun blocco se si sceglie
  `C:\` come radice.
- L'euristica del semestre non usa «nessun materiale»: senza piano di studi i corsi senza file
  datati restano «non noto» (si corregge a mano o importando il piano).
- Possibile riduzione dell'exe (21 MB dalla 0.1.3, era 13): la libreria PDF e `unwind` pesano circa 8 MB.
- Fase 4 (prossima): GitHub Actions (fmt, clippy, test su push; build e release sui tag `v*`),
  updater firmato (`tauri signer generate`: la chiave privata va **solo** nei secrets di GitHub e
  richiede un intervento dell'utente), build per macOS e Linux, «Esporta diagnostica» redatta,
  i18n in inglese. Per la condivisione: ripulire i nomi dei docenti in questo file e decidere con
  l'utente se rendere pubblico il repo.

---

## 1. Obiettivo

Riscrivere da zero (in **Rust + Tauri 2**) un client desktop che sincronizza in locale il
materiale didattico di **WeBeep** (`https://webeep.polimi.it`, il Moodle del Politecnico di
Milano), migliorando l'app open source esistente **WeBeep Sync v1** di toto04
(https://github.com/toto04/webeep-sync).

Il tool verrà **condiviso con i compagni di corso**: deve essere installabile da chi non è
sviluppatore, sicuro nella gestione delle credenziali e mantenibile.

### Problemi della v1 da risolvere (segnalati dall'utente)

1. **Nessun filtro sui corsi attivi.** La v1 mostra *tutte* le iscrizioni (l'utente ne ha 48).
   Caso tipico: lo studente è assegnato d'ufficio al corso del suo scaglione e si iscrive in
   **iscrizione spontanea** al corso dello stesso insegnamento tenuto da un altro docente.
   Superato l'esame, il corso d'ufficio viene archiviato e sparisce, quello spontaneo no e
   resta in lista per sempre. Serve un filtro analogo a quello della dashboard WeBeep
   ("Anno accademico corrente" / "In corso").
2. **Sottocartelle e cartelle compresse a volte non vengono scaricate.** Nella sezione
   principale "Materiali" possono esserci file singoli o sottocartelle. Spesso vengono
   scaricati solo i file singoli.
3. **Notifiche/avvisi**: l'utente vuole avvisi della bacheca (forum "Avvisi") anche come
   notifiche in app/di sistema.

### Richiesta aggiuntiva: semestri

Riconoscere se un corso è del **1° o del 2° semestre** e **consigliare** quali corsi
sincronizzare. Va considerato che nel 2° semestre si possono ancora dare esami del 1°, quindi
il sistema propone e l'utente decide. Vedi §6.

---

## 2. Decisioni già prese

| Tema | Decisione | Motivo |
|------|-----------|--------|
| Approccio | Riscrittura da zero, **non** fork | Il codice utile della v1 sono circa 300 righe. Le modifiche toccano ogni parte (corsi, albero file, notifiche, errori) |
| Stack | **Tauri 2 + Rust** (backend), frontend web leggero | Leggerezza: binario di circa 10 MB contro i circa 100 MB di Electron, poca RAM in background |
| Frontend | **Svelte 5 + Vite + TypeScript** (proposta, confermabile) | Bundle piccolo, niente runtime pesante. Node è già installato |
| Licenza | **GPL-3.0**, con credito a toto04 nel README | La v1 è GPL-3.0 (il file `LICENSE` prevale sul campo `"license": "MIT"` del package.json). Usiamo la v1 come riferimento per capire le API |
| Piattaforme | Windows prima. macOS e Linux via CI in seguito | L'utente è su Windows 11, i compagni possono avere Mac |
| Lingua UI | Italiano, con i18n predisposto per l'inglese | Studenti internazionali |
| Codice | Identificatori e commenti in inglese, UI e documentazione utente in italiano | |

---

## 3. Prerequisiti di sviluppo: ✅ INSTALLATI E VERIFICATI (2026-11-09)

| Strumento | Versione |
|---|---|
| VS 2022 Build Tools (workload C++) | 17.14, in `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools` |
| Rust (rustup, toolchain stable msvc) | 1.99.1, componenti `clippy`, `rustfmt`, `llvm-tools-preview` |
| cargo-binstall | 1.26.2 (installa crate CLI precompilati: `cargo binstall <crate>`) |
| cargo-nextest / cargo-llvm-cov | 0.9.151 / 0.9.1 |
| Tauri CLI (`cargo tauri`) | 2.12.4 |
| Node / npm | 24.20 / 11.19 |
| WebView2 | 157 |

Verifica eseguita: un'app Tauri 2 di prova (template vanilla) e un crate con `rusqlite` bundled
compilano ed eseguono correttamente. Se un terminale non trova `cargo`, va riaperto (il PATH
utente include `%USERPROFILE%\.cargo\bin`).

Istruzioni originali, utili solo per reinstallare su un'altra macchina:

1. **Visual Studio Build Tools 2022** con il workload *"Sviluppo di applicazioni desktop con C++"*
   (MSVC + Windows SDK). `winget install Microsoft.VisualStudio.2022.BuildTools` e poi aggiungi il
   workload dal Visual Studio Installer.
2. **Rust** via rustup: `winget install Rustlang.Rustup`, poi `rustup default stable`.
3. **WebView2**: già presente su Windows 11.
4. `cargo install create-tauri-app` oppure `npm create tauri-app@latest`.
5. Strumenti consigliati: `cargo install cargo-nextest cargo-llvm-cov`, `rustup component add clippy rustfmt`.

Verifica finale: `cargo --version`, `rustc --version`, `npm run tauri dev` sull'app scaffold.

---

## 4. Come funziona WeBeep (Moodle) — conoscenza ricavata dalla v1 e da verifiche

### 4.1 Info pubbliche del sito (verificate il 2026-11-09)

`tool_mobile_get_public_config` (endpoint senza login) restituisce:
- `sitename = WeBeep`, `httpswwwroot = https://webeep.polimi.it`
- `launchurl = https://webeep.polimi.it/admin/tool/mobile/launch.php`
- `typeoflogin = 2`: login tramite browser/SSO, come l'app Moodle ufficiale
- `release`/`version`: **nascosti**. La versione si legge dopo il login da
  `core_webservice_get_site_info` (campi `release`, `version`). **Da verificare in Fase 0**:
  serve a capire se ci sono le *sottosezioni* di Moodle 4.5+.

Chiamata usata:
```
POST https://webeep.polimi.it/lib/ajax/service-nologin.php?info=tool_mobile_get_public_config
Content-Type: application/json
[{"index":0,"methodname":"tool_mobile_get_public_config","args":{}}]
```

### 4.2 Login: ottenere il token del web service mobile

È lo stesso meccanismo dell'app Moodle ufficiale. Riferimento: `_riferimenti/webeep-sync-v1/src/modules/login.ts`.

Flusso v1:
1. Apre una finestra embedded su `http://webeep.polimi.it/auth/shibboleth/index.php`
   (SSO Polimi: AUNICA, eventuale SPID e 2FA, tutto gestito dalla pagina stessa).
2. Quando la finestra raggiunge `https://webeep.polimi.it/my/` (login completato), la
   reindirizza a:
   `https://webeep.polimi.it/admin/tool/mobile/launch.php?service=moodle_mobile_app&passport=<random>`
3. Moodle risponde con un redirect a `moodlemobile://token=<BASE64>`.
4. `BASE64` decodificato vale `<signature>:::<token>[:::<privatetoken>]`. Il token è il secondo campo.

Implementazione in Tauri 2:
- `WebviewWindowBuilder` con `.on_navigation(|url| ...)`: se `url.scheme() == "moodlemobile"`,
  estrai il token, chiudi la finestra e restituisci `false` (blocca la navigazione).
  Se l'URL è `/my/`, naviga verso `launch.php`.
- **Variante più pulita da provare per prima:** aprire direttamente `launch.php?...`. Moodle
  forza il login e poi torna a `launch.php`, quindi non serve intercettare `/my/`. Se con
  Shibboleth non funziona, usa il flusso v1.
- `launch.php` accetta anche `urlscheme=<schema>`, utile se WebView2 non segnala la
  navigazione verso `moodlemobile://`.
- **Sicurezza:** `passport` deve essere casuale (es. 16+ caratteri). Verifica la firma:
  `signature == md5(httpswwwroot + passport)`, come fa l'app ufficiale. **Da verificare**:
  alcune installazioni usano `wwwroot` senza https. Se la firma non torna, rifiuta il token.
- Token `invalidtoken` in risposta: richiedi un nuovo login (la v1 apre la finestra in automatico).
- Usa una sessione/data directory del webview **dedicata** (non quella del browser di sistema)
  e offri "Esci" che cancella token e cookie.

**Salvataggio del token:** crate `keyring` (Windows Credential Manager, macOS Keychain, Linux
Secret Service). Nota da BeepBar: su macOS con build **non firmate** il Keychain richiede di
nuovo il permesso a ogni aggiornamento, perché la firma cambia. Valutare per macOS un file con
permessi `0600` nella data dir. **Mai** loggare il token.

### 4.3 Chiamare le API

```
POST https://webeep.polimi.it/webservice/rest/server.php
form: wstoken=<token>, wsfunction=<nome>, moodlewsrestformat=json,
      moodlewssettingfilter=true, moodlewssettinglang=it, ...parametri
```
- Le risposte di errore sono **HTTP 200** con JSON `{"exception":..., "errorcode":..., "message":...}`.
  Vanno sempre controllate.
- Array nei form: `courseids[0]=1&courseids[1]=2`.
- **Download file:** l'URL `fileurl` (forma `.../webservice/pluginfile.php/...`) va chiamato
  con il parametro query **`token=`** (non `wstoken`). Un URL `.../pluginfile.php/...` trovato
  nell'HTML (etichette, pagine) va convertito in `/webservice/pluginfile.php/...` più `token=`.
- Timeout v1 di 10 s per chiamata: troppo stretto con molte chiamate in parallelo. Usa
  timeout più ampi, retry con backoff esponenziale e un limite di concorrenza.

### 4.4 Funzioni API rilevanti

| Funzione | Uso |
|----------|-----|
| `core_webservice_get_site_info` | `userid`, `fullname`, `release` (versione Moodle) |
| `core_enrol_get_users_courses` (`userid`) | Tutte le iscrizioni. Usata dalla v1, ed è la causa del problema 1. Restituisce anche `startdate`, `enddate`, `category`, `lastaccess`, `hidden`, `completed` |
| `core_course_get_enrolled_courses_by_timeline_classification` | **Il filtro della dashboard.** `classification` = `all`, `inprogress`, `future`, `past`, `favourites`, `hidden`, `customfield` (più `customfieldname`, `customfieldvalue`). Parametri: `limit`, `offset`, `sort` |
| `core_course_get_courses_by_field` (`field=ids`, `value=1,2,3`) | Dettagli corso inclusi i **`customfields`**. Probabile fonte di anno accademico e semestre |
| `core_course_get_contents` (`courseid`) | Struttura: sezioni, moduli, `contents` (file) |
| `mod_assign_get_assignments` (`courseids[]`) | Allegati delle consegne (`introattachments`, `introfiles`). **Ignorati dalla v1** |
| `mod_forum_get_forums_by_courses` (`courseids[]`) | Trovare il forum `type == "news"` ("Avvisi") di ogni corso |
| `mod_forum_get_forum_discussions` (`forumid`, `sortorder`, `page`, `perpage`) | Avvisi, inclusi allegati |
| `message_popup_get_popup_notifications` (`useridto=0`) | Notifiche generiche (la v1 tiene solo `eventtype == "posts"`) |
| `core_message_mark_notification_read` | Segna come letta |
| `core_calendar_get_action_events_by_timesort` (`timesortfrom`, `timesortto`, `limitnum`) | Scadenze (consegne, quiz) |
| `core_course_check_updates` / `core_course_get_updates_since` | Ottimizzazione: sapere se un corso è cambiato senza riscaricare la struttura |

### 4.5-bis ✅ RISOLTO: il filtro "Anno accademico corrente"

È `classification = "inprogress"` (il filtro "In corso" di Moodle con un'etichetta diversa).
**Regola per l'app**, che definisce i corsi attivi:
```
core_course_get_enrolled_courses_by_timeline_classification
  classification=inprogress, offset=0, limit=0, sort=fullname
→ poi tenere solo i corsi con coursecategory == anno accademico corrente
  (AA corrente: "YYYY-YY", cambia a settembre, giorno esatto configurabile)
```
Il secondo filtro serve perché i corsi Polimi hanno `enddate = 0` e potrebbero non uscire mai
da "in corso" (vedi la tabella Verifiche). Mostrare comunque all'utente un'opzione "tutti i
corsi" per l'override manuale. `requiredfields` è usato dalla dashboard via AJAX; nel web
service REST va provato, e se non è accettato si omette.

### 4.5 Come è stato scoperto il filtro (procedura storica, utile per altri filtri)

Il filtro è probabilmente un **custom field** di corso usato dal blocco "Panoramica corsi" (`block_myoverview`).
Il modo più affidabile per saperlo:
1. Apri `https://webeep.polimi.it/my/courses.php` nel browser, F12, scheda **Rete**, filtra `service.php`.
2. Seleziona il filtro "Anno accademico corrente" nella dashboard.
3. Apri la richiesta `core_course_get_enrolled_courses_by_timeline_classification` e copia il
   **payload** (`classification`, `customfieldname`, `customfieldvalue`) e un estratto della risposta.
4. Annota qui il risultato (sezione 11, "Verifiche").

---

## 5. Perché la v1 perde file (analisi del codice `moodle.ts` / `download.ts`)

Ipotesi da **confermare in Fase 0** con il dump JSON di un corso reale. Caso di test indicato
dall'utente: **corso 056892 "Data Mining"**, che ha un albero di cartelle su WeBeep.

1. **Sottosezioni (Moodle 4.5+, `modname == "subsection"`):** la sottosezione compare come
   modulo *senza contents*. Il suo contenuto è una **sezione separata** nella risposta di
   `core_course_get_contents` (campo `component == "mod_subsection"`, `itemid` = id
   dell'istanza del modulo). La v1 salta il modulo vuoto e tratta la sezione delegata come
   sezione di primo livello, quindi i file finiscono fuori posto o vengono ignorati.
   **Fix:** costruire l'albero sezione → sottosezione → modulo.
2. **Moduli esclusi a priori:** la v1 scarta `page`, `label`, `url`, `forum`, `lesson`, ecc.
   I docenti spesso mettono PDF e zip in un'**etichetta** (link `pluginfile.php` dentro
   `description`), in una **pagina** (file in `contents` oltre a `index.html`) o in un **URL**
   che punta a un file di WeBeep. **Fix:** estrarre i link `pluginfile.php` dall'HTML e scaricare
   gli URL interni a WeBeep. Gli URL esterni (Drive, OneDrive, YouTube, Teams) vanno elencati
   come "contenuti esterni" senza scaricarli.
   **✅ Confermato il 2026-11-09 per `page`:** è la causa reale del problema 2 (vedi Verifiche).
   Per `label` e `url` nessun caso reale finora, ma la gestione resta prevista.
3. **Consegne (`assign`):** gli allegati del testo non arrivano da `core_course_get_contents`.
   **Fix:** `mod_assign_get_assignments`.
4. **Cartelle (`folder`):** i file hanno `filepath` tipo `/sub/dir/`. La v1 fa
   `path.join(modulename, filepath)`, che è corretto, ma verifica sul caso Data Mining che
   l'albero venga ricostruito per intero, anche su più livelli.
5. **Un errore blocca tutto:** `Promise.all` su tutti i corsi. Un timeout su un corso
   annulla l'intera sincronizzazione. **Fix:** isolamento per corso, errori riportati nel riepilogo.
6. **Rilevamento modifiche fragile:** solo `mtime == timemodified` e la dimensione. Se il file
   viene toccato in locale viene riscaricato e **sovrascritto**, perdendo le annotazioni
   sulle slide. **Fix:** stato in SQLite più sync a tre vie (§7).
7. **Nomi file:** dedup con `(1)`, `(2)` dipendente dall'ordine. Va resa stabile, legata all'id del modulo.
8. **Cartelle compresse: ✅ caso reale trovato (Data Mining, 2026-11-09).** Sono ZIP caricati dal
   docente **senza estensione** (`filename = "06 Classification"`, `mimetype = "application/zip"`,
   `fileurl` che termina con `?forcedownload=1`). La v1 li scarica ma li salva senza estensione,
   quindi in Windows non sembrano archivi. **Fix:** se `filename` non ha estensione, aggiungerla
   in base al `mimetype` (funzione pura in `paths.rs`, testata). Opzione futura: estrazione
   automatica degli zip scaricati (disattivabile).

---

## 6. Corsi attivi e semestri

### 6.1 Modello degli stati di un corso

| Stato | Significato | Comportamento predefinito |
|-------|-------------|---------------------------|
| `current` | Semestre in corso | Sync attivo, avvisi attivi |
| `exam_pending` | Semestre concluso nello stesso anno accademico, esame non ancora segnato come superato | Sync attivo, consigliato |
| `passed` | Segnato come superato dall'utente | Sync disattivo. Pulizia dei file **proposta** all'avvio (§6.5), mai automatica senza conferma |
| `past` | Anni accademici precedenti | Nascosto, visibile con un filtro |
| `ignored` | Escluso manualmente | Nascosto |

L'utente può sempre sovrascrivere stato e flag di sync a mano. Le scelte manuali vincono sulle euristiche.

### 6.1-bis Dati reali di un corso (risposta della dashboard, verificata il 2026-11-09)

Oggetto corso restituito da `core_course_get_enrolled_courses_by_timeline_classification`
(campo `courseimage` omesso):
```json
{
  "id": 25817,
  "fullname": "051589 - MODEL IDENTIFICATION AND DATA ANALYSIS - 2ND MODULE (DOCENTE A) {051587 - MODEL IDENTIFICATION AND DATA ANALYSIS [sezione A]} [2026-27]",
  "shortname": "051589 - MODEL IDENTIFICATION... 889103",
  "idnumber": "889103-STD",
  "startdate": 1789385061,
  "enddate": 0,
  "visible": true, "hidden": false, "isfavourite": false,
  "progress": 0, "hasprogress": false,
  "viewurl": "https://webeep.polimi.it/course/view.php?id=25817",
  "coursecategory": "2026-27"
}
```

Cosa se ne ricava:
- **Anno accademico = `coursecategory`** (formato `YYYY-YY`), ripetuto anche in coda a `fullname` (`[2026-27]`).
  Per avere i "corsi dell'anno accademico corrente" basta confrontare `coursecategory`
  con l'anno corrente (da settembre a agosto: a novembre 2026 è `2026-27`). È robusto e
  indipendente dal filtro della dashboard. Nessun `customfields` nella risposta.
- **Formato di `fullname`** (da parsare con una regex testata su molti esempi reali):
  `<codice> - <NOME>[ - <N>ND/ST MODULE] (<DOCENTE/I>) {<codice padre> - <NOME PADRE> [sezione X]} [<AA>]`
  - `codice`: codice del corso o modulo (`051589`)
  - `DOCENTE`: tra parentesi tonde. **Dati reali:** il docente è l'**ultimo** gruppo tra parentesi
    (esiste `052511 - SISTEMI INFORMATIVI (PER IL SETTORE DELL'INFORMAZIONE) (DOCENTE B)`);
    più docenti sono separati da virgola; a volte manca lo spazio prima dell'anno (`(…)[2024-25]`);
    il corso del corso di studi (`Ingegneria Informatica`, categoria `CCS`) non segue il formato
  - `{…}` facoltativo: **insegnamento padre** (`051587`), utile per i corsi integrati a moduli, e
    **sezione o scaglione** (`sezione A`). La chiave di raggruppamento del §6.3 deve essere il
    **codice padre** se presente, altrimenti il codice del corso.
  - `[AA]`: anno accademico
- `enddate = 0`: **non impostata**, quindi inutilizzabile.
- `startdate` = 2026-09-14, coerente con l'inizio del 1° semestre. **Ma** è un caso raro: la maggior
  parte dei corsi ha la data predefinita di fine luglio (vedi §6.2 e la tabella Verifiche).
- `idnumber` (`889103-STD`): probabilmente l'id dell'offerta didattica. Il significato è ancora da capire.

### 6.2 Rilevamento del semestre (euristica a cascata, da validare sui dati reali)

> **Aggiornamento dal dump del 2026-11-09:** nessun custom field (verificato con
> `core_course_get_courses_by_field`), `enddate = 0` nell'AA corrente e **`startdate` quasi sempre
> la data predefinita di fine luglio** (14 corsi su 16), quindi inutile come segnale nella maggior
> parte dei casi. Usarla solo quando cade fuori da luglio–agosto. Segnali alternativi da valutare:
> (a) **attività recente** del corso (`timemodified` dei file, date degli avvisi) rispetto al
> calendario; (b) il **Manifesto degli studi** pubblico del Polimi, che indica il semestre per
> codice insegnamento (scraping di un sito esterno); (c) override manuale, sempre disponibile.

1. **Custom field** del corso (anno accademico, semestre, periodo didattico) da
   `core_course_get_courses_by_field`. È la fonte migliore, se esiste.
2. **Nome del corso**: la v1 usa la regex `\d+ - (.+) \(.+\)` per estrarre il nome breve, quindi
   il `fullname` contiene codice insegnamento e docente. Verificare se contiene anche anno o semestre.
3. **`startdate` / `enddate`**, con il calendario Polimi indicativo:
   - 1° semestre: lezioni da metà settembre a dicembre, esami gennaio–febbraio
   - 2° semestre: lezioni da fine febbraio/marzo a giugno, esami giugno–luglio
   - Appello di settembre per entrambi
   Inizio tra agosto e ottobre: 1° semestre. Inizio tra febbraio e marzo: 2° semestre.
   Corsi annuali: date che coprono entrambi.
4. **Override manuale** nella UI.

### 6.3 Raggruppamento per insegnamento (iscrizioni spontanee)

Raggruppare i corsi per **codice insegnamento** (es. `056892`) ricavato dal nome: lo stesso
insegnamento con docenti diversi diventa un unico gruppo ("Data Mining: 2 corsi, il tuo
scaglione più il prof. X"). Segnare "superato" vale per tutto il gruppo, così il corso
spontaneo non resta in lista quando quello d'ufficio viene archiviato.

### 6.4 Consigli in UI

Vista corsi divisa in: **Questo semestre**, **Esami in sospeso**, **Altri / passati**.
Badge "consigliato". Al primo avvio un onboarding propone la selezione in base a semestre e
filtro "anno accademico corrente", e l'utente conferma.

### 6.5 Pulizia dei file dei corsi superati (richiesta dall'utente)

Quando un corso (o un gruppo di insegnamento, §6.3) passa a `passed`, l'app propone di eliminarne i file.

**Flusso:**
1. Il passaggio a `passed` crea una **pulizia in sospeso** in SQLite (id corso, data). Non elimina nulla subito.
2. **All'avvio successivo** (e anche subito, se l'utente segna "superato" dalla UI con l'app aperta)
   compare una finestra di dialogo, una sola per tutte le pulizie in sospeso:
   > "Hai superato **Data Mining**. Vuoi eliminare i 142 file scaricati (1,3 GB)?  [Sì] [No] [Chiedimelo più tardi]"
   - **Sì**: esegue la pulizia (vedi regole di sicurezza).
   - **No**: annulla la pulizia per quel corso. I file restano e la domanda non torna.
   - **Più tardi**: la richiesta resta in sospeso e ricompare al prossimo avvio.
3. Impostazione globale "Quando supero un corso": **Chiedi** (default) / **Elimina sempre** / **Non eliminare mai**.
   Anche con "Elimina sempre" valgono tutte le regole qui sotto.

**Rilevamento automatico (suggerimento, non decisione):** se un corso d'ufficio sparisce dalle
iscrizioni (archiviato dopo l'esame, vedi §1) mentre resta il corso spontaneo dello stesso
codice, proponi all'avvio: "Sembra che tu abbia superato Data Mining. Segnarlo come superato?".
La pulizia parte solo dopo questa conferma, poi segue il flusso sopra.

**Regole di sicurezza (obbligatorie):**
- Elimina **solo i file tracciati nella baseline SQLite**, cioè scaricati dall'app. File aggiunti
  a mano dall'utente nella cartella del corso **non vengono mai toccati**.
- I file **modificati in locale** (hash o mtime diversi dalla baseline, es. slide annotate) sono
  **esclusi di default** e vanno elencati a parte: "3 file che hai modificato: elimina anche questi?".
- Spostare nel **Cestino** (crate `trash`), non eliminare in modo permanente. Il messaggio di
  conferma lo dice esplicitamente ("puoi recuperarli dal Cestino").
- Dopo la pulizia rimuovere solo le cartelle rimaste **vuote**. Se restano file dell'utente,
  la cartella del corso resta.
- Mostrare prima numero di file e spazio liberato. A fine operazione mostrare un riepilogo
  (eliminati, saltati, errori). Registrare l'operazione nel log.
- Se un file è aperto o bloccato (Windows), saltarlo e segnalarlo, senza interrompere il resto.
- Se l'utente riporta il corso da `passed` ad attivo, il sync successivo riscarica i file mancanti normalmente.
- La logica che decide **cosa** eliminare è una funzione pura in `sync/cleanup.rs`
  (baseline + stato locale → lista di azioni), testata con TDD. L'I/O è separato.

---

## 7. Motore di sincronizzazione

- **Stato locale in SQLite** (`rusqlite` con feature `bundled`). Per ogni file remoto: id corso,
  id modulo, id/percorso remoto stabile, `fileurl`, `timemodified`, `filesize`, percorso locale,
  hash/mtime/dimensione **al momento della scrittura** (baseline).
- **Sync a tre vie** (idea presa da BeepBar), confronto tra baseline, file locale e file remoto:
  - cambiato solo il remoto: scarica e sostituisci
  - cambiato solo il locale (slide annotate): **non toccare** e **non** aggiornare la baseline
    *(corretto il 2026-11-09: aggiornandola, un successivo aggiornamento del docente sembrerebbe
    "cambiato solo il remoto" e sovrascriverebbe le annotazioni. Con la baseline originale diventa
    un conflitto e il remoto viene salvato a parte)*
  - cambiati entrambi: salva il remoto come `nome (WeBeep aggiornato YYYY-MM-DD).ext` e
    segnala un conflitto nel riepilogo. **Mai sovrascrivere in silenzio.**
- **Download atomico:** scrivi in `*.part`, poi rinomina. Imposta `mtime = timemodified`.
- **File rimosso dal docente:** non cancellare in locale, segnalalo. L'unico caso in cui l'app
  elimina file è la pulizia dei corsi superati (§6.5), sempre confermata dall'utente.
- **Concorrenza limitata** (semaforo `tokio`, default 4, configurabile, massimo 8) per non stressare WeBeep.
- **Isolamento per corso:** il fallimento di un corso non ferma gli altri.
- **Struttura cartelle:** `<radice>/<NomeCorso>/<Sezione>/<Sottosezione>/<Modulo>/<sottopercorso>/file`.
  Opzione (attiva di default, come la v1) per appiattire la sezione "Materiali".
- **Sanitizzazione percorsi Windows:** caratteri vietati `<>:"/\|?*`, nomi riservati (`CON`, `NUL`,
  `COM1`…), punti e spazi finali, limite di 260 caratteri (usa prefisso `\\?\` oppure accorcia).
  Gestire i nomi che differiscono solo per maiuscole/minuscole.
- **Migrazione dalla v1:** cartella della v1 dell'utente (vedi `CLAUDE.local.md`). Al primo
  avvio offri di "adottare" una cartella esistente: i file con stessa dimensione e percorso
  compatibile entrano nella baseline senza essere riscaricati.
- **Riepilogo dopo ogni sync:** nuovi, aggiornati, conflitti, contenuti esterni non scaricabili, errori per corso.
- **Autosync** a intervallo configurabile (default 2 h, minimo 30 min), solo per corsi attivi.

---

## 8. Avvisi, scadenze e notifiche

- **Avvisi:** per ogni corso attivo trova il forum `news` e interroga
  `mod_forum_get_forum_discussions` periodicamente (default 15 min). Dedup tramite id discussione
  in SQLite. Notifica di sistema con `tauri-plugin-notification`; il clic apre l'avviso in app.
  Pagina "Bacheca" con elenco filtrabile per corso, letto/non letto e allegati scaricabili.
- **Scadenze:** `core_calendar_get_action_events_by_timesort` per i prossimi 14 giorni, con
  promemoria configurabile (es. 24 h prima).
- **Notifiche generiche:** `message_popup_get_popup_notifications` come fonte secondaria.
- Silenzia per corso; nessuna notifica per i corsi `passed`/`ignored`.

---

## 9. Condivisione con i compagni: cosa serve oltre al codice

1. **Installer:** Tauri genera NSIS (`.exe`) e MSI. Configurare `webviewInstallMode` (bootstrapper
   WebView2) per i PC Windows 10 vecchi.
2. **SmartScreen:** un `.exe` non firmato mostra "Windows ha protetto il PC" (si apre con
   "Ulteriori informazioni → Esegui comunque"). Opzioni: accettarlo e spiegarlo nel README;
   **SignPath Foundation** (firma gratuita per progetti open source); Azure Trusted Signing (a pagamento).
3. **macOS:** build via GitHub Actions (`tauri-apps/tauri-action`, target universale Apple
   Silicon/Intel). Senza Apple Developer ID (99 €/anno) l'app va aperta con tasto destro, Apri,
   oppure `xattr -dr com.apple.quarantine`. Documentarlo. Linux: AppImage e `.deb`.
4. **Aggiornamenti automatici:** `tauri-plugin-updater` più GitHub Releases (`latest.json`). Richiede
   una **coppia di chiavi di firma** (`tauri signer generate`). La chiave privata va **solo** nei
   secrets di GitHub, mai nel repo. Senza auto-update i compagni restano su versioni buggate.
5. **CI/CD:** GitHub Actions con fmt, clippy, test su ogni push e build e release sui tag `v*`.
6. **Privacy (da scrivere nel README):** nessun server, nessuna telemetria. Ogni studente usa il
   proprio login; token salvato solo sul suo PC (keyring) e inviato solo a `webeep.polimi.it`.
7. **Disclaimer:** "Progetto non ufficiale, non affiliato al Politecnico di Milano". Non usare il
   logo Polimi. Valutare un nome che non sembri ufficiale.
8. **Uso rispettoso delle API:** concorrenza bassa, intervalli minimi, backoff, User-Agent
   identificativo (`WeBeepSyncV2/<versione>`) senza dati personali.
9. **Supporto:** pulsante "Esporta diagnostica" che produce zip con log e dump JSON della
   struttura di un corso, **con token e dati personali rimossi**. Template di issue su GitHub che
   lo richiede.
10. **Istanza singola:** `tauri-plugin-single-instance`. *(Avvio con il sistema e tray scartati
    dall'utente il 2026-11-09: l'app non lavora in background.)*
11. **README in italiano** con installazione, primo avvio, FAQ (SmartScreen, Gatekeeper,
    "dove sono i file", "come segno un corso come superato").

---

## 10. Architettura proposta

```
webeep-syncV2/
├─ CLAUDE.md                ← questo file
├─ _riferimenti/            ← codice v1 solo da leggere (GPL-3.0). Mettere in .gitignore
├─ src/                     ← frontend Svelte + TS
│  ├─ lib/api.ts            ← wrapper dei comandi Tauri (invoke) tipizzati
│  ├─ routes|views/         ← Corsi, Sync, Bacheca, Scadenze, Impostazioni, Onboarding
│  └─ i18n/
└─ src-tauri/
   ├─ Cargo.toml
   ├─ tauri.conf.json
   ├─ capabilities/         ← permessi minimi Tauri 2
   └─ src/
      ├─ main.rs / lib.rs   ← setup app, plugin, tray
      ├─ commands/          ← comandi Tauri sottili: validano l'input e delegano ai servizi
      ├─ auth/              ← finestra di login, parsing e verifica token, keyring
      ├─ moodle/            ← client HTTP (reqwest), tipi serde delle risposte, errori
      ├─ courses/           ← classificazione, semestri, raggruppamento per codice
      ├─ sync/
      │  ├─ tree.rs         ← risposta get_contents → albero di file remoti (funzione PURA, testata)
      │  ├─ extract.rs      ← link pluginfile da HTML di label/page/url
      │  ├─ plan.rs         ← baseline + locale + remoto → azioni (funzione PURA, testata)
      │  ├─ cleanup.rs      ← pulizia dei corsi superati: cosa eliminare (funzione PURA) + spostamento nel Cestino
      │  ├─ download.rs     ← download atomici, concorrenza, retry
      │  └─ paths.rs        ← sanitizzazione percorsi Windows
      ├─ notify/            ← avvisi, scadenze, notifiche di sistema
      ├─ store/             ← SQLite (migrazioni), impostazioni
      └─ diagnostics/       ← export redatto
```

Crate consigliati: `tauri` 2, `reqwest` (rustls, json, stream), `tokio`, `serde`/`serde_json`,
`rusqlite` (bundled), `keyring`, `thiserror` (errori di libreria), `anyhow` (solo ai bordi),
`tracing` e `tracing-appender` (log con redazione del token), `scraper` o `regex` (link HTML),
`md-5`, `base64`, `url`, `chrono`. Per i test: `wiremock` (mock HTTP), `tempfile`, `insta` (snapshot dell'albero).

**Principio chiave:** la logica che decide *cosa* scaricare e *dove* (`tree.rs`, `plan.rs`,
`courses/`) è fatta di **funzioni pure senza I/O**, testate con fixture JSON reali. È la parte
che nella v1 sbagliava.

---

## 11. Piano di lavoro

### Fase 0: setup e verifica sul sito reale (prima di tutto il resto)
- [x] Installare i prerequisiti (§3) e creare lo scaffold Tauri 2 + Svelte. `git init`, `.gitignore`
      (`_riferimenti/`, `target/`, `node_modules/`, `fixtures/raw/`). *(2026-11-09: template
      ufficiale SvelteKit SPA + adapter-static; LICENSE GPL-3.0, README)*
- [x] Implementare il **login** (§4.2) e un comando "dump" che salva in `fixtures/raw/` le risposte
      *(2026-11-09: `auth/`, `moodle/client.rs`, `diagnostics/dump.rs`, comando `dump_phase0`.
      Login e primo dump riusciti sul sito reale. Il dump ora salva `get_contents` di tutti i
      corsi `inprogress`, più consegne e forum)*. Risposte
      JSON di: `core_webservice_get_site_info`, `core_enrol_get_users_courses`,
      `core_course_get_enrolled_courses_by_timeline_classification` (classificazioni `inprogress`,
      `past`, `all`), `core_course_get_courses_by_field` e `core_course_get_contents` del corso
      **056892 Data Mining**.
- [x] Passo manuale dell'utente: payload del filtro "Anno accademico corrente" (§4.5).
- [x] Confrontare l'albero di Data Mining su WeBeep con quello che produrrebbe la logica v1 e
      **confermare o smentire le ipotesi del §5** *(albero corretto; trovati gli ZIP senza estensione)*.
- [x] Ripetere il dump esteso e cercare il problema 2 negli altri corsi `inprogress`
      *(causa: moduli `page` scartati dalla v1; vedi Verifiche)*.
- [x] Creare fixture **redatte** (senza nomi, email, token) in `fixtures/` per i test
      *(2026-11-09: `scripts/redact-fixtures.mjs` con controllo anti-fuga; 7 corsi, 3 timeline,
      consegne; vedi `fixtures/README.md`)*.
- [x] Aggiornare la sezione "Verifiche" qui sotto *(2026-11-09)*.

### Fase 1: MVP sync
- [x] Client Moodle con gestione errori, retry e redazione dei log
      *(errori HTTP 200 con `exception`, `invalidtoken`, niente URL negli errori di rete; il retry con
      backoff è nei download, le chiamate REST falliscono subito e il corso viene segnalato)*
- [x] Lista corsi con filtro (attivi, anno corrente, tutti) e selezione manuale
      *(2026-11-09: `courses/name.rs` analizza il `fullname` (codice, nome, docenti = ultimo gruppo tra
      parentesi, padre, sezione, AA), testato su tutti i 33 nomi reali; `courses/mod.rs`: timeline
      `all`, selezione di default = `coursecategory` uguale all'AA corrente (da settembre), le scelte
      manuali prevalgono. Cartella del corso = nome breve (come la v1, es. `DATA MINING`), assegnata
      una volta e salvata in SQLite: non cambia mai. In caso di omonimi si aggiunge il docente (stesso
      AA) o l'AA (anni diversi, il corso corrente tiene il nome semplice). Nota: l'utente si iscrive
      di nuovo agli stessi corsi in AA diversi, quindi il filtro per AA è indispensabile)*
- [x] `tree.rs` più `extract.rs` (cartelle, sottosezioni, label, page, url interni, assign), con TDD su fixture
      *(2026-11-09: `paths.rs` (sanitizzazione Windows, estensione da mimetype); `tree.rs`
      (folder annidati, `page` senza `index.html`, `resource` col nome del modulo, sottosezioni,
      tipi sconosciuti inclusi, dedup stabile per id modulo e case-insensitive, i file elencati da
      Moodle vincono sui file collegati); `extract.rs` (solo link `<a href>` a `pluginfile.php`,
      convertiti in `/webservice/pluginfile.php`; le `<img>` delle etichette sono decorative e
      ignorate, verificato sui dati reali); `url` interni scaricati, esterni elencati;
      `assign.rs` (allegati delle consegne). I file collegati hanno dimensione e data sconosciute (0):
      `plan.rs` dovrà scaricarli se mancano senza poterne rilevare gli aggiornamenti)*
- [x] `plan.rs` (sync a tre vie) con TDD e download atomico
      *(2026-11-09: `plan.rs` fatto, 17 test: nuovo, adozione di file già presenti (v1) con stessa
      dimensione, solo remoto, solo locale, entrambi → copia "WeBeep aggiornato", file locale
      cancellato → riscaricato, rimosso dal docente → solo segnalato, cambio di percorso → rinomina
      se intatto. File locale non tracciato e diverso → conflitto, mai sovrascritto.
      `download.rs`: `.part` poi rinomina, `mtime = timemodified`, `token=` aggiunto solo a URL
      dello stesso sito (mai inviato altrove, mai negli errori), retry con backoff su rete/5xx/429,
      niente redirect. `execute.rs`: applica le azioni di un corso con un semaforo condiviso, errori
      raccolti per file, rinomina mai sopra un file esistente; dopo un conflitto la baseline registra
      la copia, così l'originale annotato resta "cambiato solo in locale" e non genera una copia a
      ogni sync. `local.rs`: `safe_join` rifiuta `..`, assoluti e unità. `store/`: SQLite con
      migrazioni `user_version` (baseline, selezione manuale dei corsi, impostazioni).
      Dopo la review di sicurezza: subito prima della rinomina si ricontrolla il file di destinazione
      (deve essere assente o identico alla baseline, altrimenti non viene toccato); il numero di byte
      ricevuti deve coincidere con `filesize` (una pagina di errore servita con 200 non sostituisce un
      file buono); le copie di conflitto prendono un nome libero (` (2)`…); il file temporaneo si
      chiama `.webeep-part`; il token va solo a `/webservice/pluginfile.php/`; l'adozione richiede
      dimensione **e** `mtime = timemodified` (verificato: 76 file v1 su 76 coincidono). Limiti
      accettati: junction/symlink dentro la cartella non controllati, nessun blocco su `C:\` come radice)*
- [x] Riepilogo della sincronizzazione e scelta della cartella di destinazione, con adozione della cartella v1
      *(✅ provato sul sito reale il 2026-11-09: primo sync in una cartella nuova = 436 file, 0 errori,
      7 cartelle (i 9 corsi del 2° semestre non hanno ancora materiale); il secondo sync non scarica
      nulla. Login confermato dopo l'irrigidimento della finestra. Aggiunti poi: "Cambia" cartella
      del corso (`sync/folders.rs`: rinomina su disco mantenendo la baseline, oppure punta a una
      cartella esistente azzerando la baseline, così la v1 viene adottata) e impostazioni
      (`store/settings.rs`: download contemporanei 1–8, appiattimento di "Materiali").
      Comandi divisi in `commands/{courses,settings,sync,shared}.rs`)*
      *Note storiche:*
      *(2026-11-09: `sync/run.rs` orchestra il sync (corsi in sequenza, download in parallelo con un
      semaforo da 4, corso isolato, token non valido → stop), comandi `list_courses`,
      `set_course_selected`, `sync_root`, `choose_sync_root` (selettore cartelle nativo lato Rust),
      `sync_now` (evento `sync-progress`, un solo sync alla volta). UI: cartella, lista corsi con
      "mostra anche gli anni passati", pulsante di sync, riepilogo per corso. Adozione v1: si sceglie
      `Desktop\polimi` come cartella e i file con stessa dimensione vengono adottati. **Provato**
      (vedi sopra). Testo originale: **Da provare
      sul sito reale.** Limite noto: le cartelle v1 con nome completo (es. `052537 - TECHNOLOGIES…
      [2026-27]`) non coincidono con il nome breve e quei corsi verrebbero riscaricati in una
      cartella nuova: servirà un "cambia cartella del corso")*

### Fase 2: corsi intelligenti
- [x] Rilevamento del semestre, stati dei corsi, raggruppamento per codice, onboarding con consigli
      *(2026-11-09: `courses/semester.rs`, euristica a cascata verificata sui dati reali: scelta
      manuale > **date dei file** caricati nell'AA (15 ago–14 feb = 1°, 15 feb–14 ago = 2°; i file
      riusati da anni precedenti non contano; i 5 corsi reali del 1° semestre con file vengono
      riconosciuti) > `startdate` solo se fuori da luglio–agosto > sconosciuto. "Nessun materiale"
      **non** è usato: Machine Learning è del 1° semestre ma non ha file datati. L'attività viene
      salvata a ogni sync (`course_activity`). `courses/state.rs`: stati `current`, `upcoming`
      (nuovo: corso del 2° semestre visto nel 1°, sincronizzato), `examPending`, `passed`, `past`,
      `ignored`; periodo corrente: 1° dal 1° settembre al 14 febbraio. **Appello di settembre**
      (aggiunto il 2026-11-13): a settembre i corsi dell'AA appena finito non segnati come superati
      restano `examPending` (sincronizzati, avvisi controllati) e diventano `past` solo dal 1°
      ottobre; i corsi del nuovo AA sono attivi dal 1° settembre (`course_year` in
      `courses/mod.rs`, `CourseYear::LastSession`). Le scelte "superato",
      "ignorato" e il semestre valgono per l'intero insegnamento (`teaching_key` = codice padre o
      codice del corso), in tutti gli AA. UI a sezioni (Questo semestre, Prossimo semestre, Esami in
      sospeso, Superati e ignorati, Anni precedenti), guida al primo avvio, data dell'ultimo sync)*
- [x] Pulizia dei corsi superati (§6.5): richiesta all'avvio Sì/No/Più tardi, Cestino, esclusione dei file modificati, rilevamento dei corsi archiviati
      *(2026-11-09: `sync/cleanup.rs` (piano puro: scaricati intatti / modificati / spariti;
      esecuzione con Cestino iniettabile, ricontrollo prima di spostare, rimozione delle sole cartelle
      vuote), `sync/retire.rs` (impostazione Chiedi / Elimina sempre / Non eliminare mai; con "sempre"
      i file modificati restano comunque), `courses/archived.rs` (corso dell'AA corrente sparito
      dalle iscrizioni mentre resta un altro corso dello stesso insegnamento → suggerimento "Sì /
      No", mai automatico). Migrazione SQLite 2: `teaching`, `course_activity`, `pending_cleanup`,
      `known_course`)*

### Fase 3: notifiche
- [x] Bacheca avvisi, scadenze, notifiche di sistema, tray, autostart, autosync
      *(2026-11-09, provato dall'utente in `tauri dev`: "sembra funzionare tutto". Forma delle
      risposte verificata sul dump (forum
      `news` = "Annunci"; discussioni con `message` HTML (`p, a, div, strong, span, br, pre`),
      `attachments` presente solo se ci sono allegati; scadenze da
      `core_calendar_get_action_events_by_timesort`: quiz `close` e assign `due`). Fixture redatte
      con `scripts/redact-phase3.mjs` in `fixtures/notify/`. `notify/html.rs`: HTML ripulito con
      `ammonia` (allow-list, niente script/eventi/img/`javascript:`) e anteprima in testo per le
      notifiche; i link si aprono nel browser. `notify/announcements.rs`: le discussioni di un
      forum si scaricano solo se `numdiscussions` è cambiato (di solito 1 chiamata per controllo);
      la prima importazione non notifica (i post degli ultimi 7 giorni restano non letti).
      `notify/deadlines.rs`: 14 giorni, promemoria una volta sola entro 24 ore. `background.rs`:
      ogni minuto valuta avvisi (15 min), scadenze (60 min) e sync automatico (default ogni 2 ore,
      min 30 min, solo dopo il primo sync manuale); notifiche di sistema disattivabili; corsi
      silenziabili. UI a schede: Corsi, Bacheca, Scadenze, Impostazioni.
      **Decisione dell'utente (2026-11-09): niente background.** Chiudere la finestra chiude l'app;
      niente icona nell'area di notifica, niente avvio con Windows (tolti `tray.rs` e
      `tauri-plugin-autostart`). I controlli periodici girano solo a finestra aperta; all'apertura
      gli avvisi si controllano dopo 5 s e il sync automatico parte se l'ultimo è più vecchio
      dell'intervallo. Resta l'istanza singola (un secondo avvio mostra la finestra). Limiti noti: gli
      allegati degli avvisi si aprono dalla pagina WeBeep (serve il login nel browser); il clic
      sulla notifica non apre l'avviso specifico)*

### Fase 4: distribuzione
- [ ] CI, release, updater firmato, build macOS e Linux, README, export diagnostica, i18n EN

### Fase 5: app vera e propria per PC (richiesta dell'utente, 2026-11-09, da fare alla fine)
Passare dal progetto avviato con `npm run tauri dev` a un'app installata e usata ogni giorno:
- [x] Build di release e installer Windows (NSIS): voce nel menu Start, icona propria (non quella
      di Tauri), disinstallazione pulita che **non** cancella i file scaricati
      *(2026-11-09: `npm run tauri build` → `src-tauri/target/release/bundle/nsis/WeBeep
      Sync_0.1.0_x64-setup.exe`, 4,3 MB. Installazione per utente (senza admin), in italiano, con
      licenza GPL; WebView2 scaricato se manca. Icona propria da `icons/app-icon.svg` (frecce di
      sync attorno a una freccia di download, niente logo Polimi; rigenerare con
      `npm run tauri icon src-tauri/icons/app-icon.svg` e cancellare le cartelle android/ios).
      I file sincronizzati non sono mai nella cartella di installazione. Log su file giornaliero
      (`logging.rs`, `%LOCALAPPDATA%\it.webeepsync.app\logs`, 7 giorni). README con installazione,
      SmartScreen e FAQ. **Da provare: installazione e uso dell'app installata**)*
- ~~Avvio con Windows, icona nella tray, chiusura = riduzione nella tray, sync in background~~
      **Scartato dall'utente (2026-11-09):** l'app non deve restare attiva quando la finestra è
      chiusa. Sync e controlli solo a finestra aperta (vedi Fase 3)
- [x] Rimuovere dalla build di release gli strumenti di sviluppo (dump della Fase 0)
      *(UI solo con `import.meta.env.DEV`; il comando rifiuta già nelle build di release)*
- [x] Pagine separate (Corsi, Sincronizzazione, Bacheca, Impostazioni) al posto della pagina unica
      *(schede Corsi, Bacheca, Scadenze, Impostazioni)*
- [ ] Aggiornamenti automatici (vedi Fase 4) e prova completa su un PC "pulito"

### Idee future (non prioritarie)
- Estrazione automatica degli zip; elenco delle registrazioni delle lezioni (vedi il repo
  `aleo113/webeep-sync-recordings` e BeepBar); esportazione delle scadenze in `.ics`.

### Verifiche (da compilare durante la Fase 0)
| Ipotesi | Esito | Note |
|---------|-------|------|
| Versione Moodle di WeBeep (`release`) | **4.5.14** | `release = "4.5.14 (Build: 20260914)"`, `version = 2024100714` (dump del 2026-11-09). Le sottosezioni (`mod_subsection`) sono quindi disponibili sul sito |
| Il filtro "Anno accademico corrente" usa `classification=customfield` | **no** | È `classification=inprogress` (vedi riga "Payload esatto" sotto) |
| Versione Moodle recente (≥ 4.5?) | **sì** | 4.5.14 |
| Esiste un custom field per il semestre | **no** | `core_course_get_courses_by_field` su 33 corsi: nessun corso ha `customfields` |
| `startdate` indica il semestre | **quasi mai** | 14 corsi su 16 del 2026-27 hanno `startdate` = 28/29 luglio (data predefinita di creazione). Solo alcuni docenti la impostano davvero (2026-09-14; un corso del 2° semestre 2025-26 ha 2026-02-20). Il semestre va quindi ricavato da altro: override manuale, attività recente del corso, Manifesto degli studi (§6.2) |
| Corsi nascosti dall'utente | **esclusi da `all`** | `core_enrol_get_users_courses` restituisce 46 iscrizioni, la timeline `all` 33: le 13 mancanti hanno `hidden=true` (nascoste dall'utente nella dashboard, tutte di AA passati). Stanno nella classificazione `hidden` |
| Il formato di `fullname` contiene il codice insegnamento | **sì** | Contiene codice, docente, codice padre, sezione e anno accademico. Vedi §6.1-bis |
| L'anno accademico si ricava da `coursecategory` | **sì** | `"2026-27"` |
| `enddate` utilizzabile | **solo a posteriori** | Vale 0 per i corsi dell'AA corrente; nei corsi degli AA passati è valorizzata con la fine dell'AA (es. 2026-08-30, una volta 2025-07-31). Indica la fine dell'anno accademico, non del semestre |
| Payload esatto del filtro "Anno accademico corrente" | **sì** | È il filtro standard **"In corso" rinominato**. Preferenza salvata: `{"value":"inprogress"}`. Chiamata: `{"offset":0,"limit":0,"classification":"inprogress","sort":"fullname","customfieldname":"","customfieldvalue":"","requiredfields":["id","fullname","shortname","showcoursecategory","showshortname","visible","enddate"]}`. Nessun custom field |
| Con `enddate = 0` i vecchi corsi restano `inprogress` per sempre? | **no, con un'eccezione** | I corsi degli AA passati ricevono una `enddate` e passano in `past` (16 corsi: 2023-24, 2024-25, 2025-26). `inprogress` = 16 corsi del 2026-27 **più** "Ingegneria Informatica" (`coursecategory = "CCS"`, comunità del corso di studi, dal 2021, `enddate = 0`), che resta in corso per sempre. Il filtro `coursecategory == AA corrente` lo esclude: va tenuto. I corsi CCS potrebbero servire per gli avvisi (Fase 3) |
| Data Mining usa `subsection` o `folder` annidati | **folder annidati** | Sezioni: Introduzione, Bacheca (2 forum), Materiali (3 `folder`: Syllabus 36 file, Exams 44 file, Extras vuoto), Registrazioni (1 `url` esterno verso `aunicalogin.polimi.it`). `filepath` fino a 2 livelli (`/Notebooks/Optional Notebooks/`). Nessuna `subsection`, `label`, `page` o `resource` |
| La v1 perde file di Data Mining | **no (albero corretto)** | Confronto con la cartella `DATA MINING` della v1: 79 file locali contro 80 remoti, le differenze sono solo file rinominati o aggiunti dal docente dopo l'ultimo sync della v1 (ipotesi §5.4 corretta). **Ma** 8 notebook sono ZIP caricati **senza estensione** (es. `06 Classification`, `mimetype = application/zip`, header `PK`): la v1 li salva senza estensione e Windows non li apre. Sono le "cartelle compresse" del problema 2 (§5.8) |
| Il login aprendo direttamente `launch.php` funziona | **sì** | Flusso osservato: `launch.php` → SSO (aunicalogin, eventuale SPID, shibidp) → `/Shibboleth.sso/SAML2/POST` → `/auth/shibboleth/index.php` → `launch.php` → `moodlemobile://token=...`. Il ripiego su `/my/` non è servito. WebView2 segnala la navigazione verso lo schema personalizzato a `on_navigation` |
| La firma del token è `md5(httpswwwroot + passport)` | **sì** | Accettata con `https://webeep.polimi.it` |
| Problema 2 ("solo i file singoli") su altri corsi | **✅ causa trovata: moduli `page`** | Dump esteso (17 corsi `inprogress`, 2026-11-09). La v1 scarta tutti i `page`, ma alcuni docenti ci caricano il materiale: **Data and Information Quality** ha **tutto** il materiale (21 file, 122 MB: slide e notebook) nella pagina "Course Material"; **Formal Languages and Compilers** ha 5 ZIP nella pagina "Tutoring Sessions". I file stanno in `contents` del modulo, accanto a `index.html` (da ignorare). Nessun corso usa `subsection` (§5.1 escluso per ora, ma il codice deve gestirle: Moodle 4.5). Le `label` contengono solo immagini decorative (`mod_label/intro/*.jpg`); nessun riepilogo di sezione ha link `pluginfile` |
| Moduli `url` | **tutti esterni** | 32 URL, nessuno verso WeBeep: `aunicalogin.polimi.it` (16, registrazioni), `kaggle.com` (8), `polimi365-my.sharepoint.com` (3, OneDrive), `webex.com` (2), `remaplab.deib.polimi.it` (2), `github.com` (1). Vanno elencati come "contenuti esterni" |
| Corsi senza materiale | **9 su 16** | Contengono solo i 2 forum e l'URL delle registrazioni: probabilmente corsi del 2° semestre non ancora iniziati. Possibile segnale per il semestre (§6.2) |
| Cosa cambia con i contenuti in inglese (`moodlewssettinglang=en`) | **i nomi del modello Polimi e i default di Moodle** | Dump `…-it`/`…-en` del 2026-11-13, 16 corsi, 80 sezioni. **Sezioni:** Bacheca → Notice board, Materiali → Materials, Registrazioni → Recordings, Consegne → Assignments (testi multilingua del modello dei corsi Polimi), Introduzione → General e Nuova sezione → New section (nomi predefiniti di Moodle). **Moduli:** Annunci → Announcements (forum), Archivio registrazioni → Recordings archive (url), più pochi nomi scritti dai docenti in due lingue (un url, una `page`, un **`folder` Documenti → Documents**). **File:** nessun file vero cambia nome (cambiano solo i `filename` dei moduli `url`). **Corsi:** solo il corso del corso di studi (Ingegneria Informatica → Computer Science and Engineering). Cambiano anche le etichette delle scadenze (`action.name`, `formattedtime`…), che l'app non usa. **Conseguenza:** con i contenuti in inglese «Materiali» non verrebbe più appiattita e cartelle come Consegne, Introduzione e Documenti cambierebbero nome, spostando i file. Per questo l'app chiede a WeBeep **sempre l'italiano** (`ContentLanguage::Italian`, predefinito in `moodle/client.rs`), qualunque sia la lingua dell'interfaccia |

---

## 12. Regole di sviluppo

- **TDD** sulla logica pura (`tree`, `plan`, `paths`, `courses`, `extract`): test prima, copertura ≥ 80%
  (`cargo llvm-cov`). Mock HTTP con `wiremock` per il client.
- `cargo fmt`, `cargo clippy -- -D warnings` puliti prima di ogni commit.
- Nessun `unwrap()`/`expect()` nel codice di produzione. Errori tipizzati con `thiserror` e
  messaggi chiari in italiano nella UI.
- Funzioni sotto le 50 righe, file sotto le 400–800 righe, organizzazione per dominio.
- Dati immutabili dove possibile: le funzioni pure ricevono input e restituiscono nuove strutture.
- **Sicurezza:** token solo nel keyring, mai nei log né nei messaggi di errore e mai nelle fixture.
  Capabilities Tauri minime. Validare ogni input dei comandi. Il webview di login non deve avere
  accesso ai comandi Tauri.
- **Fixture:** le risposte grezze (`fixtures/raw/`) contengono dati personali, quindi stanno in `.gitignore`.
  Solo le versioni redatte vanno nel repo.
- **Operazioni distruttive:** l'app non elimina mai file senza conferma esplicita, mai file non
  scaricati da lei, e sempre tramite Cestino (§6.5). Ogni modifica a quel codice richiede test.
- Commit in formato convenzionale (`feat:`, `fix:`, `test:` …).
- Il codice in `_riferimenti/` è **solo da leggere**: si consulta per capire le API, non si copia.

---

## 13. Riferimenti

- v1 (GPL-3.0): https://github.com/toto04/webeep-sync. Copia locale in `_riferimenti/webeep-sync-v1/`.
  File chiave: `src/modules/login.ts`, `moodle.ts`, `download.ts`.
- BeepBar (client macOS, sync a tre vie, registrazioni): https://github.com/tommaso-vaccari/BeepBar
- Registrazioni WeBeep: https://github.com/aleo113/webeep-sync-recordings
- Moodle Web Service API: https://docs.moodle.org/dev/Web_service_API_functions
- Login app mobile di Moodle (`launch.php`): https://docs.moodle.org/dev/Moodle_Mobile_developer_tools
- Tauri 2: https://v2.tauri.app (plugin: notification, updater, autostart, single-instance, dialog, opener)

## 14. Contesto utente

- Sviluppato da uno studente Polimi su Windows 11, in italiano. I dati personali (percorsi,
  installazione della v1, file di prova con dati personali) stanno in `CLAUDE.local.md`, che non è
  nel repository.
- Corso di test per l'albero cartelle: **056892 Data Mining**.
- Nessun esempio di "cartella compressa" non scaricata per ora: raccoglierlo quando capita.
