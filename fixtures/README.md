# Fixture di test

Risposte reali di WeBeep (dump del 2026-11-09, Moodle 4.5.14), **redatte** con
`scripts/redact-fixtures.mjs`. Si usano nei test della logica pura (`tree.rs`, `courses/`, …).

```sh
node scripts/redact-fixtures.mjs fixtures/raw/<stamp>   # rigenera dalle risposte grezze
```

Cosa fa la redazione:

- nomi dei docenti → `DOCENTE A`, `DOCENTE B`, … (stessa lettera per lo stesso docente in tutti i file);
- altri nomi di persona → `REDACTED`; email → `redacted@example.com`; id utente → `0`;
- rimossi `courseimage`, `overviewfiles`, `contacts`, `userpictureurl`;
- lo script fallisce se nell'output resta un nome, l'username o un'email.

`fixtures/raw/` contiene le risposte **non** redatte: è in `.gitignore` e non va mai condivisa.

## File

| File | Origine | Cosa esercita |
|------|---------|---------------|
| `courses/timeline_{inprogress,past,all}.json` | `core_course_get_enrolled_courses_by_timeline_classification` | filtro corsi attivi, anno accademico (`coursecategory`), corso `CCS` sempre in corso, parsing di `fullname` |
| `contents/data_mining.json` | `core_course_get_contents` | `folder` con sottocartelle a 2 livelli, ZIP senza estensione, `url` esterno |
| `contents/data_information_quality.json` | idem | tutto il materiale dentro un `page` (21 file + `index.html`) |
| `contents/formal_languages_compilers.json` | idem | `page` con ZIP, 6 `folder`, `resource` |
| `contents/technologies_information_systems.json` | idem | 7 `resource` a file singolo |
| `contents/recommender_systems.json` | idem | `label` con immagini, 14 `url` esterni, `resource` |
| `contents/uncertainty_ai.json` | idem | `quiz`, `assign`, `folder`, `resource` |
| `contents/machine_learning.json` | idem | corso ancora senza materiale (solo forum e registrazioni) |
| `assignments/inprogress.json` | `mod_assign_get_assignments` | consegne dei corsi in corso |

Particolarità di `fullname` presenti nei dati: due gruppi tra parentesi
(`(PER IL SETTORE DELL'INFORMAZIONE) (DOCENTE B)`: il docente è l'ultimo), più docenti
nello stesso gruppo e una parentesi chiusa attaccata all'anno (`(DOCENTE G)[2024-25]`).
