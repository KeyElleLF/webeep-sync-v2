# WeBeep Sync

Client desktop **non ufficiale** che sincronizza in locale il materiale didattico di
[WeBeep](https://webeep.polimi.it), il Moodle del Politecnico di Milano.

> Progetto non ufficiale, non affiliato al Politecnico di Milano.

## Cosa fa

- **Scarica il materiale dei corsi** dell'anno accademico in corso, divisi per semestre:
  cartelle annidate, file dentro le pagine, allegati delle consegne, ZIP con l'estensione giusta.
- **Non sovrascrive mai i tuoi file**: se modifichi delle slide e il docente le aggiorna, la
  nuova versione viene salvata accanto («… (WeBeep aggiornato AAAA-MM-GG).pdf»).
- **Bacheca degli avvisi** e **scadenze** (consegne e quiz) dei prossimi 14 giorni, con notifiche.
- **Corsi superati**: li segni come superati e l'app propone di spostare i file nel Cestino
  (solo quelli che ha scaricato lei, mai quelli che hai modificato o aggiunto tu).
- Può riusare la cartella di **WeBeep Sync v1**: i file già presenti e identici non vengono
  riscaricati.

L'app lavora solo mentre è aperta: chiudendo la finestra si chiude tutto.

## Installazione (Windows)

1. Scarica `WeBeep Sync_x.y.z_x64-setup.exe` dalla pagina delle release.
2. Avvialo. Non servono permessi di amministratore.
3. Windows potrebbe mostrare **«Windows ha protetto il PC»** (SmartScreen): succede con tutte le
   app non firmate con un certificato a pagamento. Clicca **«Ulteriori informazioni» →
   «Esegui comunque»**.
4. Apri **WeBeep Sync** dal menu Start.

Disinstallando l'app, i file scaricati restano dove sono.

## Primo avvio

1. **Accedi a WeBeep** con il tuo account Polimi: si apre la pagina di login ufficiale.
2. **Scegli la cartella** dove salvare i corsi (anche quella di WeBeep Sync v1).
3. Controlla i corsi selezionati e premi **«Sincronizza ora»**.

## Domande frequenti

- **Dove sono i file?** Nella cartella scelta, una sottocartella per corso. Il nome della
  cartella di un corso si può cambiare dalla lista dei corsi («Cambia»).
- **Come segno un corso come superato?** Nella lista dei corsi, «Superato». Vale per tutti i corsi
  dello stesso insegnamento, anche degli anni precedenti.
- **Dove sono i log?** In `%LOCALAPPDATA%\it.webeepsync.app\logs` (una settimana di storico).
  Non contengono la password né il token di accesso.

## Crediti

Riscrittura in Rust + Tauri 2 ispirata a [WeBeep Sync](https://github.com/toto04/webeep-sync)
di **toto04** (GPL-3.0), usato come riferimento per capire le API di WeBeep.

## Privacy

- Nessun server e nessuna telemetria: l'app parla solo con `webeep.polimi.it`.
- Il login avviene sulla pagina ufficiale del Politecnico, in una finestra privata dedicata.
- Il token di accesso è salvato solo nel portachiavi del sistema (Gestione credenziali di
  Windows, Portachiavi di macOS, Secret Service su Linux) e non viene mai scritto nei log.

## Sviluppo

Prerequisiti: Rust stable (MSVC su Windows), Node 20+, WebView2.

```sh
npm install
npm run tauri dev        # avvia l'app in modalità sviluppo
npm run build:installer  # installer in src-tauri/target/release/bundle/nsis/ (senza percorsi personali nell'exe)
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Il brief completo del progetto è in [`CLAUDE.md`](CLAUDE.md).

## Licenza

[GPL-3.0](LICENSE)
