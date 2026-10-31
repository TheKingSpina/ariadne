modello: opencode/nemotron-3.5-super-vl-free

Memorie Mnemosyne esaminate:
- mem_f1b05e04-85bc-4a91-b506-3a2f7ec8293a (decision, v2): emendamenti E1/E2 legge L3, commit aed10e8, 31/10/2026.
- mem_8c87fe0c-133d-4715-9681-4dbcae9f6b0b (decision, v2): legge L3 memoria lunga, commit 5f6e3d0, 30/10/2026.
- mem_ff84d6c9-caf5-4a4f-9866-f33379e2d042 (episode, v2): audit L3 NON CONFORME 30/10/2026, quattro rilievi.

Sessione Mnemosyne: ses_a616eb1d-d5ed-4a54-8472-0cf8f2992b94 — aperta con memory_open_session; memory_admin_overview: 5 memorie, 5 accettate, 0 pendingApproval, 0 rejected, 0 retracted, 0 conflitti; memory_list_conflicts: vuoto; memory_pending_review: vuoto; memory_context (query L3/E1-E2 e L3.3/E2) ha restituito le 3 memorie sopra, confrontate col codice (nessun CONFLITTO MEMORIA: codice coerente con le decisioni approvate); sessione chiusa con memory_close_session.

# Re-audit Reviewer L3 su HEAD `4f353239` — repo /Users/spina/Ariadne

## Precondizione (gate tester)
Report tester: `reports/L3_test_report_4f353239.md`. Suite completa: **119/119 passati, 0 falliti** in debug e release; 14 test L3 in `tests/l3_memory.rs`. Checklist tester tutta verde → l'audit finale di implementazione può partire (ciclo di audit rispettato).

## Limite metodologico (riportato verbatim)
I comandi `git show --stat ...` e `git rev-parse HEAD && git status --short` sono stati negati dalla shell con errore:
```
{"error":{"type":"permission.rejected","message":"Permission denied: shell"},"content":[]}
```
`git log --oneline -20` è invece riuscito e mostra HEAD in `4f353239` "fix: enforce L3 E1 recall ordering and E2 archive tracking", seguito da `0defe20` (agenti), `0f01211` (test), `aed10e8` (legge E1/E2). Per la provenienza di commit e diff mi affido quindi alle prove fornite dal tester (orchestratore), riportate con relativa citazione; non ho potuto rieseguirle autonomamente in shell. Nessun file sotto `src/`, `tests/`, `notes/`, `spec/` è stato modificato da me (sola lettura); l'unico file scrivibile è questo report, non committato.

---

## Punto 1 — I tre difetti dell'audit 30/10 sono chiusi nel codice a `4f353239`

### 1a. Ramo recall: `at` deve essere < seq del recall; errore = `L3Error::Recall` (E1, L3.7c)
**Codice**: `src/memory.rs:94`:
```rust
if tag=="recall" && at>=e.seq {return Err(L3Error::Recall)}
```
Il check esiste solo nel ramo `recall` (per promote/archive resta il controllo di evento precedente al `src/memory.rs:93`, invariato, non regredito). Il nuovo check impone `at < e.seq` esattamente come l'emendamento E1 ("at.seq < seq dell'evento recall"). Violazione → `L3Error::Recall`.
**Test di copertura**: `tests/l3_memory.rs:62-75` `l3_e1_recall_cannot_reference_future_activation`: costruisce `valid_archived_a()` (A già archiviata), calcola `recall_seq = last.seq + 1`, appende l'evento RECALL **prima** dell'evento attivante con `at = recall_seq + 1` (futuro, inesistente al momento del recall), poi appende l'attivazione di A; `build_memory` restituisce `L3Error::Recall`. Corrisponde esattamente al difetto 1 del primo audit (recall con sequela futura).
**Esito punto 1a: CONFORME** (`src/memory.rs:94`; `tests/l3_memory.rs:62-75`).

### 1b. Preimage `without_archive_hash` vs `canonical`: byte per byte, anche con `archived_at_seq` (L3.8/L3.10)
**Codice**:
- `src/memory.rs:117-121` `without_archive_hash`: serializza `{"entries":[...],"head_hash":...,"schema_version":"1"}`; dentro ogni voce `archived_at_seq` (se presente) è emesso **prima** di `id`, poi `origin_seq`, `status`.
- `src/memory.rs:123-127` `canonical`: stesso ordine di membri (`archive_hash` primo, poi `entries`, `head_hash`, `schema_version` — ordine lessicografico per L3.8) e stesso ordine dentro ogni voce (`archived_at_seq` prima di `id`).
- `src/memory.rs:130` `build_memory`: `v.archive_hash = sha(without_archive_hash(&v).as_bytes())` — l'hash è calcolato sul preimage privo del campo `archive_hash`, coerente con `canonical` meno quel campo, byte per byte, incluse le voci con `archived_at_seq` (presente solo se `status == Archived`, L3.8).
**Test di copertura**: `tests/l3_memory.rs:79-89` `l3_8_digest_matches_canonical_archived_document_preimage`: prende `memory::canonical(&view)`, toglie il prefisso `{"archive_hash":"..."}`, calcola `Sha256::digest` **direttamente con la crate `sha2`** (non con `memory::sha`, quindi non è confronto autoreferenziale) sul preimage e confronta con `view.archive_hash`. Esce su `valid_archived_a()` (voce archiviata con `archived_at_seq`).
**Esito punto 1b: CONFORME** (`src/memory.rs:117-130`; `tests/l3_memory.rs:79-89`).

### 1c. Candidati Archive al giudizio richiedono ≥1 attivazione con `seq ≤ eval seq` (E2, L3.3)
**Codice**: `src/memory.rs:145`:
```rust
let mut archives:Vec<String>=entries.iter().filter(|(id,x)| x.status!=Status::Archived
    && acts.get(*id).is_some_and(|xs| xs.iter().any(|(seq,_)| *seq<=e.seq))
    && score(id,e.seq,e.tick,h,&acts)==0).map(|(id,_)|id.clone()).collect();
```
Il filtro `acts.get(*id).is_some_and(...)` esclude le identità senza attivazioni `≤ e.seq`: un'identità mai attivata al giudizio è fuori traccia e non riceve `Archive`. Stessa logica E2 per le Promote a `src/memory.rs:147` (conta solo attivazioni `*s<=e.seq`, soglia stretta `> t`). `entries` deriva da `acts` (`src/memory.rs:82`), quindi nessuna decisione può riguardare identità mai attivate.
**Test di copertura**: `tests/l3_memory.rs:40-58` `l3_e2_future_identity_is_not_archived_before_first_activation`: attiva C al giudizio (seq 26), calcola `x_activation_seq` della prima attivazione di X (seq 27, successiva), chiama `evaluate` sul log completo e asserisce che ogni decisione su `X` abbia `at >= x_activation_seq` (quindi nessuna decisione prima della prima attivazione) e che esista un `Promote` di X con `at >= x_activation_seq` dopo l'attivazione. Corrisponde esattamente al difetto 3 del primo audit (identità future archiviate prima dell'esistenza).
**Esito punto 1c: CONFORME** (`src/memory.rs:145`, `147`; `tests/l3_memory.rs:40-58`).

---

## Punto 2 — I test corrispondono davvero ai difetti (`tests/l3_memory.rs`)

- **E1** (`tests/l3_memory.rs:62-75`): il RECALL è appendato **prima** dell'evento attivante (`action` recall a riga 67-71, `nodes ... ["A"]` a riga 72); `at = recall_seq + 1` > seq del recall; `build_memory` restituisce `L3Error::Recall` (assert a riga 73). ✓ Difetto 1 riprodotto e sanzionato.
- **E2** (`tests/l3_memory.rs:40-58`): la valutazione (evento con `nodes ["C"]`, riga 45) precede la **prima** attivazione di X (riga 49); X non riceve decisioni prima di tale attivazione (assert riga 53-56) e ottiene `Promote` solo dopo (assert riga 57). ✓ Difetto 3 riprodotto e sanzionato.
- **Digest** (`tests/l3_memory.rs:79-89`): SHA-256 calcolato autenticamente tramite `Sha256::digest` (sha2 crate, riga 87) sul preimage ottenuto da `canonical` meno il campo `archive_hash`; confronto con `view.archive_hash` memorizzato. Non è confronto autoreferenziale e fallirebbe se l'ordine dei campi divergesse. ✓ Difetto 2 riprodotto e sanzionato.
- **Soppressione re-ARCHIVE** (`tests/l3_memory.rs:93-101`): asserzione che, dopo `Archive` in log e una nuova attivazione della stessa identità, `evaluate` non emetta un nuovo `Decision3::Archive` per quell'`id` (riga 99). ✓ L3.6.

---

## Punto 3 — Correlazione articoli L3.1–L3.10 + E1/E2 ↔ test (`tests/l3_memory.rs`)

| Articolo | Test che lo dimostra (file:riga) | Esito copertura |
|----------|----------------------------------|-----------------|
| L3.1 eventi L3 (tag chiuso, no l2+l3, nessun tipo L0 nuovo) | `l3_1_l3_2_config_is_replayed_and_requires_valid_t_h:244-255`; `l3_5_l3_6_encode_order_and_archive_suppression:136-144` (encode) | Coperto |
| L3.2 config T/H, replay richiede config | `l3_1_l3_2_config_is_replayed_and_requires_valid_t_h:244-255` | Coperto |
| L3.3 conteggio, soglia >T, persistenza, E2 tracciamento | `l3_3_distinct_activation_events_strict_threshold_and_preconfig_history:105-115`; `l3_e2:40-58` | Coperto |
| L3.4 decadimento, saturante, tick di valutazione | `l3_4_decay_boundaries_saturating_sum_and_evaluation_tick:120-131` | Coperto |
| L3.5 Promote, ordine d'ingresso | `l3_5_l3_6_encode_order_and_archive_suppression:136-144` | Coperto |
| L3.6 Archive, archived_at_seq, soppressione re-archive | `l3_6_archive_in_log_is_not_reemitted:93-101`; `l3_5_l3_6_encode_order_and_archive_suppression:136-144` | Coperto |
| L3.7 Recall, sequela, transizioni, origin_seq conservato | `l3_7_recall_sequela_and_transition_errors:149-155`; `l3_7b_recall_without_sequela_with_valid_archive:160-221`; `l3_e1:62-75` (E1) | Coperto |
| L3.8 stato, proiezione canonica, archive_hash preimage | `l3_8_projection_is_deterministic_and_entries_are_ordered:258-265`; `l3_8_digest_matches_canonical_archived_document_preimage:79-89` | Coperto |
| L3.9 cutoff, config controllo, HeadMatch | `l3_9_cutoff_ignores_config_but_observes_decisions:226-231`; `l3_9_evaluate_rejects_a_nonmatching_prefix:268-273` | Coperto |
| L3.10 API (`MemoryView`, `canonical`, `build_memory`, `evaluate`, errori, encode) | `l3_pipeline_canonical_projection_through_recall_and_rearchive:236-241`; `l3_8_digest...:79-89`; codice `src/memory.rs:10-18,122-131,134-155` | Coperto |
| **E1** (sequela recall nel log precedente) | `tests/l3_memory.rs:62-75` | Coperto (nuovo, commit `0f01211`) |
| **E2** (candidati giudizio con attivazione ≤ t) | `tests/l3_memory.rs:40-58` | Coperto (nuovo, commit `0f01211`) |

**Lacune residue**: nessuna. Ogni articolo L3.1–L3.10 ha almeno un test dedicato (ereditato dal primo audit) e gli emendamenti E1/E2 hanno test dedicati aggiunti in `0f01211`. Il test `l3_7b:160-221` rafforza inoltre L3.7 (recall senza sequela valida → `Recall`, non `Transition`), sanando la domanda lasciata aperta dal primo audit (§10, punto 1 di `reports/L3_test_report.md`).

---

## Punto 4 — Provenienza commit (prove orchestratore-provviste; shell `git show`/`git diff` negata)

| Commit | Messaggio (da `git log`) | File toccati (per report tester `L3_test_report_4f353239.md`, §5) | Natura |
|--------|--------------------------|-------------------------------------------------------------------|--------|
| `aed10e8` | L3 legge, emendamenti E1/E2 firmati dall'umano (31/10/2026, in esito all'audit 30/10) | `notes/L3_legge_memoria.md` | Solo legge |
| `0f01211` | test: cover L3.7c E1, L3.3 E2, L3.8 preimage, L3.6 suppression | `tests/l3_memory.rs` (+82) — 4 test nuovi: `l3_e2...`, `l3_e1...`, `l3_8_digest...`, `l3_6_archive...`; nessun test esistente modificato | Solo test |
| `4f353239` | fix: enforce L3 E1 recall ordering and E2 archive tracking | `src/memory.rs` (+3/−2) — riga 94 (E1), righe 117-121/123-127 (preimage), riga 145 (E2); nessun test toccato | Solo fix |

Per il range `aed10e8..4f353239`, il report tester (§6) dichiara: `git diff aed10e8..4f353239 -- src/view.rs src/selection.rs src/log.rs src/event.rs src/payload.rs src/lib.rs spec/` → nessun output (zero modifiche). Non ho potuto rieseguire `git diff` in shell (errore riportato sopra); l'evidenza è quella fornita dal tester, coerente con i file letti direttamente a `4f353239` (solo `src/memory.rs` contiene i tre fix; i file L0/L1/L2 non contengono riferimenti L3).
**Esito punto 4: CONFORME** (con caveat metodologico: prove da report tester, shell negata).

---

## Punto 5 — Messaggi di commit

- `4f353239`: "fix: enforce L3 E1 recall ordering and E2 archive tracking" — cita gli emendamenti E1/E2. ✓
- `aed10e8`: "L3 legge, emendamenti E1/E2 firmati dall'umano (31/10/2026, in esito all'audit gpt-6.1-terra del 30/10)" — cita E1/E2 e l'audit che li ha mossi. ✓
**Esito punto 5: CONFORME**.

---

## Punto 6 — Invarianze L0/L1/L2 e suite

- `src/view.rs`, `src/selection.rs`, `src/log.rs`, `src/event.rs`, `src/payload.rs`, `src/lib.rs`, `spec/`: non toccati dal lavoro L3 (report tester §5-6; primo audit §5 invariante ancora valido a `4f353239`). Import di `src/memory.rs` invariati (riga 1-7: solo `view, EventType, Log, Value`, `payload::write_escaped`, `sha2`; zero I/O, wall-clock, RNG).
- Suite completa: **119/119 debug e 119/119 release** (report tester §1), inclusi 14 test L3 tutti verdi.
**Esito punto 6: CONFORME**.

---

## Punto 7 — Registro primo audit (30/10) come risolto

| Rilievo primo audit | Stato a `4f353239` | Prova |
|---------------------|---------------------|-------|
| (1) Recall accetta `at` futuro/autoreferenziale | **CHIUSO** | `src/memory.rs:94` (`at>=e.seq → L3Error::Recall`); `tests/l3_memory.rs:62-75` (recall appendato prima dell'attivante) |
| (2) Preimage `archive_hash` incoerente col canonico per voci archiviate | **CHIUSO** | `src/memory.rs:117-130` (`without_archive_hash` == `canonical` meno campo, `archived_at_seq` primo in entrambe, righe 119 e 125); `tests/l3_memory.rs:79-89` (SHA-256 vero) |
| (3) Identità future archiviate prima dell'esistenza | **CHIUSO** | `src/memory.rs:145` (filtro E2 `acts.get(...).is_some_and(... seq <= e.seq)`); `tests/l3_memory.rs:40-58` (valutazione prima della prima attivazione di X) |
| (4) Provenienza `a337652`: mancata citazione riga export `pub mod memory` nel messaggio | **APERTA (NOTA)** | Arbitrato dal pattern L2 consuetudinario; resta NOTA per mancata citazione (memoria `mem_f1b05e04-...`). Decisione umana richiesta — vedi rilievi residui. |

**Conflitti memoria Mnemosyne**: nessuno rilevato; le 3 memorie richiamate sono coerenti con il codice a `4f353239` (nessun CONFLITTO MEMORIA).

---

## Giudizio finale

**AUDIT CONFORME**

Tutti i vincoli della legge L3 (L3.1–L3.10 + emendamenti E1/E2) sono implementati a `HEAD 4f353239` con citazioni file:riga; i tre difetti dell'audit 30/10 sono chiusi da codice e da test dedicati (commit `0f01211` solo test, `4f353239` solo `src/memory.rs`); suite 119/119 verde in debug e release; invarianze L0/L1/L2 rispettate; nessuna memoria Mnemosyne in conflitto o in attesa di review.

---

## Rilievi residui che richiedono decisione umana

1. **NOTA (4) del primo audit — commit `a337652`**: il commit test-first non cita la riga di export `pub mod memory` nel messaggio; pattern L2 consuetudinario accettato, NOTA confermata (memoria `mem_f1b05e04-85bc-4a91-b506-3a2f7ec8293a`). Nessun blocco al giudizio CONFORME; spetta all'umano decidere se integrare il messaggio di commit o archiviare la nota.
2. **Limite metodologico**: `git show --stat`, `git diff*`, `git status` negati dalla shell (errore verbatim riportato in testa); provenienza commit e diff desunta dal report tester dell'orchestratore, non rieseguita autonomamente.

---
*Firme: nessuna accettazione di livello e nessun push da parte del Reviewer; l'accettazione resta in capo all'umano. Report in sola lettura su `reports/`, non committato.*
