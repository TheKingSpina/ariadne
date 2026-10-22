# L0 — Report di verifica FINALE (Tester, gate di livello)

- **SPEC di riferimento:** `spec/L0_SPEC_v0.2.md`, integrale (§1–§5 + Ciclo + Schema PRNG)
- **HEAD esaminato:** `a298cf9` (`L0 B4: fix import nel test del ciclo …`)
- **Ruolo:** Tester. **Nessuna accettazione dichiarata**: la firma è umana.
- **Data:** 2026-10-22
- **Scrittura:** incrementale (questo file è aggiornato a mano a mano che le verifiche chiudono)
- **Vincoli rispettati:** nessuna modifica a `src/` o `tests/`; nessun file creato fuori da `reports/`.

## Storia dei commit (da `git log --oneline -13`)

```
a298cf9 L0 B4: fix import nel test del ciclo — Pcg64 sta in ariadne::prng ... porta verde il test rosso 08b1536
b2790e9 L0 B4: implementazione del ciclo SENSE→ACTION→STATE — ...
08b1536 L0 B4: test ROSSI per il ciclo senso→azione→stato ... solo test, nessun codice di produzione ...
a345be7 L0 B3: fold dello State (src/state.rs + pub mod state) — ... test-first: 48e5529 (rossi)
48e5529 L0 B3: test ROSSI per fold dello State (SPEC §4, riga 43) — solo test, nessun codice di produzione
b7543f7 L0 B2: genesi con stato PCG64 reale a 128 bit (chiude D1)
d2f1610 L0 B2: test ROSSI per genesi con stato PCG64 stringa (chiude D1)
99b9d06 L0: engine PCG64 (PCG XSL RR 128/64) — test a VERDE
038e51d L0: test ROSSI per Schema PRNG (PCG64) — solo test, nessun codice di produzione
8d71937 L0: implementazione SPEC v0.2 §2-§3 + genesi — test a VERDE
6ae990a L0: test ROSSI per SPEC v0.2 §2-§3 + genesi (solo test, nessun codice di produzione)
79e106c docu L0-L6 v0.1/0.2
c15defb readme + L0 spec v0.2
```

## 3. Suite su worktree pulito all'HEAD (`a298cf9`)

Worktree: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0final_head`
`git status --short` nel worktree: **nessun output** (pulito).

### 3.1 `cargo test` — output INTEGRALE

```
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0final_head)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.11s
     Running unittests src/lib.rs (target/debug/deps/ariadne-f4839cd99eb39cde)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::genesis_line_parses ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_cycle.rs (target/debug/deps/l0_cycle-7096279ec33ea04c)

running 11 tests
test mossa_e_parse_stimolo_mod_3_su_piu_tick ... ok
test budget_draw_1_per_tick_0_per_la_policy ... ok
test mai_cicli_parziali_ritmo_sense_action_state ... ok
test seme_diverso_produce_log_diverso ... ok
test nessun_tick_salta_sequenza_0_1_1_1_2_2_2 ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
test stimolo_oltre_i64_max_parse_completo_senza_mascheratura ... ok
test un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi ... ok
test determinismo_forte_due_log_indipendenti_byte_identici ... ok
test stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-d009f4ce22e7032a)

running 16 tests
test test_event_without_seq_is_rejected ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_genesis_golden_hash_constant ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_state_string_format ... ok
test test_immutable_events_slice_borrows ... ok
test test_hash_chain_end_to_end ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_prng.rs (target/debug/deps/l0_prng-66eed8ff3b88a194)

running 7 tests
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test l_incremento_non_e_mai_pari ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_state.rs (target/debug/deps/l0_state-885695fc0ab4fead)

running 7 tests
test test_compensating_state_has_no_snapshot_and_keeps_last_prng ... ok
test test_fold_genesis_initial_state ... ok
test test_head_is_hash_of_last_event_not_genesis ... ok
test test_counters_count_hand_built_sequence ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_fold_is_pure_byte_identical ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Totale debug: 45/45 verdi** (4 unit + 11 ciclo + 16 event/log + 7 prng + 7 state), 0 falliti, 0 ignorati.

### 3.2 `cargo test --release` — output INTEGRALE

```
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0final_head)
    Finished `release` profile [optimized] target(s) in 2.36s
     Running unittests src/lib.rs (target/release/deps/ariadne-e38db9ac10129f4e)

running 4 tests
test event::tests::compute_hash_matches_documented_preimage ... ok
test event::tests::event_type_parse_is_total_and_strict ... ok
test log::tests::genesis_line_parses ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_cycle.rs (target/release/deps/l0_cycle-d91548061326f2bc)

running 11 tests
test budget_draw_1_per_tick_0_per_la_policy ... ok
test determinismo_forte_due_log_indipendenti_byte_identici ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
test mai_cicli_parziali_ritmo_sense_action_state ... ok
test mossa_e_parse_stimolo_mod_3_su_piu_tick ... ok
test seme_diverso_produce_log_diverso ... ok
test nessun_tick_salta_sequenza_0_1_1_1_2_2_2 ... ok
test stimolo_oltre_i64_max_parse_completo_senza_mascheratura ... ok
test stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa ... ok
test un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/release/deps/l0_event_log-394ff1683e79bef1)

running 16 tests
test test_event_without_seq_is_rejected ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_genesis_golden_hash_constant ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_state_string_format ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_hash_chain_end_to_end ... ok
test test_immutable_events_slice_borrows ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_prng.rs (target/release/deps/l0_prng-8bfd63ad8f966c88)

running 7 tests
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok
test l_incremento_non_e_mai_pari ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_state.rs (target/release/deps/l0_state-f1b7432b9edcbadf)

running 7 tests
test test_compensating_state_has_no_snapshot_and_keeps_last_prng ... ok
test test_counters_count_hand_built_sequence ... ok
test test_fold_genesis_initial_state ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_fold_is_pure_byte_identical ... ok
test test_head_is_hash_of_last_event_not_genesis ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Totale release: 45/45 verdi**, identico al debug. Nessuna differenza di comportamento fra profili.

*(verifiche successive in corso — sezioni 1, 2, 4, 5, 6 in aggiornamento)*
