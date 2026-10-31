# Fix L2 R1/R2/R4 — test-first da `2a9e18e`

**autore di trascrizione: orchestratore** (revisione finale). Il producer
(`openai/gpt-6.1-terra`, classe medium, sessione strumento
`ses_e6c8386afffe2JOt1mFaltgPOY`, Mnemosyne `ses_73189cec-c731-44f3-8aca-638ac1dfefff`)
ha rifiutato per due volte la stesura di questo report («divieto di ruolo»),
contrariamente al mandato Router; per provvedimento è trascritto qui senza
consumare ulteriori modelli. Contenuto ricostruito dal registro Router
(`reports/routing_fix_l2_r1_r2.md`) e dal codice/commits in repo.

## Autorizzazioni

- Fix e percorso test-first: umano, 31/10/2026 — «autorizzo il fix col
  percorso consueto test-first, e voglio un caso multi-arco nel nuovo test
  perché quel buco non si riapra mai più. R2 stessa cura: niente default
  silenziosi, tau assente = errore come prescrive L2.2».
- Chiarimento legislativo L2.27 (riprova dell'arco, Lettura A): umano,
  31/10/2026, commit `51e69f6`.

## Commit 1 — test rossi (`6cdf7b49c2fe54b6a078d797d12a8df602815ab5`)

Solo test, autorizzazione citata nel messaggio. Rossi con il codice vigente
(suite L2: 13 passati / 2 falliti):

1. `distinct_edge_events_activate_only_their_own_endpoints` (`tests/l2_selection.rs:166`)
   — caso multi-arco obbligatorio: due archi distinti dichiarati in eventi a
   seq diverse; nessun cross-attivazione.
2. `in_window_select_without_config_is_error` (`:178`) — R2: select in finestra senza
   config → `L2Error::Config`.

## Commit 2 — implementazione minima (`610b43ce57a13234c9e54b7d58967d9d0ff45577`)

Messaggio con riferimento a L2.27 e all'autorizzazione umana.

- **R1** (`src/selection.rs:67-68`): le attivazioni per `edges` derivano
  SOLO dagli archi dichiarati nell'evento stesso (helper `edge_mentions`
  sul payload dell'evento, id mappati sulla vista L1, endpoint non
  mappabile → `L2Error::Schema`). Nessun ciclo su `view.edges`. Il dedup
  esistente garantisce una sola attivazione per elemento per seq (L2.13).
- **R2** (`src/selection.rs:51`): `tau` assente/config assente →
  `Err(L2Error::Config)`; nessun default implicito (L2.2).
- **R4**: rimosso `use EventType` inutilizzato (`src/selection.rs:3`).
- Test aggiunto nel commit 2 (fissaggio L2.27, dichiarato non-rosso):
  `repeated_edge_reactivates_endpoints_on_each_event` (`:186`) — riprova
  reiterata dello stesso arco: gli endpoint si riattivano a ogni
  dichiarazione (Lettura A).
- Perimetro verificato dal gate (`51e69f6..610b43c`): solo
  `src/selection.rs` + `tests/l2_selection.rs`; `src/memory.rs` e API
  pubbliche invariati; zero I/O, zero wall-clock, zero RNG.

## Verifica di suite

`cargo test` debug e release: **122/122 e 122/122** (119 preesistenti +
3 nuovi), esiti confermati indipendentemente dal gate tester
(`reports/gate_fix_l2_r1_r2.md`, VERDE) e dal re-viewer
(`reports/re_review_fix_l2_r1_r2.md`, CONFORME — R1/R2/R4 chiusi).

## Ambiguità di legge

Unica emersa (riprova dell'arco), risolta prima dell'implementazione
dall'emendamento L2.27; nessuna interpretazione autonoma del producer,
nessuna autoria silenziosa. Nessuna ambiguità residua.
