# L2 Delta Pipeline Gate Report

**Modello:** opencode/nemotron-3-ultra-free
**HEAD verificato:** 78ba3077342d47b4ceafb32f953f1db6a9f63987 (fix: honor L2.25 window summaries and L2.26 identity)
**Worktree:** `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/ariadne_l2_delta` (pulito, detached HEAD)
**Mnemosyne:** Server non connesso (`MCP server "mnemosyne" is not connected or does not expose resources`) — nessuna memoria richiamata né proposta.

---

## 1. Suite completa `cargo test` — TUTTI VERDI

Esecuzione su worktree pulito di `78ba307`. Output integrale riportato sopra.

| Test suite | Test passati |
|------------|--------------|
| `src/lib.rs` (unit) | 4 |
| `src/bin/l0_replay.rs` | 0 |
| `tests/l0_c1_genesis.rs` | 2 |
| `tests/l0_cycle.rs` | 11 |
| `tests/l0_event_log.rs` | 18 |
| `tests/l0_l1_l2_pipeline.rs` | **1** (integrazione L0-L1-L2) |
| `tests/l0_n3_canonical_string.rs` | 13 |
| `tests/l0_n3_line_canonical.rs` | 10 |
| `tests/l0_prng.rs` | 7 |
| `tests/l0_replay_process.rs` | 6 |
| `tests/l0_state.rs` | 9 |
| `tests/l1_view.rs` | 11 |
| `tests/l2_selection.rs` | **13** (L2 unit) |
| Doc-tests | 0 |
| **TOTALE** | **105** |

**L2-specific tests:** 13 (`l2_selection.rs`) + 1 (`l0_l1_l2_pipeline.rs`) = **14 test L2**, tutti verdi.

---

## 2. Analisi copertura `tests/l0_l1_l2_pipeline.rs`

File: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/ariadne_l2_delta/tests/l0_l1_l2_pipeline.rs` (53 righe, 1 test: `l0_l1_l2_pipeline_is_replayable_and_provenanced`)

| Requisito (da dispatch) | Copertura | Evidenza `file:riga` | Esito |
|------------------------|-----------|----------------------|-------|
| Replay autorevole sull'analogo (`build_active` deterministico) | SÌ | `l0_l1_l2_pipeline.rs:45-47` — doppio `build_active` e `assert_eq!(active_a, active_b)` | 🟢 VERDE |
| `build_active` chiamato | SÌ | `l0_l1_l2_pipeline.rs:19`, `45`, `46` | 🟢 VERDE |
| `evaluate` chiamato | SÌ | `l0_l1_l2_pipeline.rs:36`, `41` | 🟢 VERDE |
| `encode` chiamato | SÌ | Implicito: `selection::encode` usato in `l2_selection.rs:130` ma **NON in `l0_l1_l2_pipeline.rs`**. Il test pipeline non chiama `encode` direttamente. | 🟡 NOTA — `encode` testato in `l2_selection.rs:25` e `130`, non nel pipeline |
| Sommari finestra con `expulsions`/`text` (L2.25/L2.26) | SÌ | `l0_l1_l2_pipeline.rs:37-40` — `Decision::Summary { expulsions, .. }` con `expulsions == &vec![2]` | 🟢 VERDE |
| Summary prima del Select di chiusura implicita | SÌ | `l0_l1_l2_pipeline.rs:37-39` — `summary_at < close_select_at` | 🟢 VERDE |
| Digest `to_jsonl` invariato (zero scritture) | SÌ | `l0_l1_l2_pipeline.rs:20-22` (`before` vs `log.to_jsonl()`), `35-43` (`before_eval` vs `log.to_jsonl()`) | 🟢 VERDE |
| Doppio `build_active` deterministico | SÌ | `l0_l1_l2_pipeline.rs:45-47` | 🟢 VERDE |
| `head_hash` = ultimo evento | SÌ | `l0_l1_l2_pipeline.rs:23` — `assert_eq!(v.head_hash, log.events().last().unwrap().hash)` | 🟢 VERDE |
| `nodes` prima di `edges` (D19) | SÌ | Testato in `l1_view.rs:44` (`nodi_prima_degli_archi_nello_stesso_evento_d19`), non nel pipeline. Il pipeline crea nodi ed archi nello stesso evento (`l0_l1_l2_pipeline.rs:18`, `34`) ma non asserisce l'ordine. | 🟡 NOTA — coperto in `l1_view.rs`, non nel pipeline |
| `origin_seq` minimi (D18) | SÌ | `l0_l1_l2_pipeline.rs:27-29` — `origin_seq` verificati per nodi e archi; testato in `l1_view.rs:54` (`arco_unicita_per_trip_e_origin_seq_minimo_d18`) | 🟢 VERDE (via L1) |
| Cicli (L1) | SÌ | `l0_l1_l2_pipeline.rs:34` crea arco `rel="cycle"`; `l1_view.rs:66` (`ciclo_quattro_nodi_determinismo_due_esecuzioni_byte_per_byte`) | 🟢 VERDE (via L1) |
| Arm `ErrorCode`/echelons L2.21 | SÌ | `l2_selection.rs:72-76` (`config_replacement_and_l2_21_schema_rules`), `99-100` (`replay_is_deterministic...`), non nel pipeline. | 🟡 NOTA — coperto in `l2_selection.rs`, non nel pipeline |

**Sintesi copertura pipeline:** 9/12 voci direttamente nel pipeline (🟢), 3 coperte in altre suite (🟡 NOTA). Nessuna voce ROSSA.

---

## 3. Legge aggiornata: `notes/L2_legge_selezione.md` — L2.25 e L2.26 verbatim

### L2.25 (commit `716e971`) — *Semantica della finestra offline*
> **L2.25 — Semantica della finestra offline (24/10/2026, 14ª domanda, risoluzione della contraddizione L2.8/L2.23).**
> La finestra offline contiene **SOLO eventi `l2`**. Dentro una finestra aperta:
> - sono ammessi solo eventi `l2` (`select`, `summary` di window precedente, `barrier`) e **nessun** evento L0 non-l2;
> - il primo evento non-`l2` dopo l'apertura chiude la finestra **ed è allo stesso tempo evento di valutazione**: emette il `Summary` della finestra (se ha expulsions) e POI il proprio `Select`, se genera transizioni (L2.15/L2.19);
> - le `expulsions` del Summary della finestra sono le espulsioni **accumulate dai `Select` già apposti nella finestra** (eventi l2 che il chiamante ha scritto tra l'apertura e la chiusura, autorevoli per L2.11/M2.14); le espulsioni generate dalla valutazione dell'evento di chiusura NON stanno in quel Summary: appaiono nel `Select` dello stesso evento di chiusura;
> - `phase="close"` chiude la finestra nello stesso evento (L2.19), senza selezioni corrispondenti se è puro controllo;
> - le valutazioni **NON continuano dentro una finestra aperta**: la clausola di L2.23 «Le valutazioni continuano dentro una finestra aperta» è **ABROGATA** da questa L2.25.

**Conformità codice:** `src/selection.rs:35-39` gestisce la chiusura implicita (primo non-l2) emettendo Summary se `!window_has_summary && !expulsions.is_empty() && e.seq>cutoff`; `src/selection.rs:23-30` processa `select` dentro finestra (barrier=true) accumulando `expulsions`/`expelled_ids`; `src/selection.rs:33` blocco `barrier` con `phase="close"` emette Summary se `!window_has_summary && !expulsions.is_empty()`. **Coerente verbatim.**

### L2.26 (commit `7090ed2`) — *Identità degli espulsi quando origin_seq si ripete*
> **L2.26 — Identità degli espulsi quando origin_seq si ripete (24/10/2026, 15ª domanda).**
> `select` porta `ups`/`downs` come array di origin_seq (L2.10, invariato). Se una medesima origin_seq in `downs`/`ups` corrisponde a più elementi L1 con lo stesso origin_seq nello stesso evento, l'identità esatta (id) che il Summary riporta in `text` è dedotta dal replay applicando lo **stesso ordine di decisione della legge** (L2.6 → L2.17 → L2.24) allo stato attivo immediatamente precedente all'evento select: sono espulsi, in ordine, gli elementi che il tie-break segnala finché il conteggio dei downs per quella origin_seq è soddisfatto. `expulsions` resta array di origin_seq (L2.16); `text` riporta gli id così dedotti, dedup, ordine lessicografico asc, separati da `|`. La deduzione è deterministica e non muta `build_active`: il set attivo resta autorevole dai select (L2.11).

**Conformità codice:** `src/selection.rs:23-30` — nel blocco `select` con `barrier=true`, il tie-break `ranked.sort_by(|a,b| score(a).cmp(&score(b)).then_with(|| origin_map[b].cmp(&origin_map[a])).then_with(|| b.cmp(a)))` implementa esattamente L2.6 (score) → L2.17 (origin_seq minore = più nuovo = resta) → L2.24 (id lessicografico minore = resta). Gli espulsi vengono inseriti in `expulsions` (origin_seq) e `expelled_ids` (id L1). Alla chiusura finestra (`barrier` phase close o chiusura implicita `src/selection.rs:35-39`), `ids_sorted` viene ordinato lessicograficamente (`ids_sorted.sort()`) e joinato con `|`. **Coerente verbatim.**

**Test di verifica L2.26:** `l2_selection.rs:141-158` (`equal_score_eviction_prefers_newer_origin_then_greater_id`) — crea due nodi stessa `origin_seq=2`, apre finestra, `select` con `downs=[2]`, chiude con `phase="close"`, verifica che `text` contenga l'id **maggiore** (lessicograficamente) perché il tie-break L2.24 fa restare il minore. Riga 156-157: `let greater = view.nodes.iter().filter(|n| n.origin_seq==2).map(|n| n.id.clone()).max().unwrap(); assert_eq!(text, &greater);`. **Prova diretta.**

**Esito legge:** 🟢 VERDE — nessuna violazione o incoerenza rilevata rispetto al testo verbatim dei due emendamenti.

---

## 4. Provenienza commit

| Commit | `git show --stat` | File toccati | Note |
|--------|-------------------|--------------|------|
| `78ba307` (HEAD) | `src/selection.rs \| 20 +++++++++++++----`<br>`tests/l0_l1_l2_pipeline.rs \| 16 ++++++++------`<br>`tests/l2_selection.rs \| 53 +++++++++++++++++++++++++++-------------------` | `src/selection.rs`, `tests/l0_l1_l2_pipeline.rs`, `tests/l2_selection.rs` | Fix L2.25/L2.26 + aggiornamento test |
| `9d378b6` (≡ `9d378b9` nel dispatch) | `src/selection.rs \| 7 ++++++-` | `src/selection.rs` | Fix: emit summary on implicit close per L2.15/L2.16 |
| `507e76f` / `53c9525` | `tests/l0_l1_l2_pipeline.rs \| 51 ++++++++++++++++++++++++++++++++++++++++++++++` | `tests/l0_l1_l2_pipeline.rs` (nuovo file) | Test: cover end-to-end L0-L1-L2 pipeline |

**Verifica ordine commit (test-first):**
- Primo commit L2 test: `8b8019f` (`test: allinea suite L2 a L2.22 (prefisso base) — preliminare all'implementazione, suite rossa`) — **solo `tests/l2_selection.rs`**, 52 righe aggiunte, nessun codice produzione.
- Primo commit pipeline test: `507e76f` — **solo `tests/l0_l1_l2_pipeline.rs`** (nuovo file), 51 righe, nessun codice produzione.
- Implementazione `src/selection.rs` arriva dopo: `c8f7ace` (`L2 implement: selection read-only con legge L2.1-L2.22`).
- **Conferma:** i test sono stati scritti **prima** dell'implementazione. Il commit `8b8019f` dichiara esplicitamente "suite rossa" nel messaggio.

---

## 5. Nessuna modifica di codice / reviewer / accettazione
Rispettato: analisi di sola lettura, nessun `edit`/`write` su codice produzione o test.

---

## 6. Report salvato in `reports/L2_delta_pipeline_report.md` (NON committato)
Questo file è l'unico artefatto prodotto.

---

## 7. Mnemosyne
Server non raggiungibile. Nessuna memoria richiamata né proposta.

---

## 8. Working tree principale escluso
Nessuna lettura/modifica di `.opencode/agents/*.md`, `reports/L1_test_report.md`, `reports/L2_test_report.md`, `notes/mnemosyne_backlog.md`.

---

## ESITO FINALE PER VOCE

| # | Voce | Esito |
|---|------|-------|
| 1 | Suite completa `cargo test` all green (L0+L1+L2+pipeline), count L2 13+1 | 🟢 VERDE (105 totali, 14 L2) |
| 2 | Copertura `l0_l1_l2_pipeline.rs` per 12 requisiti | 🟢 VERDE (9 dirette, 3 in altre suite — NOTA) |
| 3 | Legge L2.25/L2.26 verbatim vs codice/test | 🟢 VERDE (coerente, test `equal_score_eviction_prefers_newer_origin_then_greater_id` prova L2.26) |
| 4 | Provenienza commit `78ba307`, `9d378b6`, `507e76f`/`53c9525` | 🟢 VERDE (stat confermati, test-first rispettato) |
| 5 | Nessuna modifica codice | 🟢 VERDE |
| 6 | Report in `reports/L2_delta_pipeline_report.md` | 🟢 VERDE |
| 7 | Mnemosyne tentato, irraggiungibile | 🟡 NOTA (server non connesso) |
| 8 | Working tree principale escluso | 🟢 VERDE |

---

## Domande aperte / Note

1. **`encode` non testato nel pipeline:** Il test `l0_l1_l2_pipeline.rs` non chiama `selection::encode`. La codifica è testata in `l2_selection.rs:25` e `130`. È intenzionale o il pipeline dovrebbe includere un round-trip encode/append/verify?

2. **`nodes` prima di `edges` (D19) non asserito nel pipeline:** Il test crea nodi ed archi nello stesso evento ma non verifica l'ordine nella vista. Coperto in `l1_view.rs:44`. Va bene così?

3. **Arm `ErrorCode`/echelons L2.21:** Testati in `l2_selection.rs` (`config_replacement_and_l2_21_schema_rules`, `replay_is_deterministic_and_read_only_and_noncanonical_arrays_fail`), non nel pipeline. Va bene così?

4. **Mnemosyne non disponibile:** Il gate specificava di provare `memory_open_session` una volta; l'ho fatto e il server non risponde. Nessuna azione successiva.

---
*Report generato automaticamente dal Tester (Ariadne) per gate delta L2 su HEAD 78ba307.*