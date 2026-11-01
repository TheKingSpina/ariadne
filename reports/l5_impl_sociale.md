# L5 — implementazione sociale (resoconto producer, ciclo test-first)

- **Mandato:** ottavo (implementazione L5, ciclo A→B→C; i commit sono 2 su main, nessun push).
- **Repo:** `/Users/spina/Ariadne` — HEAD finale: `4f9e834`
  (`bec5e49` riscritto da amend solo-messaggio senza cambiamento di contenuto:
  tre parole corrotte del testo di commit; il contenuto del diff è identico).
- **Modello:** `opencode/glm-5.3-flash-free` (provider `opencode`, variante:
  **nessuna** — default del catalogo), classe `light`. Canale «Free-first».
- **Sessione Mnemosyne:** `ses_3927a9dd-74a0-4e31-b5b5-33d806f73a75`.

## Due commit

1. **`4374e91`** — SOLO test: `tests/l5_social.rs` (+608 righe), 18 test
   rossi (binario test non compilabile: `E0432 unresolved import
   ariadne::social` — rosso onesto del test-first; `cargo test` eseguito e
   registrato prima dell'implementazione). Nessun codice di produzione.
2. **`4f9e834`** — implementazione: `src/social.rs` (+636 righe) + UNA riga
   `pub mod social;` in `src/lib.rs`. Zero file fuori perimetro nel commit
   (il working tree contiene untracked NOT-to-committed di lavori di
   altri ruoli: lasciat Agricultural).

## Verifica

- `cargo test` (debug): **164/164** verdi, 0 falliti (146 pre-esistenti L0–L4
  + 18 L5). ZERO warning sopravvissuti (2 warning interni rimossi prima del
  commit 2).
- `cargo test --release`: **164/164** verdi — identico al debug.
- Nessun test L0–L4 pre-esistente modificato o reso rosso.

## I 18 test L5 (esito finale: tutti VERDI)

1. `l5_check1_genesis_only_models_empty_head_hash_is_genesis`
2. `l5_check1_replay_is_byte_per_byte`
3. `l5_check2_frontiera_no_l5_writes_in_log`
4. `l5_check3_predict_source_seqs_and_no_model`
5. `l5_check4_divergenza_irrelevant_events_leave_model_identical`
6. `l5_check4_divergenza_relevant_observation_changes_model`
7. `l5_check5_second_order_composition_equals_direct_prediction`
8. `l5_check6_malformed_l5_payload_is_schema`
9. `l5_check6_malformed_osservazioni_is_schema`
10. `l5_check6_osservazioni_on_non_sense_is_schema`
11. `l5_check6_l5_mixed_with_other_tags_is_schema`
12. `l5_check6_base_incoerente_is_head_match`
13. `l5_check7_window_update_mismatch_one_decision`
14. `l5_check7_window_equal_observation_no_update`
15. `l5_check7_second_predict_replaces_anchor`
16. `l5_check7_l5_in_base_prefix_gives_zero_decisions`
17. `l5_check7_l5_event_closes_l2_window_no_side_effects`
18. `l5_check8_zero_draw_and_deterministic_decisions`

## Correzioni meccaniche ai test

**ZERO.** I 18 test sono tornati VERDI alla prima compilazione
dell'implementazione: la delega dell'orchestratore sulle correzioni
meccaniche non è stata esercitata. Nessuna aspettativa semantica
contraria alla legge, nessun punto di tacevità incontrato. Nessun FERMO.

## Igiene interna (dichiarata nel messaggio del commit 2, nessun impatto semantico)

- rimossa la riga residua di primo getto `let _ = &e;` in `replay_upto`;
- campo `Accum.id` mai letto rimosso (l'id è già la chiave della mappa);
- campo `Replay.anchors` mai letto rimosso (la semantica delle finestre
  vive interamente in `evaluate_social`, L5.7: cammino in ordine di seq,
  anchor = predict più recente sostitutivo);
- in `select` del dominio predict la read del payload `sources` resta una
  validazione di FORMA sola (contenuto autorevole, L2.11).

## Interpretazioni di legge (1 sola, già dichiarata)

- **`id_altro` privato** (fuori dall'elenco pubblico chiuso L5.10, che
  elenca solo i tipi/le API della legge): pattern `node_id` di L3/L4 — gli
  helper di id restano interni al modulo e i test calcolano l'id in modo
  indipendente. Confermata, non rettificata.
- ZERO altre interpretazioni. Le scelte di costruzione derivanti
  direttamente dagli articoli: parser `osservazioni` canonico rigido
  byte-per-byte (L5.2), `at` valido come range di seq esistenti (L5.6),
  `sources` di `model_update` vincolata alla coppia citata `[p, o]`
  (L5.7 + precisazione della firma), membri multipli della stessa SENSE
  che concorrono in ordine di membro per `ultimo` (L5.5, ultimo
  dell'ultima SENSE: lettura dell'ordine di scorrimento «per seq»).

## Perimetro file (esattamente il contratto)

- scritti: `tests/l5_social.rs` (commit 1), `src/social.rs` + 1 riga
  `src/lib.rs` (commit 2), `reports/l5_impl_sociale.md` (questo report,
  NON committato);
- NON toccati: `src/{cycle,prng,identity,memory,selection,log,event,view,payload,state}.rs`,
  leggi, SPEC, policy, altri test;
- nessun push, esattamente 2 commit (`4374e91`, `4f9e834`).

## Checklist non dichiarata

La checklist di uscita della legge L5 è esercitata dai test (1–8 coperti),
ma il gate spetterà al tester e l'accettazione all'umano a fine ciclo: qui
si dichiara SOLO lo stato di fatto sopra riportato.

---

## Aggiunta sezione: patch R1 (rilievo audit — test-first, 2 commit)

- **Rilievo:** `reports/audit_l5_impl.md` R1 (violante, unico NON CONFORME)
  — un evento SENSE con chiave `osservazioni` di valore NON-`Value::Str`
  veniva accettato SILENZIOSAMENTE (`if let Some(Value::Str(raw))` senza
  ramo di rifiuto) in `src/social.rs:385` (`replay_upto`) e `:583`
  (`evaluate_social`), contro L5.2 (`notes/L5_legge_sociale.md`, commit
  `23a7104`, righe 59–70) e la checklist 6; regola identica già in vigore
  in `src/selection.rs:47` e `src/identity.rs:283`.
- **Commit 1 (test rosso): `a2e6aee`** — solo `tests/l5_social.rs`
  (+44 righe): test `l5_check_osservazioni_non_f1_is_schema` (4 casi:
  `Value::Int` su SENSE — rosso onesto, anche su `evaluate_social`;
  JSON abbreviato di stringhe; oggetto top-level senza array; spazi
  fuori canone). Rosso verificato con `cargo test --test l5_social`
  prima del fix: `build_social` su `Value::Int(5)` → `Ok`
  (accettazione silenziosa). Nessun codice di produzione.
- **Commit 2 (fix minimo): `77070b7`** (composto originariamente
  `61fc85d` e rinsaldato da amend solo-messaggio per due parole
  corrotte del testo; contenuto immutato) — la validazione F1 RIGIDA
  nei SOLI DUE luoghi dichiarati: match sul valore della chiave →
  `Value::Str` → parser canonico; qualunque altro valore →
  `SocialError::Schema`. Nessun altro punto del modulo toccato.
- **Correzione meccanica al test (dichiarata, autorizzazione
  preventiva dell'orchestratore):** rimozione di un residuo di primo
  getto dell'edizione del commit 1 — commento + attributo `#[test]`
  DUPPLICATI sopra `l5_check6_base_incoerente_is_head_match`, che
  causavano la DOPPIA esecuzione dello stesso test (il binario
  `l5_social` eseguiva 20 test anziché 19: il duplicato era già
  presente a commit-time). Nessuna semantica di test cambiata: stessa
  funzione, resta UN test.
- **Verifica:** `cargo test` debug **165/165** (146 L0–L4 invariati +
  19 L5 distinti, 0 falliti); `cargo test --release` **165/165**,
  identici al debug.
- **Modello:** `opencode/glm-5.3-flash-free` (classe `light`, variante:
  nessuna — default del catalogo), canale «Free-first», 31/10/2026.
- **Sessione Mnemosyne:** vedi resoconto finale del mandato
  (proposta + chiusura con job id).

Checklist non dichiarata: gate al tester, accettazione all'umano —
qui solo lo stato di fatto sopra riportato.

---

**Footer di provenienza.** Modello
`opencode/glm-5.3-flash-free` (classe `light`; variante: nessuna —
default del catalogo), assegnato dal Router (policy `.opencode/routing.md`
v1, ATTIVA, canale «Free-first»), 31/10/2026. Test-first visto: `4374e91`
(rossi) → `4f9e834` (verdi) → `a2e6aee` (rossi R1) → `77070b7` (verdi).

---

## Aggiunta sezione: copertura simmetria l5-key + smentita R1-ter

- **Rilievo chiuso:** R1-ter (re-audit, `reports/audit_l5_impl.md` append)
  — «in `evaluate_social` un evento con chiave `l5` non-stringa/fuori
  dominio viene saltato silenziosamente rispetto al replay». SMENTITA con
  prova: `replay_upto` valida la chiave `l5` PRIMA del cammino segnalato a
  `:580` — `str_field` (`:416`), match dominio (`:417`), ramo non-ACTION
  (`:408`) — la validazione copre sempre l'intero log ed è chiamata da
  `evaluate_social` (`:561–562`) e da `build_social`: sul log patologico le
  due API sono già simmetriche (`SocialError::Schema` entrambe).
- **Disciplina test-first LODATA:** al decimo mandato il Producer si è
  fermo CORRETTAMENTE quando il test proposto risultò verde alla stesura
  (nessun commit di un falso rosso); il rilievo è stato riportato con
  prove invece di applicarne il no-op.
- **Commit di copertura: `cdcdfce`** (unico nel mandato; originariamente
  `5198a1f` e rinsaldato da amend solo-messaggio per testo — contenuto
  immutato): SOLO `tests/l5_social.rs` (+43 righe), test
  **`l5_check5_l5_key_pathologies_symmetric_schema`** — 3 casi (`l5` =
  `Value::Int` su ACTION; `l5` = `"foo"` fuori dominio; `l5` non-Str su
  SENSE non-ACTION) con **asserzioni su ENTRAMBE le API**. `src/social.rs`
  e `src/lib.rs` INTATTI (nessun fix no-op).
- **Autorizzazioni citate nel messaggio:** pattern coperture `0f01211`
  (L3) e `8d82f47` (L4), firma «test subito» 31/10/2026, decisione
  dell'orchestratore del 01/11/2026 estensione opzione (a); legge
  `notes/L5_legge_sociale.md` (`23a7104`) L5.1/L5.2; canale «Free-first».
- **Esiti:** `cargo test` debug **166/166** (146 L0–L4 + 20 L5), zero
  falliti; `cargo test --release` **166/166**, identici al debug.
- **Modello:** `opencode/glm-5.3-flash-free` (classe `light`, variante:
  nessuna — default del catalogo), canale «Free-first», 01/11/2026.
- **Sessione Mnemosyne:** sessione del mandato undicesimo (vedi resoconto
  finale del mandato: apertura di sessione, proposta, chiusura con job id).
