# L0 — Gate PRNG (B1) e Genesi con stato reale (B2)

- Livello: L0
- SPEC: `spec/L0_SPEC_v0.2.md` §4 + "Schema PRNG" (righe 21–25, 40–44)
- Commit sotto gate:
  - B1: test `038e51d` → implementazione `99b9d06`
  - B2: test `d2f1610` → implementazione `b7543f7` (HEAD)
- Worktree pulito: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0_b7543f7` (detached HEAD su `b7543f7`)
- Tester: misura sola conformità; nessuna correzione applicata; nessun file creato fuori da `reports/`.

---

## 1. Golden non circolare (priorità massima)

### 1.1 Costante nel repo

`tests/l0_event_log.rs:27`

```
const GENESIS_1042_HASH: &str = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";
```

Lunghezza misurata: **64** caratteri esadecimali (8 gruppi da 8). Il valore "da 55 caratteri" citato in chat è quindi una **trascrizione troncata in chat**, non un valore del repo: l'ipotesi (a) è quella vera per la chat; la costante in repo è a 64 cifre.

Prova del valore nel commit di test (precedente all'implementazione):

```
$ git show d2f1610:tests/l0_event_log.rs
...
const GENESIS_1042_HASH: &str = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";
...
```

### 1.2 Preimage ricostruito

Byte (152), ricostruiti a mano dai campi §2 e dallo Schema PRNG:

```
0x1F-separated: "0" 0x1F "0" 0x1F "STATE" 0x1F
{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"} 0x1F
"0000000000000000000000000000000000000000000000000000000000000000"
```

Hex verificato (152 byte):

```
301f301f53544154451f7b2270726e67223a225043473634222c2273656564223a313034322c227374617465223a223433313439393734383634323533383036323836333336303434363835373032363632333539227d1f30303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030
```

La stessa sequenza di byte è rappresentata anche in base64 (per l'esecuzione sotto riportata, la decodifica base64 ignora il newline aggiunto dall'here-string):

```
MB8wH1NUQVRFH3sicHJuZyI6IlBDRzY0Iiwic2VlZCI6MTA0Miwic3RhdGUiOiI0MzE0OTk3NDg2NDI1MzgwNjI4NjMzNjA0NDY4NTcwMjY2MjM1OSJ9HzAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDA=
```

### 1.3 `shasum` eseguito sui byte ricostruiti

Comando equivalente (la shell del tester ha `printf` e `python3` negati dai permessi; `head`, `openssl`, `shasum` e le pipe sono ammessi — vedi §1.5):

```
$ openssl base64 -d <<< "<base64 sopra>" | shasum -a 256
4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7  -
```

Controllo del metodo (stessa pipeline su "hi", SHA-256 noto):

```
$ openssl base64 -d <<< "aGk=" | shasum -a 256
8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4  -
```

Seconda implementazione indipendente (SHA-256 in JS nel runtime del tester, autotest su vettori NIST `abc` e stringa vuota):

```
sha256("abc")   = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
sha256("")      = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
sha256(preimage) = 4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7  (len 64)
```

### 1.4 Confronto carattere per carattere

```
shasum  : 4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7
costante: 4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7
```

**COINCIDONO — 64/64 caratteri identici.**

### 1.5 Esito golden

- Il test **non** calcola il golden con `compute_hash`/`to_jsonl`: `test_genesis_golden_hash_constant` (`tests/l0_event_log.rs:100-109`) confronta `log.events()[0].hash` e `log.to_jsonl()` con costanti letterali (`GENESIS_1042_HASH`, `GENESIS_1042_LINE`). Il ricalcolo compare solo nei test della catena (§2), dove è la cosa giusta.
- La costante era già in `d2f1610` (commit test, precedente a `b7543f7`).
- Unica riserva (NOTA, non rosso): la porzione "state" del preimage è a sua volta prodotta dal crate (`GENESIS_1042_STATE_DEC` dichiarato come output di `Pcg64::new(1042).state_decimal()`, `tests/l0_event_log.rs:29-32`); l'hash la *congela* ma non ne dimostra la correttezza contro un riferimento esterno. Vedi §6.

**Voce 1: VERDE** (shasum esterno riprodotto: coincide).

---

## 2. Test-first B1 e B2

### 2.1 Ordine dei commit (linearità)

```
$ git log --format='%h %p %ad %s' --date=iso -6
b7543f7 d2f1610 2026-10-22 17:24:17 +0200 L0 B2: genesi con stato PCG64 reale a 128 bit (chiude D1)
d2f1610 99b9d06 2026-10-22 17:23:42 +0200 L0 B2: test ROSSI per genesi con stato PCG64 stringa (chiude D1)
99b9d06 038e51d 2026-10-22 17:18:59 +0200 L0: engine PCG64 (PCG XSL RR 128/64) — test a VERDE
038e51d 8d71937 2026-10-22 17:14:08 +0200 L0: test ROSSI per Schema PRNG (PCG64) — solo test, nessun codice di produzione
8d71937 6ae990a 2026-10-22 17:00:22 +0200 L0: implementazione SPEC v0.2 §2-§3 + genesi — test a VERDE
6ae990a 79e106c 2026-10-22 16:50:47 +0200 L0: test ROSSI per SPEC v0.2 §2-§3 + genesi (solo test, nessun codice di produzione)
```

Catena lineare: test B1 (`038e51d`) precede impl B1 (`99b9d06`) che precede test B2 (`d2f1610`) che precede impl B2 (`b7543f7`). **VERDE**.

### 2.2 B1 — commit di test `038e51d`

```
$ git show --stat --oneline 038e51d
038e51d L0: test ROSSI per Schema PRNG (PCG64) — solo test, nessun codice di produzione
 tests/l0_prng.rs | 208 +++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 208 insertions(+)
```

Nessun file in `src/`. Il file è interamente test.

Prova di ROSSO (worktree dedicato su `038e51d`, checkout pulito):

```
$ cargo test
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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0_red_b1)
error[E0432]: unresolved import `ariadne::prng`
  --> tests/l0_prng.rs:24:14
   |
24 | use ariadne::prng::Pcg64;
   |              ^^^^ could not find `prng` in `ariadne`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `ariadne` (test "l0_prng") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

Exited with code 101
```

### 2.3 B1 — commit di implementazione `99b9d06`

```
$ git show --stat --oneline 99b9d06
99b9d06 L0: engine PCG64 (PCG XSL RR 128/64) — test a VERDE
 src/lib.rs  |   5 ++-
 src/prng.rs | 130 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 134 insertions(+), 1 deletion(-)
```

Diff dei test fra commit di test e commit di implementazione:

```
$ git diff 038e51d 99b9d06 -- tests/l0_prng.rs
(nessun output)
```

Nessuna asserzione modificata/allentata. **VERDE**.

### 2.4 B2 — commit di test `d2f1610`

```
$ git show --stat --oneline d2f1610
d2f1610 L0 B2: test ROSSI per genesi con stato PCG64 stringa (chiude D1)
 src/log.rs            |  6 ++---
 tests/l0_event_log.rs | 72 ++++++++++++++++++++++++++++++++++++++++++++++-----
 2 files changed, 68 insertions(+), 10 deletions(-)
```

Il commit tocca `src/log.rs`, ma **solo il blocco `#[cfg(test)]`** (righe 468–500), non codice di produzione:

```
$ git show d2f1610 -- src/log.rs
...
@@ -465,13 +465,13 @@ fn parse_u4x(c: &mut Cursor) -> Result<u32, Error> {
 mod tests {
     use super::*;
 
-    /// Parser: la riga genesi canonica si compone.
+    /// Parser: la riga genesi canonica si compone (formato B2: state stringa).
     #[test]
     fn genesis_line_parses() {
         const GENESIS_1042_LINE: &str = concat!(
-            r#"{"seq":0,"tick":0,"type":"STATE","payload":{"prng":"PCG64","seed":1042,"state":1042},"#,
+            r#"{"seq":0,"tick":0,"type":"STATE","payload":{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"},"#,
             r#""prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","#,
-            r#""hash":"e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97"}"#,
+            r#""hash":"4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7"}"#,
         );
```

Nessuna funzione di produzione modificata in `d2f1610`: `src/log.rs` è toccato solo dentro `mod tests` (unit test), `tests/l0_event_log.rs` è test integrazione. La regola "solo test" è rispettata nella sostanza (nessuna riga di produzione); resta la nota formale che il commit di test modifica un file in `src/`. **VERDE con nota**.

Prova di ROSSO (worktree dedicato su `d2f1610`, checkout pulito):

```
$ cargo test
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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0_red_b2)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.14s
     Running unittests src/lib.rs (target/debug/deps/ariadne-f4839cd99eb39cde)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test log::tests::genesis_line_parses ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-d009f4ce22e7032a)

running 16 tests
test test_event_without_seq_is_rejected ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_genesis_state_is_string_not_integer ... FAILED
test test_genesis_prev_hash_zeroes_and_shape ... FAILED
test test_genesis_golden_hash_constant ... FAILED
test test_genesis_state_matches_engine_state_decimal ... FAILED
test test_genesis_state_string_format ... FAILED
test test_event_types_constrained_to_sense_action_state ... ok
test test_immutable_events_slice_borrows ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_hash_chain_end_to_end ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok

failures:

---- test_genesis_state_is_string_not_integer stdout ----

thread 'test_genesis_state_is_string_not_integer' (1170004) panicked at tests/l0_event_log.rs:120:5:
state è stringa: {"prng":"PCG64","seed":1042,"state":1042}

---- test_genesis_prev_hash_zeroes_and_shape stdout ----

thread 'test_genesis_prev_hash_zeroes_and_shape' (1170003) panicked at tests/l0_event_log.rs:91:5:
assertion `left == right` failed: payload genesi: seed intero, prng PCG64, state STRINGA decimale
  left: "{\"prng\":\"PCG64\",\"seed\":1042,\"state\":1042}"
 right: "{\"prng\":\"PCG64\",\"seed\":1042,\"state\":\"43149974864253806286336044685702662359\"}"

---- test_genesis_golden_hash_constant stdout ----

thread 'test_genesis_golden_hash_constant' (1170002) panicked at tests/l0_event_log.rs:106:5:
assertion `left == right` failed
  left: "e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97"
 right: "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- test_genesis_state_matches_engine_state_decimal stdout ----

thread 'test_genesis_state_matches_engine_state_decimal' (1170005) panicked at tests/l0_event_log.rs:139:18:
state del payload genesi non è una stringa: Some(Int(1042))

---- test_genesis_state_string_format stdout ----

thread 'test_genesis_state_string_format' (1170006) panicked at tests/l0_event_log.rs:150:18:
state non è una stringa: Some(Int(1042))


failures:
    test_genesis_golden_hash_constant
    test_genesis_prev_hash_zeroes_and_shape
    test_genesis_state_is_string_not_integer
    test_genesis_state_matches_engine_state_decimal
    test_genesis_state_string_format

test result: FAILED. 11 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test l0_event_log`

Exited with code 101
```

5 test rossi, esattamente i 5 nuovi test B2; gli altri 11 restano verdi (nessuna regressione indotta dal commit di test). **VERDE**.

### 2.5 B2 — commit di implementazione `b7543f7`

```
$ git show --stat --oneline b7543f7
b7543f7 L0 B2: genesi con stato PCG64 reale a 128 bit (chiude D1)
 src/log.rs | 20 ++++++++++++--------
 1 file changed, 12 insertions(+), 8 deletions(-)
```

Diff dei test fra commit di test e commit di implementazione:

```
$ git diff d2f1610 b7543f7 -- tests/l0_event_log.rs
(nessun output)
```

Nessuna asserzione modificata/allentata. **VERDE**.

---

## 3. Suite su worktree pulito

### 3.1 Preparazione

```
$ git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0_b7543f7 b7543f7
Preparing worktree (detached HEAD b7543f7)
HEAD is now at b7543f7 L0 B2: genesi con stato PCG64 reale a 128 bit (chiude D1)

$ git status --short
(nessun output)
```

Worktree pulito prima e dopo la build (`target/` è in `.gitignore`). **VERDE**.

### 3.2 `cargo test` (debug) — output integrale

```
$ cargo test
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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0_b7543f7)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.50s
     Running unittests src/lib.rs (target/debug/deps/ariadne-f4839cd99eb39cde)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test log::tests::genesis_line_parses ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-d009f4ce22e7032a)

running 16 tests
test test_event_without_seq_is_rejected ... ok
test test_genesis_golden_hash_constant ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_genesis_state_string_format ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_immutable_events_slice_borrows ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_hash_chain_end_to_end ... ok
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

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 3.3 `cargo test --release` — output integrale

```
$ cargo test --release
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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0_b7543f7)
    Finished `release` profile [optimized] target(s) in 3.61s
     Running unittests src/lib.rs (target/release/deps/ariadne-e38db9ac10129f4e)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::genesis_line_parses ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/release/deps/l0_event_log-394ff1683e79bef1)

running 16 tests
test test_event_types_constrained_to_sense_action_state ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_event_without_seq_is_rejected ... ok
test test_genesis_golden_hash_constant ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_genesis_state_string_format ... ok
test test_hash_chain_end_to_end ... ok
test test_immutable_events_slice_borrows ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_unknown_top_level_member_rejected ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_prng.rs (target/release/deps/l0_prng-8bfd63ad8f966c88)

running 7 tests
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok
test l_incremento_non_e_mai_pari ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**VERDE**: 4 + 16 + 7 test verdi, 0 falliti, debug e release.

### 3.4 Replay bit-perfetto (due esecuzioni indipendenti)

Due processi indipendenti (build debug e build release), ciascuno esegue il confronto byte-per-byte interno:

```
$ cargo test --test l0_event_log test_replay_prefix_byte_per_byte -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-d009f4ce22e7032a)

running 1 test
test test_replay_prefix_byte_per_byte ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s

$ cargo test --release --test l0_event_log test_replay_prefix_byte_per_byte -- --nocapture
    Finished `release` profile [optimized] target(s) in 0.02s
     Running tests/l0_event_log.rs (target/release/deps/l0_event_log-394ff1683e79bef1)

running 1 test
test test_replay_prefix_byte_per_byte ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
```

Il test (`tests/l0_event_log.rs:315-336`) costruisce il prefisso due volte da codice indipendente, confronta `jsonl_a.as_bytes() == jsonl_b.as_bytes()`, poi rilegge con `from_jsonl` e confronta di nuovo i byte del round-trip: `assert_eq!(replayed.to_jsonl().as_bytes(), jsonl_a.as_bytes(), "riplay byte per byte")`.

**NOTA metodologica:** il crate non espone un binario/CLI che stampi il JSONL, quindi non è stato possibile estrarre i byte da due processi e diffarli con `cmp` esterno. La prova eseguita è: (i) doppia build interna al test + round-trip byte-per-byte, (ii) due processi separati (debug, release) che la eseguono con esito verde. Comando esatto per chiudere anche la parte esterna, quando esisterà un entrypoint che stampi `to_jsonl()`:

```
# due esecuzioni indipendenti, poi:
cmp run1.jsonl run2.jsonl && shasum -a 256 run1.jsonl run2.jsonl
```

**Voce 3: VERDE** (suite + replay interno); punto (ii) del byte-diff esterno **NOTA**.

---

## 4. Formato e coerenza `state`

Payload genesi effettivo (`tests/l0_event_log.rs:91-93`, asserito contro letterale; il test passa):

```
{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"}
```

- `prng` = stringa `"PCG64"` — **VERDE** (`src/log.rs:62`, `Value::Str("PCG64")`).
- `seed` = intero `1042` (JSON senza virgolette) — **VERDE** (`src/log.rs:63`, `Value::Int(seed as i64)`; asserito in `tests/l0_event_log.rs:122`).
- `state` = **stringa** decimale, non intero — **VERDE** (`src/log.rs:64`, `Value::Str(state)`; asserito in `tests/l0_event_log.rs:120` e `124-125`).
- `prev_hash` = 64 zeri — **VERDE** (`tests/l0_event_log.rs:90`; `src/event.rs:24`).
- `seq` = 0 — **VERDE** (`tests/l0_event_log.rs:87`; `src/log.rs:66`).
- `tick` = 0 — **VERDE** (`tests/l0_event_log.rs:88`; `src/log.rs:66`).
- `type` = `STATE` — **VERDE** (`tests/l0_event_log.rs:89`; `src/log.rs:66`).
- Formato `state`: solo cifre / nessun segno / nessuno zero in testa / ≤ 39 — **VERDE** (`tests/l0_event_log.rs:145-157`). Il valore ha 38 cifre, inizia con `4`: nessuno zero in testa, `len=38 ≤ 39`.

### 4.1 `state` == `Pcg64::new(1042).state_decimal()` — non tautologico?

`tests/l0_event_log.rs:129-142`:

```rust
let atteso = Pcg64::new(1042).state_decimal();
assert_eq!(atteso, GENESIS_1042_STATE_DEC, "lo stato d'engine non è cambiato");
let log = Log::genesis(1042);
... match payload.get("state") { Some(Value::Str(s)) => s.clone(), ... }
assert_eq!(state, atteso);
```

- Il confronto **payload ↔ engine** attraversa due percorsi distinti (serializzazione del payload in `Log::genesis` vs output diretto dell'engine): non è tautologico. **VERDE**.
- Il confronto **engine ↔ costante** è un pin di regressione: la costante è dichiarata come output dell'engine (`tests/l0_event_log.rs:29-32`). Di per sé sarebbe circolare; l'ho rotto ricostruendo il valore con una **seconda implementazione indipendente** della procedura di seeding documentata (splitmix64 + due passi LCG, aritmetica 128 bit a limb 16-bit, autotest su vettore noto `splitmix64(0)=e220a8397b1dcdaf` e su `(2^64+1)^2 mod 2^128 = 0x20000000000000001`):

```
g0 (splitmix #1) = 95f9f779259e6b9d
g1 (splitmix #2) = 5f3e81892c6c34f0
increment        = be7d031258d869e1
state (dec)      = 43149974864253806286336044685702662359
atteso           = 43149974864253806286336044685702662359
match            = true
```

**VERDE** (valore indipendentemente riprodotto dal crate, non solo copiato).

---

## 5. Proprietà PCG64 (`tests/l0_prng.rs`, 7 test)

1. `stesso_seed_produce_sequenza_identica_da_istanze_indipendenti` (`:42-47`) — `draws()` (`:30-33`) crea **due `Pcg64::new(SEED_RIF)` separati**, nessun clone: `let a = draws(...); let b = draws(...); assert_eq!(a, b)`. **VERDE**.
2. `interleaving_di_osservazioni_non_altera_la_sequenza` (`:50-63`) — due istanze indipendenti, `state_decimal()` chiamato tra i draw sull'istanza "pulita"; le sequenze devono coincidere. Prova l'assenza di sorgenti esterne nei draw (osservazione senza side effect: `state_decimal(&self)`). **VERDE** (proprietà debole ma coerente col nome).
3. `clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato` (`:72-92`) — un'istanza avanzata di N draw e un clone della stessa istanza fresca che fa gli stessi N draw devono avere lo stesso `state_decimal()` e lo stesso draw successivo. **VERDE**.
4. `clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect` (`:95-126`) — snapshot a metà percorso (`orig.clone()` dopo `DRAWS`); il futuro atteso è calcolato da una **terza istanza indipendente** avanzata degli stessi `DRAWS`; 8 cloni riproducono `atteso[0..DRAWS/8]`; poi `orig` continua e deve produrre `atteso[..8]`. Verifica il **futuro**, non solo lo stato. **VERDE**.
5. `il_corpus_l6_produce_sequenze_distinte_per_seme` (`:134-151`) — `CORPUS = 1000..1100` (`:27`), 64 uscite per seme, `assert_ne!` **a coppie su tutti i 4950 confronti**: verifica la distinzione, non la semplice assenza di panico. **VERDE**.
6. `state_decimal_e_round_trippabile_e_senza_zero_in_testa` (`:159-189`) — su 9 semi: solo cifre ASCII, `parse::<u128>()` e `v.to_string() == s`, nessuno zero in testa (salvo `"0"`), `len ≤ 39`. **VERDE**.
7. `l_incremento_non_e_mai_pari` (`:197-207`) — su 7 semi `increment() % 2 == 1`. **VERDE**.

**Osservazione (NOTA, non rosso):** i 7 test sono *property-based*; il repo non contiene alcun golden numerico delle uscite `next_u64` (né numpy né riferimento C++). Quindi essi provano determinismo/istantanea/formato/distinzione, **non** che l'output coincida con PCG64 di riferimento. Il cross-check numpy citato dal producer non è in repo e non è ripetibile in CI (dichiarato nel commit `99b9d06`). La conformità "PCG64" dell'output poggia su revisione della formula (`src/prng.rs:105-115`: `rot = state>>122`, `xsl = (state>>64)^state`, `rotate_right`), non su un test.

---

## 6. Divieti

### 6.1 Zero wall-clock / I/O / rete / float in `src/`

```
$ grep -E 'std::time|SystemTime|Instant|std::fs|File::|std::net|TcpStream|UdpSocket|f32|f64|rand::|thread_rng|std::env|std::io|println!|eprintln!|Command::new|as f64|as f32' -r src/
(nessun match)
```

Ispezionati anche `src/lib.rs:15` ("Determinismo: zero wall-clock, zero I/O esterna, zero reti, zero float"), `src/event.rs:19` (solo `use std::fmt`), `src/log.rs:20` (solo `use std::fmt`), `src/prng.rs` (sola aritmetica intera `u64`/`u128`). Dipendenza unica: `sha2` (`Cargo.toml:9`). **VERDE**.

### 6.2 Nessuna API di mutazione del log

API pubblica di `Log` (`src/log.rs`):

- `genesis(seed) -> Log` (`:59`)
- `append(&mut self, ...) -> Result<&Event, Error>` (`:72`) — solo in coda
- `compensate(&mut self, ...) -> Result<Event, Error>` (`:87`) — solo in coda
- `events(&self) -> &[Event]` (`:105`) — sola lettura
- `verify(&self)` (`:112`), `to_jsonl(&self)` (`:140`), `from_jsonl(&str)` (`:152`)

Nessun `delete`/`truncate`/`remove`/`insert_at`/`set`/riordino; `events` restituisce una slice immutabile; il campo `events` è privato (`src/log.rs:51`). **VERDE**.

Nota (non violazione): `Payload::insert` (`src/payload.rs:67`) è un mutatore pubblico su un *valore* payload, non sul log; `Log::append` accetta un payload per valore e `Event::seal` è `pub(crate)`. Non esiste un percorso pubblico per mutare un evento già nel log.

---

## 7. Riepilogo voci

| # | Voce | Esito | Prova |
|---|------|-------|-------|
| 1 | Golden non circolare: `shasum` sui byte ricostruiti | **VERDE** | `4649d1f4…26b7` = costante, 64/64 char; `tests/l0_event_log.rs:27`, `:101-109` |
| 2a | B1 test-first (`038e51d` solo test, rosso) | **VERDE** | `git show --stat`, `E0432` su `ariadne::prng` |
| 2b | B1 impl non tocca i test | **VERDE** | `git diff 038e51d 99b9d06 -- tests/l0_prng.rs` vuoto |
| 2c | B2 test-first (`d2f1610`, 5 rossi) | **VERDE con nota** | `src/log.rs` toccato solo in `mod tests`; 5 FAILED |
| 2d | B2 impl non tocca i test | **VERDE** | `git diff d2f1610 b7543f7 -- tests/l0_event_log.rs` vuoto |
| 3 | Suite debug + release su worktree pulito | **VERDE** | 4+16+7 passed, 0 failed |
| 3b | Byte-diff replay fra due processi esterni | **NOTA** | nessun entrypoint che stampi il JSONL; test interno verde in 2 processi |
| 4 | Payload/formato `state` e campi genesi | **VERDE** | `tests/l0_event_log.rs:82-157`, `src/log.rs:59-68` |
| 4b | `state` == engine `state_decimal()` | **VERDE** | `tests/l0_event_log.rs:129-142` + ricostruzione indipendente |
| 5 | 7 test PCG64 provano ciò che dichiarano | **VERDE** (con NOTA: nessun golden d'uscita) | `tests/l0_prng.rs:42-207` |
| 6 | Divieti wall-clock/I/O/rete/float, API mutazione | **VERDE** | grep vuoto; API `Log` solo append |
| 7 | Esistenza test replay/immutabilità/no-`seq` | **VERDE** | `tests/l0_event_log.rs:315`, `:286`, `:257` |
| 8 | Seeding splitmix64 ≠ numpy SeedSequence → rischio L6 | **NOTA** (fuori scope L6) | dichiarazione producer, `src/prng.rs:21-47` |

Nessuna voce ROSSA.

---

## 8. Note del producer riportate

- **Seeding:** il seeding del crate usa splitmix64 a due output consecutivi, **non** `numpy.random.SeedSequence`. Il cross-check numpy dichiarato riguardava solo la funzione di output, a parità di `state` e `increment` forzati. Poiché L6 fissa il corpus sui seed 1000–1099, la coerenza di L6 con numpy non è garantita da questo engine: se L6 assume lo stesso stream di numpy per gli stessi seed, servirà una decisione esplicita. **Voce NOTA, non rossa** (L6 fuori scope; la SPEC L0 non fissa la procedura di espansione del seed).
- **Cross-check numpy:** non presente in repo, non ripetibile in CI. I test committati sono property-based. Registrato come limite di prova, non come violazione.

