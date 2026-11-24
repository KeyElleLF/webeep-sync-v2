//! Language of the texts the backend shows: command errors, per-file errors in the sync
//! summary, system notifications and file-dialog titles. Italian is written in the code (as
//! before); the English text comes from `EN`, where `{}` stands for a part that changes (a
//! path, a number, an inner error). Texts not in the table stay in Italian.
//!
//! The language is a process-wide setting (one user, one window), loaded at startup and changed
//! from the settings. Content from WeBeep is never translated (see `moodle::client`).

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    Italian,
    English,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::Italian => "it",
            Self::English => "en",
        }
    }

    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "it" => Some(Self::Italian),
            "en" => Some(Self::English),
            _ => None,
        }
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn current() -> Language {
    match CURRENT.load(Ordering::Relaxed) {
        1 => Language::English,
        _ => Language::Italian,
    }
}

pub fn set_current(language: Language) {
    CURRENT.store(u8::from(language == Language::English), Ordering::Relaxed);
}

/// `it` or `en`, for texts built in place (notifications with numbers).
pub fn pick<'a>(language: Language, it: &'a str, en: &'a str) -> &'a str {
    match language {
        Language::Italian => it,
        Language::English => en,
    }
}

/// The Italian `text` in the current language.
pub fn localize(text: &str) -> String {
    localize_in(current(), text)
}

pub fn localize_in(language: Language, text: &str) -> String {
    if language == Language::Italian {
        return text.to_owned();
    }
    EN.iter()
        .find_map(|(it, en)| fill(it, en, text))
        .unwrap_or_else(|| text.to_owned())
}

/// `text` read with the Italian template `it` (`{}` = any text), written with `en`.
fn fill(it: &str, en: &str, text: &str) -> Option<String> {
    let parts: Vec<&str> = it.split("{}").collect();
    let en_parts: Vec<&str> = en.split("{}").collect();
    if parts.len() != en_parts.len() {
        return None;
    }
    let mut rest = text.strip_prefix(parts[0])?;
    let mut captured = Vec::new();
    for (i, part) in parts.iter().enumerate().skip(1) {
        let at = if i == parts.len() - 1 {
            rest.strip_suffix(part)?.len()
        } else {
            rest.find(part)?
        };
        captured.push(&rest[..at]);
        rest = &rest[at + part.len()..];
    }
    if parts.len() == 1 && !rest.is_empty() {
        return None;
    }
    let mut out = en_parts[0].to_owned();
    for (piece, part) in captured.iter().zip(&en_parts[1..]) {
        out.push_str(piece);
        out.push_str(part);
    }
    Some(out)
}

/// Italian text (as written in the code) → English. Keep it in sync when adding messages.
const EN: &[(&str, &str)] = &[
    // Login and token
    (
        "impossibile accedere al portachiavi di sistema: {}",
        "cannot access the system keychain: {}",
    ),
    (
        "l'URL di ritorno non contiene un token",
        "the return URL contains no token",
    ),
    (
        "il token ricevuto non è codificato correttamente",
        "the received token is not encoded correctly",
    ),
    (
        "il token ricevuto ha un formato inatteso",
        "the received token has an unexpected format",
    ),
    (
        "la firma del token non è valida: login rifiutato",
        "the token signature is not valid: login refused",
    ),
    (
        "accesso annullato: la finestra di login è stata chiusa",
        "login cancelled: the login window was closed",
    ),
    (
        "una finestra di login è già aperta",
        "a login window is already open",
    ),
    (
        "impossibile aprire la finestra di login: {}",
        "cannot open the login window: {}",
    ),
    // WeBeep and the local database
    (
        "sessione scaduta: effettua di nuovo l'accesso",
        "session expired: log in again",
    ),
    (
        "WeBeep ha risposto con un errore ({}): {}",
        "WeBeep answered with an error ({}): {}",
    ),
    (
        "WeBeep ha risposto con lo stato HTTP {}",
        "WeBeep answered with HTTP status {}",
    ),
    (
        "errore di rete verso WeBeep: {}",
        "network error reaching WeBeep: {}",
    ),
    (
        "risposta di WeBeep non riconosciuta: {}",
        "unrecognised answer from WeBeep: {}",
    ),
    (
        "configurazione del client non valida: {}",
        "invalid client configuration: {}",
    ),
    (
        "database locale non disponibile",
        "local database not available",
    ),
    ("errore del database locale: {}", "local database error: {}"),
    (
        "il database locale è di una versione più recente dell'app ({}): aggiorna WeBeep Sync",
        "the local database comes from a newer version of the app ({}): update WeBeep Sync",
    ),
    // Folders and sync
    (
        "una sincronizzazione è già in corso",
        "a sync is already running",
    ),
    (
        "attendi la fine della sincronizzazione per cambiare cartella",
        "wait for the sync to finish before changing folders",
    ),
    (
        "attendi la fine della sincronizzazione",
        "wait for the sync to finish",
    ),
    (
        "scegli prima la cartella di destinazione",
        "choose the destination folder first",
    ),
    (
        "la cartella di destinazione non esiste più: sceglila di nuovo",
        "the destination folder no longer exists: choose it again",
    ),
    (
        "il percorso della cartella contiene caratteri non validi",
        "the folder path contains invalid characters",
    ),
    ("cartella non valida", "invalid folder"),
    (
        "il nome della cartella non è valido",
        "the folder name is not valid",
    ),
    (
        "la cartella «{}» è già usata da un altro corso",
        "the folder “{}” is already used by another course",
    ),
    (
        "impossibile spostare la cartella: {}",
        "cannot move the folder: {}",
    ),
    (
        "nome di cartella non valido: «{}»",
        "invalid folder name: “{}”",
    ),
    (
        "il corso non ha ancora una cartella: sincronizzalo",
        "the course has no folder yet: sync it first",
    ),
    (
        "impossibile creare la cartella del corso: {}",
        "cannot create the course folder: {}",
    ),
    (
        "impossibile aprire la cartella: {}",
        "cannot open the folder: {}",
    ),
    ("percorso non valido: «{}»", "invalid path: “{}”"),
    (
        "il file non c'è più: è stato spostato o eliminato",
        "the file is no longer there: it was moved or deleted",
    ),
    (
        "impossibile mostrare il file: {}",
        "cannot show the file: {}",
    ),
    ("semestre non valido", "invalid semester"),
    (
        "nessuna pulizia in attesa per questo corso",
        "no cleanup pending for this course",
    ),
    // Downloads and files (sync summary, cleanup)
    (
        "errore di rete durante il download: {}",
        "network error during the download: {}",
    ),
    ("impossibile scrivere «{}»: {}", "cannot write “{}”: {}"),
    (
        "link non scaricabile: non punta a un file di WeBeep",
        "link not downloadable: it does not point to a WeBeep file",
    ),
    (
        "WeBeep ha inviato {} byte invece di {}: file non salvato",
        "WeBeep sent {} bytes instead of {}: file not saved",
    ),
    (
        "il file è stato modificato durante la sincronizzazione: non è stato toccato",
        "the file changed during the sync: it was not touched",
    ),
    (
        "il file è cambiato nel frattempo: non è stato toccato",
        "the file changed in the meantime: it was not touched",
    ),
    (
        "il file non è più presente sul disco",
        "the file is no longer on disk",
    ),
    ("il file non c'è più", "the file is gone"),
    (
        "esiste già un file con il nuovo nome",
        "a file with the new name already exists",
    ),
    ("percorso non valido", "invalid path"),
    // Study plan
    ("scegli un file PDF", "choose a PDF file"),
    (
        "copia il PDF sul tuo PC: i file nelle cartelle di rete non vengono aperti",
        "copy the PDF to your PC: files in network folders are not opened",
    ),
    (
        "il file è troppo grande per essere la ricevuta del piano degli studi",
        "the file is too big to be a study plan",
    ),
    (
        "impossibile leggere il file: {}",
        "cannot read the file: {}",
    ),
    ("il PDF non si riesce a leggere", "the PDF cannot be read"),
    (
        "nel PDF non ci sono insegnamenti: serve la ricevuta del piano degli studi o la pagina Piano degli studi salvata in PDF",
        "no teachings in the PDF: use the study plan receipt or the \"Piano degli studi\" page saved as PDF",
    ),
    ("percorso del file non valido", "invalid file path"),
    ("file non valido", "invalid file"),
    (
        "il testo è troppo lungo: incolla solo la tabella degli insegnamenti",
        "the text is too long: paste only the table of teachings",
    ),
    (
        "non ho trovato insegnamenti: copia la tabella della pagina Piano degli studi",
        "no teachings found: copy the table of the \"Piano degli studi\" page",
    ),
    ("lingua non valida", "invalid language"),
    // File dialogs
    (
        "Scegli la cartella in cui sincronizzare i corsi",
        "Choose the folder to sync your courses into",
    ),
    (
        "Scegli la ricevuta del piano degli studi (PDF)",
        "Choose the study plan (PDF)",
    ),
    ("Salva il file di diagnostica", "Save the diagnostics file"),
    (
        "impossibile salvare il file: {}",
        "cannot save the file: {}",
    ),
    // Development tools
    (
        "cartella dati dell'app non disponibile: {}",
        "app data folder not available: {}",
    ),
    (
        "il dump è disponibile solo nelle build di sviluppo",
        "the dump is only available in development builds",
    ),
    ("lingua del dump non valida", "invalid dump language"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use Language::{English, Italian};

    #[test]
    fn italian_is_the_text_itself() {
        assert_eq!(
            localize_in(Italian, "una sincronizzazione è già in corso"),
            "una sincronizzazione è già in corso"
        );
    }

    #[test]
    fn fixed_texts_are_translated() {
        assert_eq!(
            localize_in(English, "una sincronizzazione è già in corso"),
            "a sync is already running"
        );
        assert_eq!(localize_in(English, "percorso non valido"), "invalid path");
        assert_eq!(
            localize_in(English, "percorso del file non valido"),
            "invalid file path"
        );
    }

    #[test]
    fn variable_parts_are_kept() {
        assert_eq!(
            localize_in(
                English,
                "impossibile scrivere «Materiali/a.pdf»: accesso negato"
            ),
            "cannot write “Materiali/a.pdf”: accesso negato"
        );
        assert_eq!(
            localize_in(
                English,
                "WeBeep ha inviato 10 byte invece di 20: file non salvato"
            ),
            "WeBeep sent 10 bytes instead of 20: file not saved"
        );
        assert_eq!(
            localize_in(English, "errore di rete verso WeBeep: timeout"),
            "network error reaching WeBeep: timeout"
        );
    }

    #[test]
    fn a_fixed_text_does_not_match_a_longer_one() {
        // "cartella non valida" must not eat "cartella non valida: x".
        assert_eq!(
            localize_in(English, "cartella non valida: x"),
            "cartella non valida: x"
        );
    }

    #[test]
    fn unknown_texts_stay_as_they_are() {
        assert_eq!(
            localize_in(English, "qualcosa di nuovo"),
            "qualcosa di nuovo"
        );
    }

    #[test]
    fn every_entry_has_matching_placeholders_and_translates_itself() {
        for (it, en) in EN {
            assert_eq!(it.matches("{}").count(), en.matches("{}").count(), "{it}");
            let sample = it.replace("{}", "X");
            let expected = en.replace("{}", "X");
            assert_eq!(localize_in(English, &sample), expected, "{it}");
        }
    }

    #[test]
    fn languages_round_trip_through_their_codes() {
        for language in [Italian, English] {
            assert_eq!(Language::parse(language.code()), Some(language));
        }
        assert_eq!(Language::parse("fr"), None);
    }
}
