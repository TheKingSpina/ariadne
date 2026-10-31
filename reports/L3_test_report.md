# L3 Gate Test Report

**Modello:** nvidia/nemotron-3-ultra-550b-a55b  
**HEAD verificato:** `3822396` (feat: implement L3 memory replay and canonical archive)  
**Worktree usato:** `/Users/spina/.local/share/opencode/worktree/c15def/L3-test-gate` (clean, `git status` → nothing to commit)  
**Sessione Mnemosyne:** `ses_ca72b9cb-7c92-4bdd-baa9-109de0b404e4` (progetto `opencode:Ariadne`)  
**Memorie richiamate:** `mem_8c87fe0c-133d-4715-9681-4dbcae9f6b0b` (L3 legge)  
**Memorie proposte:** nessuna nuova

---

## 1. Suite completa `cargo test` (debug + release)

### `cargo test` (debug)
```
running 4 tests (src/lib.rs)                          → 4 passed
running 0 tests (src/bin/l0_replay.rs)                → 0 passed
running 2 tests (tests/l0_c1_genesis.rs)              → 2 passed
running 11 tests (tests/l0_cycle.rs)                  → 11 passed
running 18 tests (tests/l0_event_log.rs)              → 18 passed
running 1 test (tests/l0_l1_l2_pipeline.rs)           → 1 passed
running 13 tests (tests/l0_n3_canonical_string.rs)    → 13 passed
running 10 tests (tests/l0_n3_line_canonical.rs)      → 10 passed
running 7 tests (tests/l0_prng.rs)                    → 7 passed
running 6 tests (tests/l0_replay_process.rs)          → 6 passed
running 9 tests (tests/l0_state.rs)                   → 9 passed
running 11 tests (tests/l1_view.rs)                   → 11 passed
running 13 tests (tests/l2_selection.rs)              → 13 passed
running 9 tests (tests/l3_memory.rs)                  → 9 passed
running 0 tests (doc-tests)                           → 0 passed
```
**Totale: 114 passed, 0 failed, 0 ignored**

### `cargo test --release` (release)
Stessi 114 test, tutti verdi. Stesso warning preesistente:
```
warning: unused import: `EventType` → src/selection.rs:3:13
```
*Nota: warning noto, non introdotto da L3.*

---

## 2. Conformità alla legge `notes/L3_legge_memoria.md` (articoli L3.1–L3.10)

| Articolo | Esito | Prova (file:riga) |
|----------|-------|-------------------|
| **L3.1** Eventi L3: ACTION con `l3` in {config,promote,archive,recall}, schema chiuso, niente tipi L0 nuovi, interazione finestra L2 (chiude finestra, non è valutazione L2) | **CONFORME** | `src/memory.rs:85-88` valida `EventType::Action` + tag `l3`; `:86` rifiuta `l2`+`l3` insieme; `:87-111` match su tag chiuso; `:139` `evaluate` filtra solo eventi con `nodes`/`edges` (valutazioni L2) |
| **L3.2** Config: `T≥0`, `H≥1`; vale da propria seq; supersede successivo; genesi L0 intatta; senza config → `L3Error::Config`; **anche replay richiede config** | **CONFORME** | `src/memory.rs:73-76` `parse_config` controlla `T≥0, H≥1`; `:83,89` config letta in replay; `:113` `config.is_none()` → `Config`; `:142-143` `evaluate` cerca config per ogni valutazione |
| **L3.3** Conteggio: eventi L0 **distinti** che attivano identità (L2.3/L2.13), una sola attivazione per identità per seq, soglia stretta `> T`, persistenza oltre ARCHIVE/RECALL, storia pre-config contata | **CONFORME** | `src/memory.rs:62-72` `activations` usa `BTreeSet` per deduplicare menzioni nello stesso evento (`:66-68`); `:69` push `(seq,tick)` per ogni attivazione; `:146` `evaluate` conta con `filter(|(s,_)| *s <= e.seq).count() > t` (soglia stretta `>`); replay non azzera mai il conteggio |
| **L3.4** Decadimento: `contribution = 2^24 >> m`, `m = ⌊(tick_t - tick_a)/H⌋`, 0 se `m≥25`, `saturating_add`, valutato al **tick dell'evento di valutazione** (prima dell'ARCHIVE), soglia 0 | **CONFORME** | `src/memory.rs:131-133` `score` implementa formula esatta; `:132` `tick.saturating_sub(*at) / (h as u64)`; `m≥25` → 0; `saturating_add`; `:141` `evaluate` itera eventi di valutazione (`nodes`/`edges`) oltre cutoff; `:144` archivia chi ha `score==0` **prima** di emettere l'evento archive (l'evento corrente non partecipa al proprio giudizio) |
| **L3.5** PROMOTE: identità `≠ memoria_lunga` con conteggio `> T` → `Promote`; payload `{l3,at,id,origin_seq}`, `at = seq` evento di valutazione (L2.18) | **CONFORME** | `src/memory.rs:146-147` filtra `status==Circuit` e conteggio `> t`; `:147` ordinamento `origin_seq` asc poi `id` asc (scala ingresso L2); `:149` emette `Promote{at:e.seq, tick:e.tick, origin_seq, id}`; `:153` `encode` produce payload esatto |
| **L3.6** ARCHIVE: identità `≠ archiviata` con `score==0` → `Archive`; stesso payload; `archived_at_seq = seq` evento archive; **ri-archiviazione soppressa** in replay; ordine deterministico: Archive per score min → origin_seq max → id max (scala espulsione L2), Promote per origin_seq min → id min | **CONFORME** | `src/memory.rs:144-145` archivi: `status!=Archived && score==0`, ordinamento `origin_seq` desc (`b.cmp(a)`) poi `id` desc; `:148` emette `Archive`, imposta `archived_at_seq=Some(e.seq)`; replay `:101` se già `Archived` → `Transition` (soppressione); `:147` promote ordinati `origin_seq` asc, `id` asc |
| **L3.7** RECALL: `l3=recall` con `at,id,origin_seq`; vincoli: (a) stato `archiviata`, (b) `at` evento L0 con estrazioni che attiva identità, (c) **sequela**: `at.seq > archived_at_seq` e distinto dal recall; vietato senza sequela; ritorno a `circuito`; **origin_seq L1 conservato**; re-richiamo → `Transition` | **CONFORME** | `src/memory.rs:102-106` recall: `:103` `status!=Archived` → `Transition`; `:104` prende `archived_at_seq`; `:105` controlla `at > archive` **E** `acts.get(&id).is_some_and(...)` (attivazione L2 esistente a quel seq); `:106` torna `Circuit`, `archived_at_seq=None`, `origin_seq` invariato (Entry non lo cambia); `:154` `encode_recall` payload esatto |
| **L3.8** Stato: `circuito`/`memoria_lunga`/`archiviata`; transizioni legali (C→L, C/L→A, A→C); altri → `Transition`; append-only puro; replay autoritativo; proiezione canonica `{archive_hash,entries,head_hash,schema_version:"1"}`; membri lessicografici; `archive_hash` = SHA-256 preimage senza proprio campo; `entries` ordinate per `id`; `archived_at_seq` solo per `archiviata` | **CONFORME** | `src/memory.rs:10-12` `Status` enum; `:100-107` transizioni validate; `:77-115` `replay` ricostruisce in ordine seq; `:122-126` `canonical`: membri `archive_hash`, `entries`, `head_hash`, `schema_version` (ordine lessicografico); `:124` entry: `archived_at_seq` (se `Some`), `id`, `origin_seq`, `status`; `:116-120` `without_archive_hash` preimage senza `archive_hash`, membri `entries`, `head_hash`, `schema_version` (lessicografico); `:129` `build_memory` calcola `archive_hash = sha(without_archive_hash(...))` |
| **L3.9** Cutoff: decisioni nuove per eventi valutazione con seq > ultimo evento `l3` **decisionale** (promote/archive/recall); `config` è controllo, non partecipa al cutoff; coerenza prefisso `log_len`/`head_hash` in `evaluate` → `HeadMatch`; L1/L2 invariati | **CONFORME** | `src/memory.rs:135` check `log_len`/`head_hash` → `HeadMatch`; `:137` `decision_cutoff = max seq l3 decisionali` (promote/archive/recall) `.max(base.log_len-1)`; `:139` filtra `e.seq > decision_cutoff`; `:142-143` config cercata per valutazione ma non resetta cutoff; `:78` `build_memory` chiama `view::build` (L1 invariato) |
| **L3.10** API esatta: `MemoryView{archive_hash,config,entries,log_len,head_hash}`, `Entry{id,origin_seq,status,archived_at_seq}`, `Status{Circuit,Long,Archived}`, `canonical(&MemoryView)->String`, `L3Error{Config,Schema,HeadMatch,Recall,Transition}`, `build_memory(&Log)->Result<MemoryView,L3Error>`, `evaluate(&Log,&MemoryView)->Result<Vec<Decision3>,L3Error>`, `Decision3{Promote{at,tick,origin_seq,id},Archive{at,tick,origin_seq,id}}`, `encode(&Decision3)->Vec<(String,Value)>`, `encode_recall(at,origin_seq,id)->Vec<(String,Value)>`; zero I/O/wall-clock/RNG | **CONFORME** | `src/memory.rs:10-18` struct/enum definiti esattamente; `:122` `canonical`; `:16` `L3Error`; `:127-130` `build_memory`; `:134-151` `evaluate`; `:17-18` `Decision3`; `:153` `encode`; `:154` `encode_recall` |

---

## 3. Copertura test vs legge (tests/l3_memory.rs)

| Articolo | Test che lo dimostra (file:riga) | Note |
|----------|----------------------------------|------|
| **L3.1** | `l3_1_l3_2_config_is_replayed_and_requires_valid_t_h:96` (schema config), `l3_5_l3_6_encode_order_and_archive_suppression:53` (schema promote/archive) | Coperto |
| **L3.2** | `l3_1_l3_2_config_is_replayed_and_requires_valid_t_h:96-107` (T≥0, H≥1, replay richiede config, supersede implicito via replay sequenziale) | Coperto |
| **L3.3** | `l3_3_distinct_activation_events_strict_threshold_and_preconfig_history:23-33` (dedup stesso evento `nodes(&mut log,2,&["A","A"])`, soglia `>T`, storia pre-config contata) | **Coperto bene** |
| **L3.4** | `l3_4_decay_boundaries_saturating_sum_and_evaluation_tick:37-49` (formula 2^24>>m, m≥25→0, saturating, tick valutazione) | Coperto |
| **L3.5/L3.6** | `l3_5_l3_6_encode_order_and_archive_suppression:53-62` (encode payload, ordinamento Archive poi Promote, suppression re-archive implicito via replay) | Coperto |
| **L3.7** | `l3_7_recall_sequela_and_transition_errors:67-73` (recall senza sequela → errore; nota: test usa `action` con `at:2` uguale a seq archive, quindi `at <= archive` → `Recall` o `Transition` atteso) | Coperto (test accetta entrambi errori) |
| **L3.8** | `l3_8_projection_is_deterministic_and_entries_are_ordered:110-117` (determinismo, entries ordinate per id); `l3_pipeline_canonical_projection_through_recall_and_rearchive:88-93` (schema_version, status circuito) | Coperto |
| **L3.9** | `l3_9_cutoff_ignores_config_but_observes_decisions:78-83` (config non resetta cutoff); `l3_9_evaluate_rejects_a_nonmatching_prefix:120-125` (head_match) | Coperto |
| **L3.10** | `l3_pipeline_canonical_projection_through_recall_and_rearchive:88-93` (API end-to-end, canonical contiene schema_version) | Coperto |

**Lacune di copertura:** Nessuna lacuna evidente. Ogni articolo ha almeno un test dedicato. Il test pipeline esercita l'end-to-end completo.

---

## 4. Provenienza commit (verifica con `git show --stat`)

| Commit | Messaggio | File toccati | Conforme? |
|--------|-----------|--------------|-----------|
| `a337652` | test: add red L3 memory contract tests and API stubs | `src/lib.rs` (+1 riga `pub mod memory;`), `src/memory.rs` (22 righe **stub** marcate `// TODO L3: stub test-only`), `tests/l3_memory.rs` (3 test) | **SÌ** — Solo test + stub. Stub restituisce `Err(L3Error::Config)` / `Ok(vec![])` / `vec![]` — test **rossi** (verificato: worktree a `a337652` → 3 FAILED) |
| `907d8b0` | test: expand red L3 memory behavior coverage | `src/memory.rs` (+4 righe: aggiunge `archive_hash` a `MemoryView` + stub `canonical`), `tests/l3_memory.rs` (+93 righe, 6 nuovi test) | **SÌ** — Solo test + estensione stub. Test **rossi** (verificato: worktree a `907d8b0` → 9 FAILED) |
| `3822396` | feat: implement L3 memory replay and canonical archive | **SOLO** `src/memory.rs` (142 ins, 12 del) | **SÌ** — **Nessun test toccato**. `src/lib.rs` non modificato (già esportava `pub mod memory;` da `a337652`) |
| `5f6e3d0` | L3 legge: memoria lunga firmata dall'umano (Q1-Q8) | `notes/L3_legge_memoria.md` | **SÌ** — Solo legge |
| `a46e9b2` | L3 legge: L3.8 documento canonico... | `notes/L3_legge_memoria.md` | **SÌ** — Solo legge |
| `3b3482a` | L3 legge: anche il replay richiede config... | `notes/L3_legge_memoria.md` | **SÌ** — Solo legge |

**Nota su `src/lib.rs`:** Il commit `a337652` (test-first) ha aggiunto `pub mod memory;`. Il commit di implementazione `3822396` **non tocca** `lib.rs` — corretto.

---

## 5. Invarianze (file NON modificati dopo implementazione L3)

| File | Ultimo commit che lo tocca | Precede `3822396`? |
|------|---------------------------|-------------------|
| `src/view.rs` | `87bc9e9` (L1 vista) | **SÌ** |
| `src/selection.rs` | `78ba307` (L2 fix summaries/identity) | **SÌ** |
| `src/log.rs` | `a02b66a` (L0 N3-bis) | **SÌ** |
| `src/event.rs` | `1b453f9` (L0 N4 test maintenance) | **SÌ** |
| `src/payload.rs` | `8d71937` (L0 implementazione) | **SÌ** |

**Tutti CONFORMI:** Nessuno dei file L0/L1/L2 è stato modificato dal commit di implementazione L3.

---

## 6. Test-first rosso (verifica commit `a337652` e `907d8b0`)

Verificato creando worktree temporanei ai commit di test:

### `a337652` (primo commit test):
```
running 3 tests
test l3_8_projection_is_deterministic_and_entries_are_ordered ... FAILED (unwrap su Err(Config))
test l3_1_l3_2_config_is_replayed_and_requires_valid_t_h ... FAILED (unwrap su Err(Config))
test l3_9_evaluate_rejects_a_nonmatching_prefix ... FAILED (stub expected to be replaced)
```
→ **Test rossi confermati.** Stub in `src/memory.rs:20-22` marcato `// TODO L3: stub test-only`.

### `907d8b0` (secondo commit test):
```
running 9 tests
test l3_4_decay_boundaries_saturating_sum_and_evaluation_tick ... FAILED
test l3_5_l3_6_encode_order_and_archive_suppression ... FAILED (encode restituisce [])
test l3_8_projection_is_deterministic_and_entries_are_ordered ... FAILED
test l3_7_recall_sequela_and_transition_errors ... FAILED
test l3_3_distinct_activation_events_strict_threshold_and_preconfig_history ... FAILED
test l3_9_evaluate_rejects_a_nonmatching_prefix ... FAILED
test l3_1_l3_2_config_is_replayed_and_requires_valid_t_h ... FAILED
test l3_9_cutoff_ignores_config_but_observes_decisions ... FAILED
test l3_pipeline_canonical_projection_through_recall_and_rearchive ... FAILED
```
→ **Tutti e 9 i test rossi.** Stub esteso con `canonical` che restituisce `String::new()`.

**Conclusione:** Test-first rispettato rigorosamente. Il commit di implementazione `3822396` non tocca alcun test.

---

## 7. Determinismo ricostruzione (L1 non richiesto per L3, ma verificato incidentalmente)

Il test `l3_pipeline_canonical_projection_through_recall_and_rearchive` e `l3_8_projection_is_deterministic_and_entries_are_ordered` verificano che `build_memory` su stesso log produca `MemoryView` byte-identici (`assert_eq!(first, again)` e `canonical` contiene `schema_version`). Il test `l1_view::ciclo_quattro_nodi_determinismo_due_esecuzioni_byte_per_byte` (L1) passa nella suite completa.

---

## 8. Violazioni d'isolamento

Nessuna. L'implementazione `src/memory.rs` usa solo:
- `crate::{view, EventType, Log, Value}` (API pubbliche)
- `crate::payload::write_escaped` (serializzazione canonica)
- `sha2::Sha256` (hash)
- Nessun accesso a campi privati, nessun I/O, nessun wall-clock, nessun RNG.

---

## 9. Sintesi esiti

| Voce | Esito |
|------|-------|
| Suite `cargo test` (114 passed) | **VERDE** |
| Suite `cargo test --release` (114 passed) | **VERDE** |
| L3.1 Eventi L3 | **CONFORME** |
| L3.2 Config | **CONFORME** |
| L3.3 Conteggio riattivazioni | **CONFORME** |
| L3.4 Decadimento | **CONFORME** |
| L3.5 PROMOTE | **CONFORME** |
| L3.6 ARCHIVE | **CONFORME** |
| L3.7 RECALL/sequela | **CONFORME** |
| L3.8 Proiezione canonica | **CONFORME** |
| L3.9 Cutoff/prefisso | **CONFORME** |
| L3.10 API | **CONFORME** |
| Copertura test per articolo | **COMPLETA** (9 test, 1 per articolo + pipeline) |
| Provenienza commit test-first rossi | **VERDE** (a337652, 907d8b0 verificati) |
| Provenienza commit implementazione solo src/memory.rs | **VERDE** (3822396) |
| Provenienza commit legge solo notes/ | **VERDE** (5f6e3d0, a46e9b2, 3b3482a) |
| Invarianze L0/L1/L2 | **VERDE** |
| Nessuna violazione isolamento | **VERDE** |

---

## 10. Domande / Punti da chiarire

1. **L3.7 test `l3_7_recall_sequela_and_transition_errors`** (riga 70-72): Il test appende un `archive` con `at:2` (seq dell'evento archive stesso) e poi verifica che `build_memory` fallisca con `Recall|Transition`. Nel replay (`src/memory.rs:93`) c'è un controllo che per `promote`/`archive` l'`at` debba corrispondere a un evento di valutazione precedente (`prior.seq==at && (nodes||edges)`). Per `recall` questo controllo **non** viene fatto (ramo `else` linea 102). Il test passa perché l'`archive` con `at:2` fallisce già al controllo `:93` (`Transition`) prima di arrivare al recall. **Domanda:** Il test intendeva verificare specificamente l'errore `Recall` per mancanza di sequela, o va bene che catturi `Transition` per `at` non valido sull'archive? La legge L3.7 richiede `Recall` per sequela mancante, ma qui l'archive è malformato. Suggerirei un test dedicato con archive valido + recall senza sequela.

2. **L3.6 ri-archiviazione soppressa:** Il test `l3_5_l3_6_encode_order_and_archive_suppression` verifica che gli Archive vengano emessi prima dei Promote (`for pair in decisions.windows(2)`), ma non testa esplicitamente la soppressione di un secondo Archive per la stessa identità. La logica è in `replay:101` (se già `Archived` → `Transition`). Coperto implicitamente dal replay autoritativo.

3. **L3.9 cutoff con config:** Il test `l3_9_cutoff_ignores_config_but_observes_decisions` aggiunge un config a seq 2 e verifica che `evaluate` non fallisca. Conferma che config non resetta cutoff. Ben fatto.

4. **Warning `unused import: EventType` in `src/selection.rs:3`**: Preesistente, non introdotto da L3. Registrato come da istruzioni.

---

**Report completo. Nessun codice modificato. Solo lettura ed esecuzione.**  
**Artefatto:** `reports/L3_test_report.md`