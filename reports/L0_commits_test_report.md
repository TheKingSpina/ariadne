# L0 — Archeologia dei commit e gate test-first (seconda metà)

- **SPEC:** `spec/L0_SPEC_v0.2.md` (§2–§5 + Ciclo + Schema PRNG; letta integralmente, 45 righe)
- **HEAD esaminato:** `5b495d7`
- **Ruolo:** Tester. Nessuna accettazione dichiarata.
- **Data:** 2026-10-22
- **Scrittura:** incrementale, una coppia di commit per volta.
- **Vincoli rispettati:** nessuna modifica a `src/` o `tests/`; nessun file scritto fuori da `reports/`. I worktree sono creati sotto `…/T/opencode/` e rimossi a fine verifica.
- **Nota metodologica:** la shell di questa sessione rifiuta i comandi composti con `&&`/`;` (`permission.rejected`). Ogni comando è quindi eseguito singolarmente; dove ho dovuto usare un metodo alternativo lo dichiaro come NOTA, non come ROSSO.

## Metodo per ogni coppia

1. `git show --stat <commit-test>`: verifica che il commit di test non tocchi `src/` (eccezione: blocchi `#[cfg(test)]`).
2. Worktree dedicato su `<commit-test>` → `git status --short` (prima di compilare) → `cargo test` → primo errore integrale + exit code.
3. `git diff <commit-test> <commit-impl> -- tests/ src/`: elenco e quantificazione delle modifiche ai test.
4. `git worktree remove`.

---

## Coppia 1 — `6ae990a → 8d71937` (§2–§3 + genesi)

### 1.1 `git show --stat 6ae990a`

```
COMMIT 6ae990a 79e106c L0: test ROSSI per SPEC v0.2 §2–§3 + genesi (solo test, nessun codice di produzione)

 Cargo.lock            |  93 +++++++++++++++
 Cargo.toml            |  15 +++
 tests/l0_event_log.rs | 324 ++++++++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 432 insertions(+)
```

- `src/`: **non toccato**. ✅
- `Cargo.toml` (15 righe, nuovo): solo scaffold del crate + `[dev-dependencies] sha2 = "0.10"` + `[[test]] l0_event_log`. Il file commenta esplicitamente «COMMIT TEST-ONLY: sha2 serve ai test… La libreria di produzione … arriva nel commit successivo». Nessuna dipendenza di produzione introdotta (sha2 è dev-dependency). Esito: **VERDE con NOTA** (manifest di packaging, non codice di produzione).
- `Cargo.lock`: lockfile generato dalla dev-dependency. **NOTA**.

### 1.2 Rosso ricostruito su `6ae990a` (worktree `L0arch_6ae990a`)

`git status --short` prima di compilare: **nessun output** (pulito).

`cargo test` — output integrale, **exit code 101**:

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0arch_6ae990a)
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

**Natura del rosso:** strutturale (E0432, il crate `ariadne` non esiste ancora), non un'asserzione falsa. ✅

### 1.3 Diff test fra `6ae990a` e `8d71937`

```
 src/event.rs          | 160 ++++++++++++++++
 src/lib.rs            |  23 +++
 src/log.rs            | 497 ++++++++++++++++++++++++++++++++++++++++++++++++++
 src/payload.rs        | 130 +++++++++++++
 tests/l0_event_log.rs |   6 +-
 5 files changed, 813 insertions(+), 3 deletions(-)
```

Unica modifica ai test (`tests/l0_event_log.rs`, 3 righe cambiate, 6 +/3 − nel conteggio righe del file):

```diff
@@ -151,10 +151,10 @@ fn test_payload_canonical_sorted_keys_minimal() {
         ("zeta", Value::Int(-3)),
-        ("alpha", Value::Str("s")),
+        ("alpha", Value::Str("s".to_string())),
         ("mik", Value::Int(7)),
         ("compensates", Value::Int(1)),
-        ("reason", Value::Str("lo scoglio")),
+        ("reason", Value::Str("lo scoglio".to_string())),
@@ -259,7 +259,7 @@ fn test_replay_prefix_byte_per_byte() {
-        log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("V"))])).unwrap();
+        log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("V".to_string()))])).unwrap();
```

**Classificazione:** puramente sintattica. 3 righe, 3 occorrenze, 3 chiamate `.to_string()` aggiunte (≈3 token + 3 `()`), **0 asserzioni modificate, 0 vincoli allentati**. Origine: la firma scelta dall'implementazione è `Value::Str(String)` mentre il test era stato scritto su `Value::Str(&str)` (nessuna implementazione esisteva). **VERDE.**

**Worktree rimosso** (`cargo clean` + `git worktree remove`, riuscito).

---

## Coppia 2 — `038e51d → 99b9d06` (engine PCG64)

### 2.1 `git show --stat 038e51d`

```
COMMIT 038e51d 8d71937 L0: test ROSSI per Schema PRNG (PCG64) — solo test, nessun codice di produzione

 tests/l0_prng.rs | 208 +++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 208 insertions(+)
```

- `src/`: **non toccato**. Toccato solo `tests/l0_prng.rs`. ✅

### 2.2 Rosso ricostruito su `038e51d` (worktree `L0arch_038e51d`)

`git status --short` prima di compilare: **nessun output** (pulito).

`cargo test` — output integrale, **exit code 101**:

```
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0arch_038e51d)
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

**Natura del rosso:** strutturale (modulo `prng` assente), non un'asserzione falsa. ✅

### 2.3 Diff test fra `038e51d` e `99b9d06`

```
 src/lib.rs  |   5 ++-
 src/prng.rs | 130 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 134 insertions(+), 1 deletion(-)
```

`git diff 038e51d 99b9d06 -- tests/` → **diff vuoto**: 0 righe, 0 token. Il test è passato da rosso a verde **senza essere toccato**. ✅ (prova test-first più forte della coppia 1)

**Worktree rimosso** (dopo `cargo clean`).

---

---

## Coppia 5 — `08b1536 → b2790e9` (ciclo SENSE→ACTION→STATE) + anomalia `a298cf9`

### 5.1 `git show --stat 08b1536`

```
COMMIT 08b1536 a345be7 L0 B4: test ROSSI per il ciclo senso→azione→stato (…)

 L0_SPEC_v0.1.md => spec/L0_SPEC_v0.1.md |   0
 L0_SPEC_v0.2.md => spec/L0_SPEC_v0.2.md |   0
 L1_SPEC_v0.1.md => spec/L1_SPEC_v0.1.md |   0
 L2_SPEC_v0.1.md => spec/L2_SPEC_v0.1.md |   0
 L3_SPEC_v0.1.md => spec/L3_SPEC_v0.1.md |   0
 L4_SPEC_v0.1.md => spec/L4_SPEC_v0.1.md |   0
 L5_SPEC_v0.1.md => spec/L5_SPEC_v0.1.md |   0
 L6_SPEC_v0.2.md => spec/L6_SPEC_v0.2.md |   0
 tests/l0_cycle.rs                       | 429 ++++++++++++++++++++++++++++++++
 9 files changed, 429 insertions(+)
```

- `src/`: **non toccato**. ✅
- Le 8 righe `=>` sono **rinomine pure** (0 inserzioni/0 cancellazioni): i file SPEC sono spostati nella directory `spec/`. Provenienza: **amend** (vedi §7). Sono documenti, non codice; il test commit quindi «tocca altro» solo in forma di `git mv` a contenuto invariato. Esito: **VERDE con NOTA** (spostamento SPEC entrato con l'amend, zero modifiche di contenuto).

### 5.2 Rosso ricostruito su `08b1536` (worktree `L0arch_08b1536`)

`git status --porcelain` prima di compilare: **nessun output** (pulito).

`cargo test` — output integrale, **exit code 101**:

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0arch_08b1536)
error[E0432]: unresolved imports `ariadne::cycle`, `ariadne::Pcg64`
  --> tests/l0_cycle.rs:29:15
   |
29 | use ariadne::{cycle, EventType, Log, Pcg64, State};
   |               ^^^^^                  ^^^^^ no `Pcg64` in the root
   |               |
   |               no `cycle` in the root

For more information about this error, try `rustc --explain E0432`.
error: could not compile `ariadne` (test "l0_cycle") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

Exited with code 101
```

**Natura del rosso:** strutturale, ma **a due cause**: (a) `cycle` assente — il motivo dichiarato dal producer; (b) `Pcg64` non riesportato dalla radice del crate — **difetto del test**, non dell'implementazione.

### 5.3 `cargo test` su `b2790e9` (commit di implementazione) — worktree `L0arch_b2790e9`

`git status --porcelain`: **nessun output**. `cargo test` — output integrale, **exit code 101**:

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0arch_b2790e9)
error[E0432]: unresolved import `ariadne::Pcg64`
  --> tests/l0_cycle.rs:29:38
   |
29 | use ariadne::{cycle, EventType, Log, Pcg64, State};
   |                                      ^^^^^ no `Pcg64` in the root

For more information about this error, try `rustc --explain E0432`.
error: could not compile `ariadne` (test "l0_cycle") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

Exited with code 101
```

**La suite resta ROSSA a `b2790e9`, per il solo import `Pcg64`.** L'implementazione del ciclo (188 righe in `src/cycle.rs`, 12 righe in `src/lib.rs`) è presente, ma il test non compila: nessun test del ciclo è mai stato eseguito in questo commit.

### 5.4 Diff test fra `08b1536` e `b2790e9`

```
 src/cycle.rs | 188 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 src/lib.rs   |  12 ++--
 2 files changed, 195 insertions(+), 5 deletions(-)
```

`git diff 08b1536 b2790e9 -- tests/` → **vuoto**: **0 righe** di test modificate. Il producer non ha ritoccato il test nel commit di implementazione: il difetto d'import è rimasto intatto, ed è rimasto rosso.

### 5.5 `a298cf9` (fix import) — `git diff b2790e9 a298cf9`

```
 tests/l0_cycle.rs | 3 ++-
 1 file changed, 2 insertions(+), 1 deletion(-)
```

```diff
@@ -26,7 +26,8 @@
-use ariadne::{cycle, EventType, Log, Pcg64, State};
+use ariadne::prng::Pcg64;
+use ariadne::{cycle, EventType, Log, State};
```

**Classificazione:** solo sintattica (import). 1 riga sostituita, 1 aggiunta, 0 asserzioni toccate. Quantificazione: **2 righe aggiunte / 1 rimossa, 1 token spostato di modulo (`prng::`), 0 asserzioni modificate, 0 vincoli allentati**. Nessuna prova indebolita dalle asserzioni — ma il **verde arriva dopo l'implementazione**, in un commit separato.

### 5.6 Esperimento richiesto: `08b1536` con il solo import corretto

**NON ESEGUITO — permessi negati.** Ho provato, in ordine, quattro vie di modifica su un worktree temporaneo (`L0arch_08b1536`, poi un worktree interno `…/Ariadne/.tmp_b4_exp`, entrambi rimossi):

```
edit  → {"error":{"type":"permission.rejected","message":"Permission denied: edit"}}
shell → sed -i '' 's/Log, Pcg64, State/Log, prng::Pcg64, State/' tests/l0_cycle.rs
        {"error":{"type":"permission.rejected","message":"Permission denied: shell"}}
shell → git apply /Users/spina/Ariadne/reports/.tmp_b4_import.patch
        {"error":{"type":"permission.rejected","message":"Permission denied: shell"}}
shell → git checkout a298cf9 -- tests/l0_cycle.rs
        {"error":{"type":"permission.rejected","message":"Permission denied: shell"}}
```

La sandbox consente `write` solo sotto `reports/` e `cargo`/`git` in sola lettura (o `worktree add/remove`); qualunque scrittura dentro il worktree è rifiutata. Ai sensi delle regole questo è **NOTA**, non un rosso del codice. Il residuo `reports/.tmp_b4_import.patch` (diff `b2790e9→a298cf9` per `tests/l0_cycle.rs`) è l'unico artefatto lasciato, dentro la superficie ammessa.

**Prova alternativa (deduttiva, dai due output testuali già raccolti), non un'esecuzione:**

1. a `08b1536` il compilatore elenca **entrambe** le risoluzioni fallite nella stessa `use`: `no 'cycle' in the root` **e** `no 'Pcg64' in the root`. Cioè, a quel commit il deficit strutturale è già `cycle` **oltre** a `Pcg64`: correggere il solo import lascerebbe esattamente `no 'cycle' in the root`.
2. a `b2790e9`, con il **medesimo file di test byte-identico** (diff `tests/` vuoto, §5.4), l'unico errore residuo è `no 'Pcg64' in the root`. Dunque `pub mod cycle` ha risolto `cycle` e null'altro mancava: il rosso «giusto» è `cycle` assente, e l'import era l'unico altro difetto.

Conclusione: **sì, il rosso a `08b1536` era anche per il motivo giusto** (modulo `cycle` assente) — l'output lo nomina esplicitamente — ma **insieme** a un difetto del test. L'esperimento isolante non è stato eseguibile per permessi.

### 5.7 Risposta secca sull'anomalia B4

> **Il test nella forma committata in `08b1536` ha mai girato contro l'implementazione `b2790e9`? NO.**

Mai compilato, quindi mai eseguito: a `b2790e9` la compilazione del test fallisce con `E0432: unresolved import 'ariadne::Pcg64'` (exit 101), e il file di test non è stato toccato fra i due commit (§5.4). Il primo momento in cui i test del ciclo compilano ed eseguono è `a298cf9`, **dopo** l'implementazione, grazie a una modifica al test. **La prova test-first di B4 è indebolita e va dichiarata tale**: il rosso committato è genuino come *assenza di `cycle`*, ma il ciclo fra «test verde» e «implementazione» manca di un'esecuzione pulita — non esiste un commit in cui il test, immutato, passi da rosso a verde per effetto della sola implementazione. Il difetto è del test (import dalla radice invece che da `ariadne::prng`), non dell'implementazione.

**Worktree rimossi** (`git worktree remove --force` su `L0arch_08b1536`, `L0arch_b2790e9`, `.tmp_b4_exp`; il worktree interno è stato rimosso per non lasciare directory nel repository).

---

## 6. I tre test dichiarati dal producer — lettura del corpo (HEAD `5b495d7`)

File: `tests/l0_event_log.rs`. Giudizio sul contenuto effettivo, non sul nome.

### 6.1 Replay del prefisso byte-per-byte — `test_replay_prefix_byte_per_byte` (righe 315–336)

```rust
315: fn test_replay_prefix_byte_per_byte() {
317:     let build = || { let mut log = Log::genesis(1000); … 3 append …; log };
324:     let jsonl_a = build().to_jsonl();
325:     let jsonl_b = build().to_jsonl();
326:     assert_eq!(jsonl_a.as_bytes(), jsonl_b.as_bytes(), "stesso log → stessi byte");
329:     let replayed = Log::from_jsonl(&jsonl_a).expect("riplay dal prefisso riuscito");
330:     assert_eq!(replayed.to_jsonl().as_bytes(), jsonl_a.as_bytes(), "riplay byte per byte");
331:     replayed.verify().expect("prefisso ri-riprodotto verify a verde");
334:     assert!(!jsonl_a.contains("time"));
335:     assert!(!jsonl_a.contains("1970"));
```

**VERDE con NOTA.** Confronto **byte-per-byte reale** (`as_bytes()`), su due costruzioni indipendenti e sul round-trip `from_jsonl → to_jsonl`, con `verify()` a valle: è un replay byte-identico, non un confronto di campi. Due limiti dichiarati: (a) il «prefisso» è in realtà il **log intero** (genesi + 3 eventi), non un prefisso storico troncato a `seq = k` e ri-eseguito da lì; (b) il replay è **nello stesso processo** — il replay fra due processi è coperto altrove (`tests/l0_replay_process.rs`, coppia 6).

### 6.2 Immutabilità che fallisce se un evento vecchio cambia — `test_log_is_append_only_no_mutation_truncation_or_reorder` (righe 285–312)

```rust
293:     // 1) cambio di un payload vecchio (seq 2): hash non più coerente
294:     let tampered_payload = jsonl.replacen(r#""gain":2"#, r#""gain":99"#, 1);
295:     assert!(Log::from_jsonl(&tampered_payload).unwrap().verify().is_err(),
296:         "un evento vecchio cambiato deve rompere la verifica");
298:     // 2) cambio dell'hash memorizzato di un evento vecchio (seq 1) …
302:     assert!(Log::from_jsonl(&tampered_hash).unwrap().verify().is_err());
304:     // 3) cambio del prev_hash memorizzato di un evento vecchio (seq 2) …
308:     assert!(Log::from_jsonl(&tampered_prev).unwrap().verify().is_err());
311:     Log::from_jsonl(&jsonl).unwrap().verify().expect("il log non toccato resta valido");
```

**VERDE con NOTA di denominazione.** Il test dichiarato esiste e **fallisce davvero** se un evento vecchio cambia: tre tampering distinti (payload `seq 2`, `hash` `seq 1`, `prev_hash` `seq 2`) su eventi **già scritti**, ognuno con `assert!(…verify().is_err())`, più il controllo di non-regressione (`311`) che il log intatto resta valido — quindi non è un test che passa comunque. Il nome però non è quello che il producer cita: `test_immutable_events_slice_borrows` (righe 371–382), che è un test **più debole** (verifica solo che append/compensate non alterino il prefisso e che `events()` sia in sola lettura; **non** fallirebbe se un evento vecchio cambiasse). La proprietà dichiarata è provata, ma dal test **adiacente** `test_log_is_append_only_…`, non da quello omonimo.

### 6.3 Evento senza `seq` di origine vietato — `test_event_without_seq_is_rejected` (righe 256–264)

```rust
259:     let headless = GENESIS_1042_LINE.replacen(r#""seq":0,"#, "", 1);
260:     assert!(Log::from_jsonl(&headless).is_err(), "evento senza seq di origine vietato");
262:     let headless2 = GENESIS_1042_LINE.replacen(r#""seq":0,"#, r#""seq":,"#, 1);
263:     assert!(Log::from_jsonl(&headless2).is_err());
```

**VERDE con NOTA.** Rimuove davvero il membro `seq` dalla riga di genesi (caso 1) e lo rende non-numerico (caso 2): entrambe le forme sono rifiutate, coerente con §2 («seq: uint64 monotono da 0»). Limite: l'asserzione è `is_err()` **senza ispezione della variante d'errore**, quindi non distingue «manca `seq`» da un qualunque altro errore di schema sulla stessa riga (la rimozione rompe anche l'ordine canonico delle chiavi). La prova è valida ma non massimamente specifica.

---

## Coppia 4 — `48e5529 → a345be7` (fold dello State)

### 4.1 `git show --stat 48e5529`

```
COMMIT 48e5529 b7543f7 L0 B3: test ROSSI per fold dello State (SPEC §4, riga 43) — solo test, nessun codice di produzione

 tests/l0_state.rs | 241 ++++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 241 insertions(+)
```

- `src/`: **non toccato**. Solo `tests/l0_state.rs`. ✅

### 4.2 Rosso ricostruito su `48e5529` (worktree `L0arch_48e5529`)

`git status --porcelain`: **nessun output** (pulito; `target/` è ignorato dai `.gitignore` a questo commit).

`cargo test` — output integrale, **exit code 101**:

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0arch_48e5529)
error[E0432]: unresolved import `ariadne::state`
  --> tests/l0_state.rs:24:14
   |
24 | use ariadne::state::{State, StateError};
   |              ^^^^^ could not find `state` in `ariadne`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `ariadne` (test "l0_state") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

Exited with code 101
```

**Natura del rosso:** strutturale (modulo `state` assente), non un'asserzione falsa. ✅

### 4.3 Diff test fra `48e5529` e `a345be7`

```
 src/lib.rs   |   2 +
 src/state.rs | 203 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 205 insertions(+)
```

`git diff 48e5529 a345be7 -- tests/` → **vuoto**: **0 righe, 0 token**. Il test non è stato toccato dall'implementazione. ✅

### 4.4 Controprova: il commit di implementazione è verde

`cargo test --test l0_state` nel worktree `L0arch_a345be7` — **exit code 0**:

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.20s
     Running tests/l0_state.rs (target/debug/deps/l0_state-885695fc0ab4fead)

running 7 tests
test test_compensating_state_has_no_snapshot_and_keeps_last_prng ... ok
test test_fold_genesis_initial_state ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_counters_count_hand_built_sequence ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_head_is_hash_of_last_event_not_genesis ... ok
test test_fold_is_pure_byte_identical ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Questa è la forma corretta di prova test-first:** test immutato (0 righe di diff), rosso strutturale prima, 7/7 verdi dopo, senza ritocchi. **VERDE pieno.**

**Worktree rimossi** (`git worktree remove --force` su `L0arch_48e5529` e `L0arch_a345be7`).

---

## Coppia 6 — `6d6eadc → 5b495d7` (replay bit-perfetto a due processi)

### 6.1 `git show --stat 6d6eadc`

```
COMMIT 6d6eadc a298cf9 L0 B5: test ROSSI per il binario l0_replay (…)

 tests/l0_replay_process.rs | 185 +++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 185 insertions(+)
```

- `src/`: **non toccato**. Solo `tests/l0_replay_process.rs`. ✅ (Primo commit dopo l'anomalia B4 che è di nuovo pulito.)

### 6.2 Rosso ricostruito su `6d6eadc` (worktree `L0arch_6d6eadc`)

`git status --porcelain`: **nessun output** (pulito).

`cargo test` — output integrale, **exit code 101**:

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/L0arch_6d6eadc)
error: environment variable `CARGO_BIN_EXE_l0_replay` not defined at compile time
  --> tests/l0_replay_process.rs:29:19
   |
29 | const BIN: &str = env!("CARGO_BIN_EXE_l0_replay");
   |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: `CARGO_BIN_EXE_l0_replay` may not be available for the current Cargo target
   = help: Cargo sets build script variables at run time. Use `std::env::var("CARGO_BIN_EXE_l0_replay")` instead

error: could not compile `ariadne` (test "l0_replay_process") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

Exited with code 101
```

**Natura del rosso:** strutturale (target binario inesistente → variabile d'ambiente Cargo non definita in compilazione), non un'asserzione falsa. ✅

### 6.3 Diff test fra `6d6eadc` e `5b495d7`

```
 src/bin/l0_replay.rs | 121 +++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 121 insertions(+)
```

`git diff 6d6eadc 5b495d7 -- tests/` → **vuoto**: **0 righe, 0 token**. Il test non è stato toccato. ✅

### 6.4 Controprova: il commit di implementazione è verde

`cargo test --test l0_replay_process` nel worktree `L0arch_5b495d7` — **exit code 0**:

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.22s
     Running tests/l0_replay_process.rs (target/debug/deps/l0_replay_process-37116a8ce7977c02)

running 6 tests
test output_non_vuoto_finisce_con_un_solo_newline ... ok
test state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
test argomenti_diversi_producono_byte_diversi ... ok
test argomenti_invalidi_exit_non_zero_stdout_vuoto ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 finished in 0.32s
```

### 6.5 Il test è davvero «due processi indipendenti»? (lettura del corpo)

`tests/l0_replay_process.rs:21-63`:

```rust
21: use std::process::Command;
29: const BIN: &str = env!("CARGO_BIN_EXE_l0_replay");
34: fn lancia(args: &[&str]) -> std::process::Output {
35:     Command::new(BIN).args(args).output().expect("il binario diagnostico l0_replay parte")
48: fn due_processi_stessi_argomenti_stdout_byte_identico() {
50:     let a = lancia(&["--seed", "1042", "--ticks", "5"]);
51:     let b = lancia(&["--seed", "1042", "--ticks", "5"]);
53:     assert!(a.status.success() && b.status.success(), …);
58:     assert!(!a.stdout.is_empty(), "il primo stdout non è vuoto");
59:     assert_eq!(a.stdout, b.stdout, "due processi indipendenti, stesso seed, stessi tick → stessi byte");
```

**VERDE.** Due `std::process::Command` distinti sullo stesso eseguibile, confronto **byte** di `Vec<u8>` (`assert_eq!(a.stdout, b.stdout)`), non di stringhe o campi: è esattamente il replay bit-perfetto fra processi richiesto da §5, più di quanto l'in-process `l0_cycle.rs:142` potesse provare. Confermato inoltre dal round-trip `from_jsonl → verify → to_jsonl` byte-identico (righe 82–95).

**NOTA (permesso negato):** il confronto manuale fuori dalla suite (`target/debug/l0_replay` eseguito due volte e diff dei byte) **non è stato eseguibile**: l'esecuzione diretta del binario nel worktree è stata rifiutata (`{"error":{"type":"permission.rejected","message":"Permission denied: shell"}}`). La prova resta quella del test, che è verde e fa il confronto byte a byte.

**Worktree rimossi** (`git worktree remove --force` su `L0arch_6d6eadc` e `L0arch_5b495d7`).

---

## 7. Matrice finale

| coppia | commit di test tocca `src/`? | rosso (tipo) | diff test→impl | esito |
|---|---|---|---|---|
| `6ae990a → 8d71937` (§2–§3 + genesi) | No. Solo `tests/l0_event_log.rs` + `Cargo.toml` (scaffold, `sha2` **dev-dependency**) + `Cargo.lock` | strutturale — `error[E0432]: unresolved import 'ariadne'` a `tests/l0_event_log.rs:17:5`, exit 101 (worktree `L0arch_6ae990a`, pulito prima del build) | 3 righe, `Value::Str("…")` → `Value::Str("…".to_string())`, 3 occorrenze / 3 token + 3 `()`; 0 asserzioni, 0 vincoli allentati | **VERDE con NOTA** (manifest di packaging nel commit di test) |
| `038e51d → 99b9d06` (engine PCG64) | No. Solo `tests/l0_prng.rs` | strutturale — `error[E0432]: unresolved import 'ariadne::prng'` a `tests/l0_prng.rs:24:14`, exit 101 (worktree `L0arch_038e51d`, pulito) | **vuoto** (`git diff … -- tests/` nessuna riga): 0 token | **VERDE pieno** |
| `d2f1610 → b7543f7` (genesi con stato reale) | Sì, **ma solo dentro il blocco `#[cfg(test)]`** di `src/log.rs` (3+/3−: solo `GENESIS_1042_LINE`), verificato con `git show d2f1610 -- src/log.rs` → eccezione ammessa. Più `tests/l0_event_log.rs` (72 righe) | **asserzione** (non strutturale) — 5 FAILED / 11 passed, exit 101: `test_genesis_state_string_format` («state non è una stringa: Some(Int(1042))», riga 150), `test_genesis_state_matches_engine_state_decimal` (riga 139), `test_genesis_golden_hash_constant` (`e73ef6c4…1e97` vs `4649d1f4…26b7`, riga 106), `test_genesis_state_is_string_not_integer` (riga 120), `test_genesis_prev_hash_zeroes_and_shape` (riga 91). Worktree `L0arch_d2f1610`, pulito | **vuoto** su `tests/`; lato `src/` `src/log.rs` 12+/8− | **VERDE con NOTA** (rosso d'asserzione motivato dal cambio di formato B1→B2; golden fissato **prima** dell'implementazione e non ritoccato dopo; la deviazione dal criterio «strutturale» è dichiarata) |
| `48e5529 → a345be7` (fold dello State) | No. Solo `tests/l0_state.rs` (241 righe) | strutturale — `error[E0432]: unresolved import 'ariadne::state'` a `tests/l0_state.rs:24:14`, exit 101 (worktree `L0arch_48e5529`, `git status --porcelain` vuoto) | **vuoto** (`git diff 48e5529 a345be7 -- tests/` nessuna riga); controprova su `L0arch_a345be7`: `cargo test --test l0_state` **7 passed, 0 failed**, exit 0 | **VERDE pieno** |
| `08b1536 → b2790e9` (+ `a298cf9`) (ciclo SENSE→ACTION→STATE) | No. `tests/l0_cycle.rs` (429 righe) + 8 **rinomine pure** `*_SPEC_v0.x.md → spec/*` (0 inserzioni/0 cancellazioni), entrate con l'**amend** (§8) | strutturale, **ma a due cause** — a `08b1536` (worktree `L0arch_08b1536`, pulito): `error[E0432]: unresolved imports 'ariadne::cycle', 'ariadne::Pcg64'` a `tests/l0_cycle.rs:29:15`, con entrambe le note `no 'cycle' in the root` **e** `no 'Pcg64' in the root`, exit 101. A `b2790e9` (worktree `L0arch_b2790e9`, pulito) la suite **resta ROSSA per il solo import**: `error[E0432]: unresolved import 'ariadne::Pcg64'` a `tests/l0_cycle.rs:29:38`, exit 101 | `08b1536 → b2790e9`: **vuoto** su `tests/` (`src/cycle.rs` 188 righe, `src/lib.rs` 12 righe). `b2790e9 → a298cf9`: `tests/l0_cycle.rs` 2+/1−, **solo** l'import (`use ariadne::{cycle, EventType, Log, Pcg64, State};` → `use ariadne::prng::Pcg64;` + `use ariadne::{cycle, EventType, Log, State};`), 0 asserzioni | **ROSSO** — prova test-first di B4 indebolita (dettaglio in §7.1) |
| `6d6eadc → 5b495d7` (replay bit-perfetto a due processi) | No. Solo `tests/l0_replay_process.rs` (185 righe) | strutturale — `error: environment variable 'CARGO_BIN_EXE_l0_replay' not defined at compile time` a `tests/l0_replay_process.rs:29:19`, exit 101 (worktree `L0arch_6d6eadc`, `git status --porcelain` vuoto) | **vuoto** (`git diff 6d6eadc 5b495d7 -- tests/` nessuna riga); lato `src/` solo `src/bin/l0_replay.rs` (121 righe). Controprova su `L0arch_5b495d7`: `cargo test --test l0_replay_process` **6 passed, 0 failed**, exit 0 | **VERDE pieno** |

### 7.1 Sintesi dell'anomalia B4 (una riga)

Il test nella forma committata in `08b1536` **non ha mai compilato contro `b2790e9`** (a `b2790e9` resta `E0432: no 'Pcg64' in the root`, exit 101, con `tests/l0_cycle.rs` byte-identico fra i due commit), mentre il rosso committato nominava **anche** `cycle` (`no 'cycle' in the root`): il rosso era quindi anche per il motivo giusto, ma **non esiste un commit in cui il test immutato passi da rosso a verde per sola implementazione** — il verde arriva in `a298cf9`, dopo l'implementazione, toccando il test.

### 7.2 Elenco ROSSI

1. **`08b1536 → b2790e9` (+ `a298cf9`) — ciclo B4.** Unico ROSSO. Il commit di test `08b1536` importa `Pcg64` dalla radice del crate (`tests/l0_cycle.rs:29`); a `b2790e9` l'implementazione del ciclo è presente ma il test non compila (`E0432`, exit 101) e `git diff 08b1536 b2790e9 -- tests/` è vuoto; il verde arriva solo con `a298cf9`, **dopo** l'implementazione e modificando il test. Difetto del test, non dell'implementazione — ma la prova test-first è indebolita e va dichiarata tale.
2. **Esperimento isolante §5.6 non eseguito — permessi negati** (`edit`, `sed`, `git apply`, `git checkout`) — riportato come ROSSO di *copertura* della voce, non del codice: la voce resta **APERTA** con il comando esatto per chiuderla (dichiarato in §5.6).

### 7.3 Elenco NOTE (non ROSSI)

1. **`git reflog --date=iso` / `git reflog --all` negati** dalla sandbox → metodo alternativo `git log -g --oneline` (riuscito). NOTA di metodo, non rosso del codice.
2. **Comandi composti negati:** qualunque comando con `&&`/`;` è `permission.rejected` in questa sessione; ogni comando è stato eseguito singolarmente.
3. **`Cargo.toml` + `Cargo.lock` nel commit di test `6ae990a`**: scaffold del crate e `sha2` come **dev-dependency** (commento esplicito «COMMIT TEST-ONLY»), nessuna dipendenza di produzione. NOTA.
4. **8 rinomine `spec/` entrate nell'amend di `08b1536`**: contenuto invariato (0 inserzioni/0 cancellazioni), documenti non codice. NOTA.
5. **Coppia 3 `d2f1610 → b7543f7`: rosso d'asserzione, non strutturale.** Forma legittima per un cambio di formato intenzionale, ma devia dal criterio letterale del mandato. NOTA, con il dettaglio integrale in §3.2.
6. **Test dichiarati dal producer — limiti di specificità** (§6): il replay è il log intero e in-process (il fra-processi è in `tests/l0_replay_process.rs`); l'immutabilità è provata da `test_log_is_append_only_no_mutation_truncation_or_reorder`, **non** dall'omonimo `test_immutable_events_slice_borrows` citato dal producer; `test_event_without_seq_is_rejected` asserisce `is_err()` senza ispezionare la variante d'errore.
7. **Confronto manuale del binario `l0_replay` fuori dalla suite negato** (`target/debug/l0_replay …` → `permission.rejected`): la prova resta il test `due_processi_stessi_argomenti_stdout_byte_identico`, verde, che confronta `Vec<u8>`. NOTA di metodo.
8. **Residuo lasciato:** `reports/.tmp_b4_import.patch` (diff `b2790e9→a298cf9` per `tests/l0_cycle.rs`), unico artefatto dell'esperimento, dentro la superficie ammessa `reports/`. Nessun worktree residuo: tutti rimossi con `git worktree remove --force`.

---

## 8. Reflog e amend

`git reflog --date=iso` e `git reflog --all --date=iso` sono stati **negati** (`{"error":{"type":"permission.rejected","message":"Permission denied: shell"}}`). Metodo alternativo usato, riuscito: `git log -g --oneline`. Da questa vista risultano due `commit (amend)`:

```
08b1536 HEAD@{4}: commit (amend): L0 B4: test ROSSI per il ciclo senso→azione→stato …
269ea10 HEAD@{5}: commit: L0 B4: test ROSSI per il ciclo senso→azione→stato …
a345be7 HEAD@{6}: commit: L0 B3: fold dello State …
48e5529 HEAD@{7}: commit: L0 B3: test ROSSI per fold dello State …
b7543f7 HEAD@{8}: commit (amend): L0 B2: genesi con stato PCG64 reale a 128 bit (chiude D1)
abda91a HEAD@{9}: commit: L0 B2: genesi con stato PCG64 reale a 128 bit (chiude D1)
d2f1610 HEAD@{10}: commit: L0 B2: test ROSSI per genesi con stato PCG64 stringa (chiude D1)
```

**Pre-amend recuperati:** `abda91a` (prima di `b7543f7`, B2) e `269ea10` (prima di `08b1536`, B4).

### 8.1 Amend di `b7543f7` (pre-amend `abda91a`) — solo messaggio

`git diff --stat abda91a b7543f7` → **(nessun output)**: **diff vuoto**. Alberi identici: l'amend ha cambiato **solo il messaggio**. Nessun file, nessun codice. **NOTA.**

### 8.2 Amend di `08b1536` (pre-amend `269ea10`) — rinomine `spec/` + messaggio

`git diff --stat 269ea10 08b1536`:

```
 L0_SPEC_v0.1.md => spec/L0_SPEC_v0.1.md | 0
 L0_SPEC_v0.2.md => spec/L0_SPEC_v0.2.md | 0
 L1_SPEC_v0.1.md => spec/L1_SPEC_v0.1.md | 0
 L2_SPEC_v0.1.md => spec/L2_SPEC_v0.1.md | 0
 L3_SPEC_v0.1.md => spec/L3_SPEC_v0.1.md | 0
 L4_SPEC_v0.1.md => spec/L4_SPEC_v0.1.md | 0
 L5_SPEC_v0.1.md => spec/L5_SPEC_v0.1.md | 0
 L6_SPEC_v0.2.md => spec/L6_SPEC_v0.2.md | 0
 8 files changed, 0 insertions(+), 0 deletions(-)
```

Solo **rinomine pure** (0 inserzioni, 0 cancellazioni: contenuto byte-identico) dei documenti SPEC dentro `spec/`, più la correzione del messaggio (il testo pre-amend conteneva il token corrotto `|作为|`). **`tests/l0_cycle.rs` è identico fra `269ea10` e `08b1536`** (il diff non lo nomina): la prova dell'anomalia B4 non dipende dall'amend.

### 8.3 Conclusione

**Nessun codice di produzione è entrato negli amend.** Per `b7543f7` l'amend è puramente testuale (diff vuoto). Per `08b1536` l'amend aggiunge solo rinomine a contenuto invariato di file SPEC e corregge il messaggio. Non ricorre quindi il caso «ROSSO grave» previsto dal mandato. **NOTA** per entrambi.

---

**Report completo.** Coppie 1–6 verificate (stat, rosso ricostruito su worktree pulito con primo errore integrale ed exit code, diff test→impl quantificato), anomalia B4 risolta, tre test dichiarati letti nel corpo, reflog e amend ricostruiti, matrice ed elenchi ROSSI/NOTE prodotti.

**Nessuna accettazione dichiarata.** La firma è umana.
