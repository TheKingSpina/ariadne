# L0 — Report di verifica (Tester)

- **SPEC di riferimento:** `spec/L0_SPEC_v0.2.md` §2 (Formato evento), §3 (Append-only e correzioni), genesi (§4 "Schema PRNG")
- **Ruolo:** Tester (gate tecnico). **Nessuna accettazione dichiarata**: la firma è umana.
- **Data verifica:** 2026-10-22
- **Commit esaminati:**
  - `8d71937` — "L0: implementazione SPEC v0.2 §2-§3 + genesi — test a VERDE" (HEAD esaminato)
  - `6ae990a` — "L0: test ROSSI per SPEC v0.2 §2-§3 + genesi (solo test, nessun codice di produzione)" (primo commit del task)
- **Cartelle di lavoro:**
  - Working tree principale `/Users/spina/Ariadne`: **NON usata per i test** (contiene lavoro non committato: `M readme.md`, rename `spec/*`, `?? .opencode/`, `?? notes/`, `?? opencode.jsonc`).
  - Worktree gate (pulito): `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate` su `8d71937`
  - Worktree red: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-red` su `6ae990a`

## 0. Comandi eseguiti

```
git log --oneline -20
git status --short
git show --stat 6ae990a
git show --stat 8d71937
git show 6ae990a -- Cargo.toml
git diff 6ae990a 8d71937 -- tests/l0_event_log.rs

git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate 8d71937
git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-red 6ae990a

# nel worktree gate (pulito):
git status --short          -> nessun output (pulito)
git log --oneline -3        -> 8d71937 / 6ae990a / 79e106c
cargo test
cargo test --release

# nel worktree red:
cargo test                  -> ROSSO (E0432)
```

---

## 1. Prova test-first (ordine dei commit)

`git show --stat 6ae990a`:

```
commit 6ae990a8785890ec53654c95c50ceb7056698b60
Author: Alessandro Spinabelli <Alessandro.spinabelli@studenti.unipr.it>
Date:   Thu Oct 22 16:50:47 2026 +0200

    L0: test ROSSI per SPEC v0.2 §2-§3 + genesi (solo test, nessun codice di produzione)
    ...
 Cargo.lock            |  93 +++++++++++++++
 Cargo.toml            |  15 +++
 tests/l0_event_log.rs | 324 ++++++++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 432 insertions(+)
```

- **Nessun file in `src/`** nel primo commit: toccati solo `Cargo.toml`, `Cargo.lock`, `tests/l0_event_log.rs`. → **test-first rispettato**.
- `Cargo.toml` in `6ae990a` dichiara solo `[package] name = "ariadne"`, `[dev-dependencies] sha2 = "0.10"`, `[[test]] name = "l0_event_log"`: **nessun `[lib]`/nessun `src/`**, quindi il crate non esiste.

### 1.1 Rosso in `6ae990a` — output INTEGRALE

```
$ cargo test          # worktree /private/var/folders/.../opencode/ariadne-l0-red (HEAD = 6ae990a)
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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-red)
error[E0432]: unresolved import `ariadne`
  --> tests/l0_event_log.rs:17:5
   |
17 | use ariadne::{EventType, Log, Payload, Value};
   |     ^^^^^^^ use of unresolved module or unlinked crate `ariadne`
   |
   = help: if you wanted to use a crate named `ariadne`, use `cargo add ariadne` to add it to your `Cargo.toml`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `ariadne` (test "l0_event_log") due to 1 previous error

Exited with code 101
```

**Motivo del rosso corretto**: crate `ariadne` inesistente (nessuna lib) → errore di compilazione `E0432`, **non** asserzioni deliberatamente false. La specifica del task ("motivo: crate inesistente, non test asseriti-falsi") è rispettata.

### 1.2 Le asserzioni del commit di test non sono state alterate in `8d71937`

`git diff 6ae990a 8d71937 -- tests/l0_event_log.rs` — diff INTEGRALE:

```
diff --git a/tests/l0_event_log.rs b/tests/l0_event_log.rs
index 44cb26a..12c5de6 100644
--- a/tests/l0_event_log.rs
+++ b/tests/l0_event_log.rs
@@ -151,10 +151,10 @@ fn test_payload_canonical_sorted_keys_minimal() {
     // chiavi inserite in ordine che NON è alfabetico: l'output deve ordinare
     let p = Payload::from_pairs(vec![
         ("zeta", Value::Int(-3)),
-        ("alpha", Value::Str("s")),
+        ("alpha", Value::Str("s".to_string())),
         ("mik", Value::Int(7)),
         ("compensates", Value::Int(1)),
-        ("reason", Value::Str("lo scoglio")),
+        ("reason", Value::Str("lo scoglio".to_string())),
     ]);
     assert_eq!(
         p.canonical(),
@@ -259,7 +259,7 @@ fn test_replay_prefix_byte_per_byte() {
     let build = || {
         let mut log = Log::genesis(1000);
         log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(42))])).unwrap();
-        log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("V"))])).unwrap();
+        log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("V".to_string()))])).unwrap();
         log.append(2, EventType::State, Payload::from_pairs(vec![("gain", Value::Int(6))])).unwrap();
         log
     };
```

Le uniche modifiche sono **3 token** di coercizione sintattica (`Value::Str(String)` non accetta `&str`): nessuna modifica semantica ad asserzioni, preimage, golden o soglie.

---

## 2. `cargo test` nel worktree pulito — output INTEGRALE

Worktree: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate` (HEAD `8d71937`, `git status --short` → nessun output).

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.35s
     Running unittests src/lib.rs (target/debug/deps/ariadne-f4839cd99eb39cde)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test log::tests::genesis_line_parses ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-d009f4ce22e7032a)

running 13 tests
test test_event_without_seq_is_rejected ... ok
test test_genesis_golden_hash_constant ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_immutable_events_slice_borrows ... ok
test test_hash_chain_end_to_end ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Totale: 17/17 verdi** (4 unit + 13 integrazione), 0 falliti, 0 ignorati.

## 3. `cargo test --release` — output INTEGRALE

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate)
    Finished `release` profile [optimized] target(s) in 3.56s
     Running unittests src/lib.rs (target/release/deps/ariadne-e38db9ac10129f4e)

running 4 tests
test event::tests::compute_hash_matches_documented_preimage ... ok
test event::tests::event_type_parse_is_total_and_strict ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok
test log::tests::genesis_line_parses ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/release/deps/l0_event_log-394ff1683e79bef1)

running 13 tests
test test_event_without_seq_is_rejected ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_genesis_golden_hash_constant ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_immutable_events_slice_borrows ... ok
test test_hash_chain_end_to_end ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_unknown_top_level_member_rejected ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok
test test_replay_prefix_byte_per_byte ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Totale: 17/17 verdi** in release. Comportamento identico fra debug e release.

---

## 4. Checklist — esito punto per punto

| # | Voce di checklist (task + SPEC §5) | Esito | Prova (file:riga) |
|---|---|---|---|
| 1 | **Ordine test-first**: il PRIMO commit contiene SOLO test (+ Cargo.toml/Cargo.lock), nessun `src/` | **VERDE** | `git show --stat 6ae990a` → `Cargo.lock`, `Cargo.toml`, `tests/l0_event_log.rs`; zero file in `src/` |
| 2 | I test del primo commit erano **ROSSI** (motivo: crate inesistente, non asserzioni false) | **VERDE** | worktree red `cargo test` → `error[E0432]: unresolved import \`ariadne\`` (`tests/l0_event_log.rs:17`), exit 101 (§1.1) |
| 3 | Le asserzioni non sono state addolcite nel commit di implementazione | **VERDE** | `git diff 6ae990a 8d71937 -- tests/l0_event_log.rs` = 3 token `.to_string()` (§1.2) |
| 4 | **Catena di hash verificabile end-to-end** | **VERDE** | `tests/l0_event_log.rs:105-122` (`test_hash_chain_end_to_end`): ricalcolo SHA-256 indipendente col crate `sha2` (`:31-40`), preimage ricostruito nel test (`:43-55`), assert su `seq` posizionale, aggancio `prev_hash`, hash ricalcolato, esadecimale minuscolo. Implementazione: `src/event.rs:95-113` (`compute_hash`), `src/log.rs:108-133` (`verify`) |
| 5 | **Immutabilità**: un evento vecchio modificato fa fallire la verifica | **VERDE** | `tests/l0_event_log.rs:227-254` (`test_log_is_append_only_no_mutation_truncation_or_reorder`): 3 tampering distinti (payload vecchio `:236-238`, hash vecchio `:241-244`, prev_hash vecchio `:247-250`) → `verify().is_err()`; più `tests/l0_event_log.rs:312-324` (`test_immutable_events_slice_borrows`): prefisso intatto dopo append, `events()` non mutabile (`src/log.rs:101-103`) |
| 6 | **Evento senza `seq` rifiutato** | **VERDE** | `tests/l0_event_log.rs:198-206` (`test_event_without_seq_is_rejected`): membro `"seq"` assente → `from_jsonl` errore; `"seq":` senza valore → errore. Codice: `src/log.rs:286-298` (membri mancanti → `Error::Schema`) |
| 7 | **`seq` monotoni da 0 senza gap** | **VERDE** | `tests/l0_event_log.rs:208-221` (`test_seq_monotonic_from_zero_no_gaps`): gap (`"seq":2`→`"seq":7`) → `verify()` errore; partenza da seq 9 → errore. Codice: `src/log.rs:110-116` (`e.seq != i as u64` → `Error::Chain`), `src/log.rs:68-77` (`append` = ultimo+1) |
| 8 | **Correzione compensativa `{compensates, reason}` con storia intatta** | **VERDE** | `tests/l0_event_log.rs:284-310` (`test_compensating_event_payload_and_history_intact`): payload canonico esatto `{"compensates":1,"reason":"..."}`, prefisso storico invariato (`after.starts_with(prefix_before)`), evento compensato ancora presente (`:304`), `verify()` verde, `compensate(999,…)` → errore. Codice: `src/log.rs:83-98` |
| 9 | **`prev_hash` = 64 zeri per la genesi** | **VERDE** | `tests/l0_event_log.rs:72-89` (`test_genesis_prev_hash_zeroes_and_shape`), costante `ZERO64` a `:19`; codice `src/event.rs:24` (`GENESIS_PREV_HASH`), `src/log.rs:56-64` (genesi con quel prev_hash), verifica in `src/log.rs:109` |
| 10 | **Payload JSON canonico** (chiavi ordinate, solo interi e stringhe) | **VERDE** | `tests/l0_event_log.rs:149-165` (`test_payload_canonical_sorted_keys_minimal`): input in ordine NON alfabetico → output `{"alpha":…,"compensates":…,"mik":…,"reason":…,"zeta":-3}`; `tests/l0_event_log.rs:167-182` (`test_parser_rejects_non_canonical_payload_values`): float/bool/null/array/object annidato/chiave duplicata rifiutati. Codice: `src/payload.rs:84-104` (BTreeMap → ordine alfabetico), `src/log.rs:303-333` (dominio dei valori), `src/log.rs:346-362` (`parse_i64` senza float) |
| 11 | **Replay di un prefisso byte-per-byte** | **VERDE** | `tests/l0_event_log.rs:256-278` (`test_replay_prefix_byte_per_byte`): due build indipendenti → `assert_eq!(jsonl_a.as_bytes(), jsonl_b.as_bytes())`; round-trip `from_jsonl`→`to_jsonl` byte-identico; `verify()` verde. Codice: `src/log.rs:136-143` (`to_jsonl`), `src/log.rs:148-168` (`from_jsonl`) |
| 12 | **Replay bit-perfetto su due esecuzioni indipendenti** (SPEC §5) | **VERDE** | `tests/l0_event_log.rs:266-268` (due costruzioni indipendenti, byte-per-byte) + `cargo test` e `cargo test --release` eseguiti in due processi separati sullo stesso worktree con esito identico 17/17 (§2, §3) |
| 13 | **Genesi**: seq 0, tick 0, type STATE, payload `{seed, prng:"PCG64", state}` | **VERDE** | `tests/l0_event_log.rs:74-84` (seq 0, tick 0, `STATE`, payload `{"prng":"PCG64","seed":1042,"state":1042}`); codice `src/log.rs:56-64` |
| 14 | **Format evento** `{seq, tick, type, payload, prev_hash, hash}` — membri esatti, duplicati vietati, membri sconosciuti vietati | **VERDE** | `tests/l0_event_log.rs:138-143` (`test_unknown_top_level_member_rejected`: `"ts":""` iniettato → errore); `tests/l0_event_log.rs:125-135` (type fuori dominio rifiutato); codice `src/log.rs:234-299` (schema rigorosa, `set_slot!` duplicati `:224-231`, membro non previsto `:279`) |
| 15 | **Zero wall-clock, I/O esterna, rete, float nel codice L0** | **VERDE** | Lettura INTEGRALE di `src/lib.rs` (23 righe), `src/event.rs` (160), `src/log.rs` (497), `src/payload.rs` (130): nessun uso di `SystemTime`/`Instant`/`std::time`, `std::fs`/`std::io`, `std::net`, nessun `f32`/`f64`/`float`; unica dipendenza `sha2` (`src/event.rs:18`); unici `std` usati: `std::collections::BTreeMap` (`src/payload.rs:12`), `std::fmt` (`src/event.rs:19`, `src/log.rs:20`). `tick` è un contatore logico (`src/log.rs:68`), mai un timestamp. Test: nessun membro "time"/"ts" ammesso (`tests/l0_event_log.rs:138-143`, `:276-277`) |
| 16 | **Nessuna API di update/delete/riscrittura/riordino del log** | **VERDE** | Superficie pubblica di `Log` in `src/log.rs:53-168`: `genesis` (`:56`), `append` (`:68`), `compensate` (`:83`), `events` (`:101`, restituisce `&[Event]`), `verify` (`:108`), `to_jsonl` (`:136`), `from_jsonl` (`:148`). Nessun `remove`/`delete`/`update`/`truncate`/`reorder`/`set`/`clear`; `Event` ha campi pubblici in sola lettura e costruzione solo via `Event::seal` crate-internal (`src/event.rs:81-91`) |
| 17 | **Correzione compensativa testata con successo** (SPEC §5) | **VERDE** | `tests/l0_event_log.rs:284-310` + `tests/l0_event_log.rs:312-324`; entrambe verdi in §2 e §3 |
| 18 | **Zero mutazioni retroattive dimostrate** (SPEC §5) | **VERDE** | `tests/l0_event_log.rs:227-254` (tampering di evento vecchio → catena non verificabile) + `src/log.rs:16-18` (separazione schema/chain: `from_jsonl` non verifica, `verify` sì — la separazione che rende i tampering significativi) |
| 19 | **`hash = SHA-256(seq ‖ tick ‖ type ‖ payload ‖ prev_hash)`** — preimage documentato e senza ambiguità | **VERDE** | `src/event.rs:11-16` (preimage con separatore 0x1F), `src/event.rs:95-113`; test indipendente sul golden `src/event.rs:128-149` e `tests/l0_event_log.rs:115-118` |
| 20 | **Golden hash della genesi (`e73ef6c4…1e97`) verificato da un'implementazione SHA-256 TERZA** | **ROSSA / APERTA** | Il golden è presente **già in `6ae990a`** (commit di test, prima dell'implementazione) — `tests/l0_event_log.rs:23` — ed è coerente con la riga canonica `tests/l0_event_log.rs:25-29`. **Non è però stato confermato da un'implementazione esterna al crate**: nel sandbox di verifica sono state negate `shasum`, `openssl`, `python3`, `printf | shasum`, e non erano disponibili `crypto.subtle` (runtime JS) né browser. La ricalcolazione effettuata nei test usa la stessa libreria `sha2` del codice di produzione, quindi **non è un controllo indipendente**. Vedi §6 per il comando di chiusura. |

### 4.1 Riepilogo esiti

- **VERDI: 19** (voci 1–19)
- **ROSSA/APERTA: 1** (voce 20 — golden hash non verificato da terza implementazione)
- Discrepanze SPEC↔codice: 1 (§7)

---

## 5. Ispezione dei divieti — metodo ed esito

**Metodo:** lettura integrale (non grep, non sampling) dei quattro file di produzione del worktree gate:
`src/lib.rs` (23 righe), `src/event.rs` (160), `src/log.rs` (497), `src/payload.rs` (130); inoltre lettura integrale di `tests/l0_event_log.rs` (324) e `src/log.rs` (parser, 187-462).

| Divieto | Esito | Metodo / prova |
|---|---|---|
| **Wall-clock** | ASSENTE | Nessun `SystemTime`, `Instant`, `std::time`, `Date`, o costruzione timestamp in `src/`. `tick: u64` è un contatore logico passato esplicitamente ad `append` (`src/log.rs:68`); la genesi usa `tick 0` (`src/log.rs:62`). Lo schema evento non prevede alcun campo timestamp: qualunque membro extra è rifiutato (`src/log.rs:279`), test `tests/l0_event_log.rs:138-143` ("zero wall-clock: la schema evento non ha spazio per un timestamp di nessun tipo"). |
| **I/O esterna** | ASSENTE | Nessun `std::fs`, `std::io`, `File`, `Read`/`Write`, `println!`/`eprintln!` in `src/`. Il crate è puro: `Log` mantiene `events: Vec<Event>` in memoria (`src/log.rs:48-51`); la serializzazione è `String` (`src/log.rs:136-143`, `:173-185`) e il parsing è su `&str` (`src/log.rs:148`). Nessun `main`, nessun binario (`src/lib.rs` dichiara solo `mod` + `pub use`). |
| **Rete** | ASSENTE | Nessun `std::net`, `TcpStream`, `UdpSocket`, `reqwest`, `curl` in `src/`. Unica dipendenza: `sha2` (`src/event.rs:18`), di tipo hashing locale. |
| **Float** | ASSENTE | Nessun `f32`/`f64`/`float` in `src/`. Il dominio dei valori payload è `Value::Int(i64) | Value::Str(String)` (`src/payload.rs:17-20`); il parser accetta solo interi o stringhe (`src/log.rs:318-326`) e rifiuta esplicitamente float/bool/null/array/object (`src/log.rs:322-324`), test `tests/l0_event_log.rs:167-182`. |
| **API di update / delete / riscrittura / riordino** | ASSENTE | Superficie pubblica di `Log` (`src/log.rs:53-168`) limitata a `genesis`, `append`, `compensate`, `events` (sola lettura, `&[Event]`), `verify`, `to_jsonl`, `from_jsonl`. `append` (`src/log.rs:68-77`) accoda in coda con `seq = ultimo+1` e `prev_hash = hash di testa`; `compensate` (`src/log.rs:83-98`) accoda in coda; `Event` non ha costruttore pubblico (solo `Event::seal`, `pub(crate)`, `src/event.rs:81`). Nessun metodo che modifichi, rimuova o riordini eventi esistenti. |

---

## 6. Voce ROSSA / APERTA — golden hash della genesi non verificato da terza implementazione

**Stato:** il valore atteso `e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97` (`tests/l0_event_log.rs:23`) è fissato nel **commit di test `6ae990a`**, cioè **prima** dell'esistenza del codice di produzione (la riga canonica corrispondente è `tests/l0_event_log.rs:25-29`). Questo è un forte indizio di precomputo esterno, ma **non è una prova indipendente**.

**Perché non chiusa:** il sandbox di verifica ha **negato** i comandi necessari a un terzo calcolatore SHA-256 (`shasum`, `openssl`, `python3`, `printf ... | shasum -a 256`), il runtime JS di `execute` non espone `crypto.subtle` né `Uint32Array` (implementazione manuale di SHA-256 non eseguibile), e non era connesso alcun browser. La ricalcolazione effettuata dai test (`tests/l0_event_log.rs:31-40`, `:115-118`) usa la **stessa libreria `sha2`** del codice di produzione: è un controllo di auto-coerenza, non di terza parte.

**Comando esatto che un umano deve eseguire per chiudere la voce** (attesa: `e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97`):

```sh
printf '%s' $'0\x1f0\x1fSTATE\x1f{"prng":"PCG64","seed":1042,"state":1042}\x1f0000000000000000000000000000000000000000000000000000000000000000' | shasum -a 256
```

Equivalente in Python (nessuna dipendenza esterna):

```sh
python3 -c "import hashlib;print(hashlib.sha256(b'0\x1f0\x1fSTATE\x1f{\"prng\":\"PCG64\",\"seed\":1042,\"state\":1042}\x1f'+b'0'*64).hexdigest())"
```

Se l'hash restituito è `e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97`, la voce 20 passa a **VERDE**; in caso contrario, la catena di hash dell'intero L0 è da rivedere.

---

## 7. Discrepanze SPEC ↔ codice

| # | SPEC | Codice | Valutazione |
|---|---|---|---|
| D1 | §"Schema PRNG": "Evento genesi (seq 0, STATE): payload {seed, prng: \"PCG64\", **state: iniziale esplicito**}" — lo stato del PRNG PCG64 dovrebbe essere lo stato iniziale *reale* del generatore, non il seed | `src/log.rs:56-64`: `("state", Value::Int(seed as i64))` → `state = seed` (1042 → `"state":1042`), documentato in `src/log.rs:9-12` come scelta di governance "finché l'engine PCG64 non è in scope, lo stato iniziale esplicito coincide col seed; nessun calcolo PRNG in L0 §2-§3". Test che lo fissano: `tests/l0_event_log.rs:82-84`, golden `tests/l0_event_log.rs:25-29`. | **Scostamento dichiarato e circoscritto.** Il dispatch di questo livello copre §2, §3 e la genesi *come schema*; l'engine PCG64 è fuori scope (`src/lib.rs:11-12`: "NON è in scope qui … PRNG PCG64 completo, fold dello stato"). Da rivedere al livello che implementerà il PRNG: se lo stato iniziale reale del PCG64 (seed 1042 → stato a 128 bit) non coinciderà col seed, la SPEC dovrà essere aggiornata **prima** che i golden di `tests/l0_event_log.rs:23` e `:25-29` congelino un valore non conforme. |

Altre osservazioni **senza discrepanza**:
- La SPEC §2 scrive `hash = SHA-256(seq ‖ tick ‖ type ‖ payload ‖ prev_hash)` senza specificare l'encoding della concatenazione. L'implementazione fissa un separatore `0x1F` (`src/event.rs:12`, `src/event.rs:27`, `src/event.rs:95-113`), dichiarato e motivato in `src/event.rs:11-16` (il carattere 0x1F non compare in nessun campo, quindi il preimage è univoco). È una scelta implementativa legittima, resa esplicita nei test (`tests/l0_event_log.rs:8-12`, `tests/l0_event_log.rs:43-55`).
- La SPEC §4 chiede che il replay verifichi **per confronto** lo snapshot di stato invece di fidarsi della riesecuzione: la genesi include già `state` nel payload (`src/log.rs:57-61`), ma il confronto vero e proprio richiede il ciclo senso→azione→stato e il fold, dichiarati fuori scope (`src/lib.rs:11-12`).

---

## 8. Cleanup

Worktree temporanei rimossi al termine della verifica:

```
git worktree remove /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate
git worktree remove /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-red
# il secondo ha richiesto --force: `git status --short` nel worktree red riportava
# solo `?? target/` (artefatti di compilazione), nessuna modifica sorgente:
git worktree remove --force /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-red
```

Verifica finale: `git worktree list` → `/Users/spina/Ariadne  8d71937 [main]` (nessun worktree temporaneo residuo).

Nessuna modifica a `src/` o `tests/` è stata effettuata dal Tester. Nessun file creato dal Tester fuori da `reports/`.

---

**Nota finale:** questo report è una **gate tecnico**. Non costituisce accettazione: la firma è umana.
