# Gate procedurale L4 — implementazione

modello: `opencode/deepseek-v4.1-flash-free` (classe `light`, variante: nessuna / default di catalogo, canale free «Free-first» 31/10/2026)
policy: `.opencode/routing.md` **v1 ATTIVO** — sha256 `bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177`
mandato: Router, sessione tester indipendente; gate PROCEDURALE (esecuzione, conteggi, perimetri, hash). Nessun giudizio legge-vs-codice (Fase C, reviewer).
session ID: `ses_84ee8331-0315-4074-91c5-d20faf3ed59e` (Mnemosyne `opencode:Ariadne`, corpus `efda4e4f-fbc4-48b3-abd5-0fb9ee3a5dc4:65`)
memorie richiamate: `mem_2affa11c-2399-4ffa-b935-8aee3b1b2177`, `mem_f1b05e04-85bc-4a91-b506-3a2f7ec8293a` (contesto L3); `memory_search("L4 violazione")` → 1 risultato (accettazione L3), nessun pattern di violazione L4 pregresso. Memorie proposte: nessuna.

## Revisione verificata

- HEAD: `280412f9ee77a19c05859d3c6d89fa8102013725` (`git log --format=%H -1`)
- catena task: `c8ae8de` test rossi → `302e7e2` precisazione L4.6 `sources` + divieto-producer → `280412f` implementazione
- legge: `notes/L4_legge_identita.md` (base `2ec39a0`, precisazione `302e7e2`, righe 171–177) — sha256 `cd7e9ce6b3d1c9355c8fa0eb59b8c7002cc461dff738ea3afdb671e8cbf0d9a5`
- worktree di esecuzione: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l4gate-280412f` (git status: `nothing to commit, working tree clean`)

## 1–2. Suite completa (worktree pulito su HEAD)

### `cargo test` (debug) — 144/144 VERDI, 0 failed

```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (unittests src/lib.rs)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (src/bin/l0_replay.rs)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (l0_c1_genesis)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l0_cycle)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l0_event_log)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (l0_l1_l2_pipeline)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l0_n3_canonical_string)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l0_n3_line_canonical)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (l0_prng)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (l0_replay_process)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (l0_state)
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l1_view)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l2_selection)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l3_memory)
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out  (l4_identity)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (doc-tests)
```

Somma: 4+2+11+18+1+13+10+7+6+9+11+16+14+22 = **144 passed; 0 failed** (122 pre-esistenti L0–L3 + 22 `l4_identity`).

### `cargo test --release` — 144/144 VERDI, 0 failed

```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 3. I 22 test di `tests/l4_identity.rs` (tutti `ok`, debug e release)

| # | test | esito |
| --- | --- | --- |
| 1 | `l4_check1_genesis_only_empty_statements_head_hash_and_preimage` | ok |
| 2 | `l4_check2_i_irrelevant_difference_produces_identical_narrated_statements` | ok |
| 3 | `l4_check2_ii_relevant_difference_produces_divergent_narrated_statements` | ok |
| 4 | `l4_check3_all_templates_exact_texts_internal_order_sources_and_join` | ok |
| 5 | `l4_check3_select_list_rendering_sorted_dedup_and_empty` | ok |
| 6 | `l4_check3_control_rementions_and_no_extraction_events_are_not_relevant` | ok |
| 7 | `l4_check4_replay_rebuilds_document_and_snapshots_verified` | ok |
| 8 | `l4_check4_lone_narrate_as_last_event_is_tolerated_and_authoritative` | ok |
| 9 | `l4_check4_unpaired_snapshot_is_schema` | ok |
| 10 | `l4_check4_broken_pairs_are_schema` | ok |
| 11 | `l4_check4_self_hash_mismatch_is_schema` | ok |
| 12 | `l4_check4_narrate_at_inexistent_or_l4_is_schema` | ok |
| 13 | `l4_check4_l4_with_l2_or_l3_is_schema` | ok |
| 14 | `l4_check4_extra_keys_and_bad_shapes_are_schema` | ok |
| 15 | `l4_check4_state_with_l4_is_schema` | ok |
| 16 | `l4_check4_malformed_l3_fields_are_schema` | ok |
| 17 | `l4_check5_cutoff_tail_only_and_prefix_coherence` | ok |
| 18 | `l4_check6_l4_event_closes_l2_window_without_side_effects` | ok |
| 19 | `l4_check7_determinism_two_evaluations_identical` | ok |
| 20 | `l4_check8_l4_apis_do_not_advance_prng_state` | ok |
| 21 | `l4_encode4_narrate_and_snapshot_payload_pairs` | ok |
| 22 | `l4_encode4_payload_appendable_as_l0_event` | ok |

Conteggio `#[test]` nel file: 22 (righe 156, 178, 225, 254, 328, 349, 400, 451, 464, 473, 506, 516, 533, 556, 629, 643, 662, 713, 744, 771, 795, 831).

### Copertura punto per punto della checklist di uscita (righe ~255–280 della legge)

1. **Genesi** — `l4_check1_…` (l.156–168): `build_self` su sola genesi, `statements` vuoto, `head_hash` = hash genesi, `self_hash` = SHA-256 del preimage ricostruito a mano nel test. **VERDE**
2. **Divergenza scoped** — `l4_check2_i_…` (l.178–220, eventi irrilevanti in coda → statements identiche; asserisce esplicitamente il NON-iniettività del documento via `assert_ne!` su `head_hash`/`self_hash`, conforme a Q9) + `l4_check2_ii_…` (l.225–243, evento rilevante → narrazione diversa). **VERDE**
3. **Template L4.3** — `l4_check3_all_templates_…` (l.254–325: nodo, arco, select, summary, promote, archive, recall, ordine interno lessicografico, `\n`, `[]`, virgole senza spazi, `sources`, ordine Narrate→Snapshot con at/tick) + `l4_check3_select_list_…` (l.328–344, ordinamento/dedup/vuoto) + `l4_check3_control_rementions_…` (l.349–388, esclusi riprove/menzioni/controlli l2-l3). **VERDE**
4. **Replay + errori** — `l4_check4_replay_…` (l.400–446, documento ricostruito identico alla simulazione, snapshot verificati, cutoff sul prefisso) + i 9 test d'errore `l4_check4_unpaired_snapshot`, `broken_pairs`, `self_hash_mismatch`, `narrate_at_inexistent_or_l4`, `l4_with_l2_or_l3`, `extra_keys_and_bad_shapes`, `state_with_l4`, `malformed_l3_fields`, `lone_narrate_as_last_event` (l.451–653): tutti `unwrap_err() == SelfError::Schema`, nessun panic (gli unwrap_err passano). **VERDE** (procedurale: nessun panic/UB osservato; la correttezza semantica degli errori è del reviewer)
5. **Cutoff** — `l4_check5_cutoff_tail_only_and_prefix_coherence` (l.662–705): decisioni solo per la coda oltre cutoff, `evaluate_self` su base = log intero → vuoto, `HeadMatch` su hash di prefisso errato / log più corto / base vuota. **VERDE**
6. **Interazione L2/L3** — `l4_check6_l4_event_closes_l2_window_without_side_effects` (l.713–736): evento `l4` chiude la finestra L2 (L2.25), `selection::evaluate` e `memory::evaluate` vuoti, cattura L2/L3 intatta, L4 coerente. **VERDE**
7. **Determinismo** — `l4_check7_determinism_two_evaluations_identical` (l.744–763): due `evaluate_self` identiche (`assert_eq!` su Vec e su `Debug`), due `build_self` identici. **VERDE**
8. **Zero draw** — `l4_check8_l4_apis_do_not_advance_prng_state` (l.771–787): stato PCG64 della genesi invariato prima/dopo `build_self`+`evaluate_self`, `log.verify()` ok. **VERDE**; "tutta la suite L0–L3 resta VERDE (122 test)" provato dalle run qui sopra (108 + 4 unit + … = 122 pre-esistenti, 0 failed). **VERDE**

Extra fuori checklist ma L4.6: `l4_encode4_narrate_and_snapshot_payload_pairs`, `l4_encode4_payload_appendable_as_l0_event` — entrambi ok.

## 4. Perimetro `302e7e2..HEAD` — INTEGRO

`git diff --stat 302e7e2..HEAD`:

```
 src/identity.rs      | 634 +++++++++++++++++++++++++++++++++++++++++++++++++++
 src/lib.rs           |   1 +
 tests/l4_identity.rs |  38 +--
 3 files changed, 656 insertions(+), 17 deletions(-)
```

`git show --stat 280412f` coincide (3 file, 656 inserzioni, 17 rimozioni). Nessun altro file: nessun `reports/**`, nessun `notes/*`, nessun `.DS_Store` nel commit. `src/lib.rs` è esattamente `+pub mod identity;` (una riga, dopo `mod event;`). Le 38 righe di `tests/l4_identity.rs` sono le correzioni autorizzate dei test (7 hunk / 8 correzioni logiche: `sources` in forma F1 `[…]`, seq 2→4 in check2(i), array `nodes` in check2(ii), `windows(2)`→`chunks(2)`, `d[0]`→`d[2]`, indici eventi in check4, `.clone()` in check5, `last()`→`[0]` per lo stato PRNG in check8).

Working tree principale: nessun file tracciato modificato (`git status`: solo untracked `notes/*` e `reports/*` preesistenti).

## 5. HEAD

```
280412f9ee77a19c05859d3c6d89fa8102013725
```

## 6. SHA-256

```
cd7e9ce6b3d1c9355c8fa0eb59b8c7002cc461dff738ea3afdb671e8cbf0d9a5  notes/L4_legge_identita.md
bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177  .opencode/routing.md
```

## 7. Test-first: primo commit = solo test, e rossi (verifica di ruolo)

`git diff --name-status 2ec39a0..c8ae8de` → `A	tests/l4_identity.rs` (unico file, +835 righe, nessun codice di produzione).

Rosso a runtime ricostruito in worktree separato su `c8ae8de` (`cargo test`):

```
error[E0432]: unresolved import `ariadne::identity`
  --> tests/l4_identity.rs:19:14
   |
19 | use ariadne::identity::{
   |              ^^^^^^^^ could not find `identity` in `ariadne`

error[E0433]: cannot find `identity` in `ariadne`
   --> tests/l4_identity.rs:155:23
    |
155 |     let doc: ariadne::identity::SelfDocument = build_self(&log).unwrap();
    |                        ^^^^^^^^ could not find `identity` in `ariadne`

Some errors have detailed explanations: E0432, E0433.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `ariadne` (test "l4_identity") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...

Exited with code 101
```

Rosso per non-compilazione, nessuno stub di produzione nel commit di test. Conforme al pattern test-first (L2/L3). **VERDE**

## Esito

| voce | esito |
| --- | --- |
| suite debug 144/144 | VERDE |
| suite release 144/144 | VERDE |
| 22 test l4 presenti e verdi | VERDE |
| checklist 1–8 coperte dai test | VERDE (mappatura sopra) |
| perimetro diff 302e7e2..HEAD | VERDE (3 file, nulla fuori) |
| HEAD hash | VERDE |
| sha256 legge + policy | VERDE |
| primo commit solo test + rosso runtime | VERDE |

**VERDETTO: VERDE** (gate procedurale L4 — scope: esecuzione, conteggi, perimetro, hash, copertura test della checklist. Non valuta la conformità legge-vs-codice, che resta al reviewer in Fase C; non approva il livello.)

Note di metodo: nessuno strumento negato ha impedito verifiche; `git rev-parse` non è in allowlist del ruolo → hash HEAD ottenuto con `git log --format=%H -1`. Worktree temporanei rimossi senza `--force` (0 modifiche). Nessun comando composto, nessuna pipeline.
