# L1 Test Report — Gate on HEAD `87bc9e9`

**Modello**: `opencode/nemotron-3-ultra-free`  
**Memorie Mnemosyne**: servizio non raggiungibile (timeout su `memory_open_session`) — nessuna memoria consultata né proposta.

---

## 1. Esecuzione della suite su worktree pulito

Worktree creato: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l1-test` (detached HEAD `87bc9e9`).  
`git status` nel worktree: pulito.

### 1.1 `cargo test --test l1_view`

```
running 11 tests
test genesi_sola_vista_vuota_ancorata_head_hash_d21 ... ok
test nodo_golden_id_origin_seq_unico_evento ... ok
test eventi_senza_estrazioni_zero_e_array_vuoto ... ok
test endpoint_non_dichiarato_errore_no_skip_no_autocreazione ... ok
test nodi_prima_degli_archi_nello_stesso_evento_d19 ... ok
test builder_non_riverifica_catena_d20 ... ok
test stesso_key_kind_diversi_nodi_distinti_e_arco_cross_kind ... ok
test zero_scritture_digest_log_intatto ... ok
test arco_unicita_per_trip_e_origin_seq_minimo_d18 ... ok
test payload_estratti_non_validi_errore ... ok
test ciclo_quattro_nodi_determinismo_due_esecuzioni_byte_per_byte ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**VERDE** — Tutti e 11 i test L1 passano.

### 1.2 `cargo test` (suite completa L0 + L1)

```
running 4 tests (unit) ... ok
running 0 tests (l0_replay) ... ok
running 2 tests (l0_c1_genesis) ... ok
running 11 tests (l0_cycle) ... ok
running 18 tests (l0_event_log) ... ok
running 13 tests (l0_n3_canonical_string) ... ok
running 10 tests (l0_n3_line_canonical) ... ok
running 7 tests (l0_prng) ... ok
running 6 tests (l0_replay_process) ... ok
running 9 tests (l0_state) ... ok
running 11 tests (l1_view) ... ok

test result: ok. 91 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**VERDE** — Suite completa (80 test L0 ereditati + 11 L1) passa senza regressioni.

---

## 2. Verifica commit test-first (c8c69d7)

**Prova ricostruita dal commit**:

- `git show c8c69d7 --name-only` → unico file: `tests/l1_view.rs`
- `git show c8c69d7:tests/l1_view.rs` → 514 righe di test con golden indipendenti calcolati via `shasum -a 256` fuori dal crate
- Messaggio del commit: «Suite ROSSA verificata: cargo test --test l1_view fallisce con error[E0432]: unresolved import ariadne::view (l'API non esiste ancora).»
- Decisioni D14–D21 pinnate nel messaggio e nei commenti del test

**VERDE** — Il primo commit del ciclo contiene **solo test**, l'API `ariadne::view` non esisteva, i test erano rossi per import irrisolto.

---

## 3. Checklist SPEC L1 (§6) + decisioni operative D14–D21

| # | Criterio | Prova (file:riga / output) | Esito |
|---|----------|----------------------------|-------|
| 1 | Ricostruzione deterministica su due esecuzioni indipendenti | `ciclo_quattro_nodi_determinismo_due_esecuzioni_byte_per_byte` (tests/l1_view.rs:243–280): `assert_eq!(v1.canonical(), v2.canonical())` + replay `from_jsonl(to_jsonl())` byte-identico | **VERDE** |
| 2 | Ogni arco tracciato alla sua seq di origine | `arco_unicita_per_trip_e_origin_seq_minimo_d18` (tests/l1_view.rs:144–179): `origin_seq` minimo per edge riprovato; `assert_view` controlla `origin_seq` per ogni arco; implementazione `edges: BTreeMap<(src_id,dst_id,rel), u64>` con `min_seq` (src/view.rs:222, 254–256) | **VERDE** |
| 3 | Vista ancorata all'hash di testa del log | `genesi_sola_vista_vuota_ancorata_head_hash_d21` (tests/l1_view.rs:102–115): `assert_eq!(view.head_hash, GENESIS_1042_HASH)`; `build` prende `log.events().last().unwrap().hash` (src/view.rs:212–217); `head_hash` = hash evento finale per ogni test | **VERDE** |
| 4 | Zero scritture al log dimostrate | `zero_scritture_digest_log_intatto` (tests/l1_view.rs:472–488): `sha256_hex(log.to_jsonl())` prima/dopo `build` identico; `log.events().len()` invariato; implementazione `build(&Log)` solo lettura (src/view.rs:211) | **VERDE** |
| 5 | Test con grafo ciclico valido superato | `ciclo_quattro_nodi_determinismo_due_esecuzioni_byte_per_byte`: nodi A,B,C,D + archi A→B, B→C, C→A, A→D, D→C; `find_edge` conferma C→A presente; nessun rifiuto | **VERDE** |
| D14 | `head_hash` = hash ultimo evento (genesi inclusa) | Test `genesi_sola_vista_vuota_ancorata_head_hash_d21` + codice src/view.rs:212–217 | **VERDE** |
| D15 | `schema_version` = stringa `"1"` | Tutti i test via `assert_view` → `assert_eq!(view.schema_version, "1")`; costruttore `schema_version: "1".to_string()` (src/view.rs:284) | **VERDE** |
| D16 | `view_hash` = SHA-256 lowercase del documento canonico escluso `view_hash`; nessun separatore 0x1F | Golden esterno: `GENESIS_ONLY_VIEW_HASH` = `60e804ca9c2d481f391e872674274d9492667be663be238897cc517caf8dc507` (tests/l1_view.rs:38, commit c8c69d7); `assert_view` ricalcola indipendente `minus_hash` e confronta (tests/l1_view.rs:84–100); implementazione `minus_view_hash` taglio per lunghezza valore attuale non costante (src/view.rs:140–146) | **VERDE** |
| D17 | Nodo `{id,key,kind,origin_seq}`; arco `{dst_id,origin_seq,rel,src_id}`; ordinamento nodi per id, archi per `(src_id,dst_id,rel)` | Struct `Node`/`Edge` (src/view.rs:51–68); serializzazione `canonical` ordine campi esatto (src/view.rs:96–104, 113–121); `out_nodes.sort_by(|a,b| a.id.cmp(&b.id))` (src/view.rs:271); `out_edges.sort_by(|a,b| (src_id,dst_id,rel))` (src/view.rs:281) | **VERDE** |
| D18 | Arco unico per `(src_id,dst_id,rel)` con `origin_seq` minimo | `BTreeMap<(String,String,String), u64>` key = trip identità; `and_modify(|s| *s = min_seq(*s, seq))` (src/view.rs:222, 254–256); test `arco_unicita_per_trip_e_origin_seq_minimo_d18` verifica riprova e `rel` diversa = arco distinto | **VERDE** |
| D19 | Nello stesso evento, `nodes` elaborato per intero prima di `edges` | Codice: blocco `if let Some(v) = e.payload.get("nodes")` **prima** di `if let Some(v) = e.payload.get("edges")` (src/view.rs:228–258); test `nodi_prima_degli_archi_nello_stesso_evento_d19` (tests/l1_view.rs:283–292) | **VERDE** |
| D20 | Builder NON rivérifica catena L0 (log manomesso → build ok) | Test `builder_non_riverifica_catena_d20` (tests/l1_view.rs:428–457): hash manomessi a `'a'*64`, `Log::from_jsonl` accetta (schema valido), `view::build` produce vista ancorata agli hash registrati | **VERDE** |
| D21 | Log "vuoto" = genesi-only: nodes/edges vuoti, `head_hash` genesi, `view_hash` normale | Test `genesi_sola_vista_vuota_ancorata_head_hash_d21`: `Log::genesis(1042)` → `nodes/edges []`, `head_hash = GENESIS_1042_HASH`, `view_hash = GENESIS_ONLY_VIEW_HASH` golden | **VERDE** |

---

## 4. Isolamento — input non dichiarati

- `build` firma: `pub fn build(log: &Log) -> Result<View, Error>` (src/view.rs:211) — solo `&Log`
- Nessun accesso a filesystem, rete, RNG, wall-clock
- Payload L0 invariato: valori top-level solo `i64`/`String`; `nodes`/`edges` restano stringhe con array canonico interno (src/view.rs:8–17, parser lines 329–381)
- Parser rifiuta: tipo sbagliato (`Value::Int` per `nodes`), array non canonico, campi extra/mancanti, tipi interni errati, `rel` fuori `[a-z0-9_]+`, ordine chiavi non canonico, spazi interni (tests `payload_estratti_non_validi_errore` — 9 sottocasi, tutti `Error::Schema`)

**VERDE** — Nessuna violazione d'isolamento rilevata.

---

## 5. Provenienza e immutabilità (SPEC L1 + eredità L0)

- **Provenienza nodi/archi `seq`**: `nodes` map `(kind,key) → min_seq`; `edges` map trip → min_seq (src/view.rs:220–222, 254–256); test `arco_unicita_per_trip_e_origin_seq_minimo_d18` + `nodo_golden_id_origin_seq_unico_evento` verificano `origin_seq` minimi
- **Zero scritture**: digest `to_jsonl()` prima/dopo identico (test `zero_scritture_digest_log_intatto` + codice `build(&Log)` solo lettura)
- **Immutabilità eventi L0**: ereditata da L0 (test `test_immutable_events_slice_borrows`, `test_log_is_append_only_no_mutation_truncation_or_reorder` in suite L0 — tutti verdi); L1 non tocca il log
- **Schema `seq` L0**: ereditato; L1 usa `event.seq` come ordinamento di replay (src/view.rs:224)

**VERDE** — Requisiti L1 soddisfatti; proprietà L0 ereditate non richiedono nuovi test L1.

---

## 6. Determinismo replay vista (checklist SPEC §5)

- Test `ciclo_quattro_nodi_determinismo_due_esecuzioni_byte_per_byte`: due `Log` costruiti identicamente → `v1.canonical() == v2.canonical()` (byte-per-byte)
- Replay dal byte: `Log::from_jsonl(&log1.to_jsonl())` → vista identica a `v1.canonical()`
- Golden esterni ricalcolati indipendentemente con `shasum -a 256` (commit c8c69d7): `NODE_A_LETTER_ID`, `NODE_B_LETTER_ID`, `GENESIS_ONLY_VIEW_HASH` — confrontati byte-per-byte nei test

**VERDE** — Determinismo forte verificato.

---

## 7. Sintesi esiti

| Voce | Esito |
|------|-------|
| Suite L1 (11 test) | **VERDE** |
| Suite completa (91 test) | **VERDE** |
| Commit test-first (c8c69d7) | **VERDE** |
| Checklist SPEC §6 (5 voci) | **VERDE** (5/5) |
| Decisioni D14–D21 (8 voci) | **VERDE** (8/8) |
| Isolamento input | **VERDE** |
| Provenienza/immutabilità | **VERDE** |
| Determinismo replay | **VERDE** |

**Nessun ROSSO. Nessuna NOTA bloccante.**  
Il gate L1 su `87bc9e9` è **superato**.

---

## 8. Artefatti di riferimento

- Worktree pulito: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l1-test`
- Commit test-only: `c8c69d7` (`tests/l1_view.rs` — 514 righe, golden `shasum` verbatim)
- Commit implementazione: `87bc9e9` (`src/view.rs` 709 righe, `src/lib.rs` +1 export)
- Spec: `/Users/spina/Ariadne/spec/L1_SPEC_v0.1.md`