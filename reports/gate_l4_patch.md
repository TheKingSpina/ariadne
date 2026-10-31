# Gate procedurale — patch L4 «validazione l2/l3 uniforme nel replay build_self»

- **modello**: `opencode/deepseek-v4.1-flash-free` — classe `light`, variante: nessuna (default di catalogo), canale «Free-first», 31/10/2026. Policy di routing **v1 ATTIVA** (mandato del Router).
- **sessione mnemosyne**: `ses_2a033a96-2673-4b30-bd62-924541aa4a78` (progetto `opencode:Ariadne`, corpusRevision `efda4e4f-fbc4-48b3-abd5-0fb9ee3a5dc4:79`).
- **memorie richiamate**: `mem_8de51b0d-4309-4a48-b673-b561dc86eeab`, `mem_e096d17e-6849-4b97-a7eb-26d10cc6598f`, `mem_7e21f9e0-bf8c-462b-9f92-05719d2ca64f`, `mem_12863c3c-2c70-4c0f-a863-03d3d85e357c`, `mem_df416075-11a9-4cbd-bb65-917e3b72ec32`, `mem_1eca95a3-bd5a-481c-8407-138c3a2eb125` (troncata in output), `mem_37f7833a-cf35-4c0c-b86a-05d5866b82ce`, `mem_dd47270a-ecfa-4ac7-a732-782334a3c896`.
- **memorie proposte**: nessuna (nessun pattern nuovo: confermo il patto già memorizzato su test-first + correzioni autorizzate).
- **natura del gate**: PROCEDURALE. Eseguo, conto, verifico perimetri. Non giudico conformità legge-vs-codice (Fase C, reviewer) e non modifico codice/test/note/policy.

## 1. Revisione verificata

- `git log -1 --format=%H` → `30adda8892a1a2786e52703632816fba17ecbf4d` (coincide con HEAD del mandato).
  - NOTA di sandbox: `git rev-parse HEAD` è stato **negato** dal layer dei permessi (`Permission denied: shell`), quindi la voce è chiusa col comando equivalente `git log -1 --format=%H`. Nessun rosso imputato al codice.
- Catena del task: `30adda8` (fix) sopra `48daac3` (test) sopra `adc0b84` (accettazione L4, 144/144).
- Suite eseguita su **worktree dedicato e pulito**: `git worktree add <scratch>/ariadne-gate-l4patch 30adda8`, `git status --short` → nessun output (checkout pulito). I test sono girati lì, non nel checkout principale.

## 2. Suite completa — DEBUG

`cargo test` (worktree pulito su `30adda8`): **145/145 passed, 0 failed** (un solo run, 4.25s di build).

| binario / file | test | esito |
|---|---|---|
| unittests `src/lib.rs` | 4 | ok |
| unittests `src/bin/l0_replay.rs` | 0 | ok |
| `tests/l0_c1_genesis.rs` | 2 | ok |
| `tests/l0_cycle.rs` | 11 | ok |
| `tests/l0_event_log.rs` | 18 | ok |
| `tests/l0_l1_l2_pipeline.rs` | 1 | ok |
| `tests/l0_n3_canonical_string.rs` | 13 | ok |
| `tests/l0_n3_line_canonical.rs` | 10 | ok |
| `tests/l0_prng.rs` | 7 | ok |
| `tests/l0_replay_process.rs` | 6 | ok |
| `tests/l0_state.rs` | 9 | ok |
| `tests/l1_view.rs` | 11 | ok |
| `tests/l2_selection.rs` | 16 | ok |
| `tests/l3_memory.rs` | 14 | ok |
| `tests/l4_identity.rs` | 23 | ok |
| doc-tests | 0 | ok |

Somma: 4+0+2+11+18+1+13+10+7+6+9+11+16+14+23 = **145**. Atteso 144+1 **confermato**.

Test nuovo presente e verde:
```
test l4_check4_replay_non_action_with_l2_or_l3_is_schema ... ok
```
`l4_identity`: `test result: ok. 23 passed; 0 failed` (22 pre-esistenti L4 + 1 nuovo; i 144 pre-esistenti del ciclo accettato restano verdi).

## 3. Suite completa — RELEASE

`cargo test --release` (stesso worktree pulito): **145/145 passed, 0 failed**.
Ripartizione identica alla debug (4+0+2+11+18+1+13+10+7+6+9+11+16+14+23 = 145); `tests/l4_identity.rs`: `test result: ok. 23 passed; 0 failed`, con `l4_check4_replay_non_action_with_l2_or_l3_is_schema ... ok`. Doc-tests: 0.

## 4. Perimetro

`git diff --stat adc0b84..HEAD`:
```
 src/identity.rs      | 15 +++++++++------
 tests/l4_identity.rs | 17 +++++++++++++++++
 2 files changed, 26 insertions(+), 6 deletions(-)
```
Solo i due file attesi, nessun altro file tracciato toccato. Il terzo elemento del perimetro, `reports/l4_patch_replay_validation.md`, è **untracked** (2506 byte, 41 righe, coerente con la convenzione del repo: tutti i `reports/` sono untracked) e l'ho letto: riporta commit 1/2, perimetro e 145/145.

Stat dei due commit, separati:

`git show --stat 48daac3` (test):
```
 tests/l4_identity.rs | 17 +++++++++++++++++
 1 file changed, 17 insertions(+)
```
`git show --stat 30adda8` (fix):
```
 src/identity.rs | 15 +++++++++------
 1 file changed, 9 insertions(+), 6 deletions(-)
```
`git status` del checkout principale: nessun file tracciato modificato; solo untracked `notes/*` e `reports/*` preesistenti. Nessuna modifica a `src/memory.rs`, `src/selection.rs`, `src/lib.rs`, `Cargo.toml`, policy, altri test.

## 5. Test-first: il primo commit è solo test ed era rosso

- `48daac3` tocca **solo** `tests/l4_identity.rs` (+17, nessuna cancellazione), nessun codice di produzione: il diff aggiunge la sola funzione `l4_check4_replay_non_action_with_l2_or_l3_is_schema` (log di genesi + `Sense` con `l2=select`, `at`, `ups="[0]"`, `downs="[]"`, assert `build_self(...).unwrap_err() == SelfError::Schema`). Nessuna eccezione «stub test-only» da verificare: non ce n'è bisogno.
- Rosso ricostruito a runtime in **secondo worktree** su `48daac3` (`git worktree add <scratch>/ariadne-redcheck-l4patch 48daac3`), `cargo test --test l4_identity`. Output integrale della parte rilevante:

```
test l4_check4_replay_non_action_with_l2_or_l3_is_schema ... FAILED
...
---- l4_check4_replay_non_action_with_l2_or_l3_is_schema stdout ----

thread 'l4_check4_replay_non_action_with_l2_or_l3_is_schema' (250264) panicked at tests/l4_identity.rs:655:33:
called `Result::unwrap_err()` on an `Ok` value: SelfDocument { head_hash: "f3b6170cc10bfd7dc9d8f4fb43a11de65810c22c92e0b93956e5a1f8697a6116", statements: [], self_hash: "e4b5085c2da3ddb763a2d5e3e34931542ba302c3ebf02474088cfd3331681963" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    l4_check4_replay_non_action_with_l2_or_l3_is_schema

test result: FAILED. 22 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
Il rosso è quello giusto: `build_self` restituiva `Ok(SelfDocument)` (divergenza da `evaluate_self`), non un errore di fixture/compilazione. A HEAD lo stesso test è verde in debug e release.

## 6. Isolamento

Nessun accesso a input non dichiarati: suite eseguita sul solo checkout `30adda8` in worktree; nessuna rete, nessun path esterno usato dai test. I due worktree creati per il gate sono stati rimossi con `git worktree remove` (senza `--force`), nessun residuo.

## 7. Esito per voce

| voce | esito | prova |
|---|---|---|
| 1. `cargo test` = 145/145 | VERDE | §2, somma 145, `l4_identity` 23 ok |
| 2. `cargo test --release` = 145/145 | VERDE | §3 |
| 3. perimetro `adc0b84..HEAD` = 2 file tracciati + report untracked | VERDE | §4 |
| 4. `git rev-parse HEAD` = `30adda8…` | VERDE (NOTA: comando negato, usato `git log -1 --format=%H`) | §1 |
| 5. test nuovo presente e verde a HEAD | VERDE | §2/§3 |
| 6. primo commit solo test e rosso a runtime | VERDE | §5 |

## VERDETTO: **VERDE** (gate procedurale)

Conteggi esatti: **debug 145/145**, **release 145/145**. Perimetro conforme: `src/identity.rs`, `tests/l4_identity.rs` (tracciati) + `reports/l4_patch_replay_validation.md` (untracked). HEAD `30adda8892a1a2786e52703632816fba17ecbf4d`. Nessun push, nessun commit, nessuna modifica fuori da questo report.

Nota di scope: questo gate **non** chiude la Fase C (conformità legge↔codice del fix `validate_decision_fields`/`Schema` su eventi non-ACTION): spetta al reviewer.

---

# Gate procedurale — copertura `l3` (append, 31/10/2026)

- **modello**: `opencode/deepseek-v4.1-flash-free` — classe `light`, variante: nessuna (default di catalogo), canale «Free-first», 31/10/2026. Policy di routing **v1 ATTIVA** (mandato del Router).
- **sessione mnemosyne**: `ses_58cdc7ec-36c5-409e-a783-eaf2249fba40` (progetto `opencode:Ariadne`, corpusRevision `efda4e4f-fbc4-48b3-abd5-0fb9ee3a5dc4:82`).
- **memorie richiamate**: `memory_context` + `memory_search` su «L4 test» / «L4 violazione» → `mem_8de51b0d-4309-4a48-b673-b561dc86eeab`, `mem_e096d17e-6849-4b97-a7eb-26d10cc6598f`, `mem_7e21f9e0-bf8c-462b-9f92-05719d2ca64f`, `mem_12863c3c-2c70-4c0f-a863-03d3d85e357c`, `mem_1eca95a3-bd5a-481c-8407-138c3a2eb125`, `mem_37f7833a-cf35-4c0c-b86a-05d5866b82ce`, `mem_df416075-11a9-4cbd-bb65-917e3b72ec32`, `mem_1fc3a41f-8618-4257-b486-0319cc6092a6`, `mem_7118d6a8-d63f-4cfc-a9c7-ae756c3f42b4`, `mem_ffe6a92e-6bb0-4d6c-9d43-5b344e3053bf`, `mem_616ad586-4307-48c3-9594-ee1c28b3bb52`, `mem_dd47270a-ecfa-4ac7-a732-782334a3c896`, `mem_2affa11c-2399-4ffa-b935-8aee3b1b2177`.
- **memorie proposte**: nessuna (nessun pattern di violazione nuovo; il pattern «copertura post-gate vs test-first» è già distinto in `mem_7118d6a8`).
- **natura del gate**: PROCEDURALE. Eseguo, conto, verifico perimetri. Non giudico conformità legge↔codice (Fase C non richiesta da questo mandato) e non modifico codice/test/note/policy.
- **oggetto**: rilievo informativo (a) di `reports/reaudit_l4_patch.md` — copertura del ramo `l3` della validazione uniforme; autorizzazione umana «test subito» (31/10/2026).

## C1. Revisione verificata

- `git rev-parse HEAD` → **NEGATO** dal layer dei permessi (`Permission denied: shell`), come nel gate precedente.
- Metodo alternativo dichiarato: `git log -1 --format=%H` → `8d82f47dca9b1787f78fda28b1b93b452f39ad74` (coincide con l'HEAD del mandato).
- Catena: `8d82f47` (copertura l3) sopra `30adda8` (fix verde) sopra `48daac3` (test rosso) sopra `adc0b84` (accettazione L4, 144/144). `git log --oneline 30adda8..HEAD` → un solo commit, `8d82f47`.
- Suite eseguita su **worktree dedicato e pulito**: `git worktree add <scratch>/ariadne-gate-l4-copertura-l3 8d82f47…`; `git status --short` → nessun output (checkout pulito). I test sono girati lì, non nel checkout principale.

## C2. Suite completa — DEBUG

`cargo test` (worktree pulito su `8d82f47`): **146/146 passed, 0 failed** (build 4.22s).

| binario / file | test | esito |
|---|---|---|
| unittests `src/lib.rs` | 4 | ok |
| unittests `src/bin/l0_replay.rs` | 0 | ok |
| `tests/l0_c1_genesis.rs` | 2 | ok |
| `tests/l0_cycle.rs` | 11 | ok |
| `tests/l0_event_log.rs` | 18 | ok |
| `tests/l0_l1_l2_pipeline.rs` | 1 | ok |
| `tests/l0_n3_canonical_string.rs` | 13 | ok |
| `tests/l0_n3_line_canonical.rs` | 10 | ok |
| `tests/l0_prng.rs` | 7 | ok |
| `tests/l0_replay_process.rs` | 6 | ok |
| `tests/l0_state.rs` | 9 | ok |
| `tests/l1_view.rs` | 11 | ok |
| `tests/l2_selection.rs` | 16 | ok |
| `tests/l3_memory.rs` | 14 | ok |
| `tests/l4_identity.rs` | 24 | ok |
| doc-tests | 0 | ok |

Somma: 4+0+2+11+18+1+13+10+7+6+9+11+16+14+24 = **146**. Atteso 145+1 **confermato**.

Test nuovo presente e verde (output integrale della riga):
```
test l4_check4_replay_non_action_with_l3_is_schema ... ok
```
`tests/l4_identity.rs`: `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` (23 preesistenti + 1 nuovo).

## C3. Suite completa — RELEASE

`cargo test --release` (stesso worktree pulito): **146/146 passed, 0 failed** (build 6.41s).
Ripartizione identica alla debug (4+0+2+11+18+1+13+10+7+6+9+11+16+14+24 = 146); `tests/l4_identity.rs`: `test result: ok. 24 passed; 0 failed`, con
```
test l4_check4_replay_non_action_with_l3_is_schema ... ok
```
Doc-tests: 0.

## C4. Perimetro `30adda8..HEAD`

`git diff --stat 30adda8..HEAD`:
```
 tests/l4_identity.rs | 16 ++++++++++++++++
 1 file changed, 16 insertions(+)
```
`git show --stat 8d82f47`:
```
commit 8d82f47dca9b1787f78fda28b1b93b452f39ad74
Author: Alessandro Spinabelli <Alessandro.spinabelli@studenti.unipr.it>
Date:   Sat Oct 31 23:06:21 2026 +0100

    L4: copertura post-gate del ramo l3 nell'uniformità replay non-ACTION
    ...
 tests/l4_identity.rs | 16 ++++++++++++++++
 1 file changed, 16 insertions(+)
```
Diff integrale `30adda8..HEAD` (unica modifica, nessuna cancellazione):
```diff
@@ -655,6 +655,22 @@ fn l4_check4_replay_non_action_with_l2_or_l3_is_schema() {
     assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
 }
 
+// [copertura re-audit L4 / rilievo informativo (a)] diretta del ramo `l3`:
+// evento non-ACTION (SENSE) con chiavi l3 nel payload → Schema ANCHE nel
+// replay di build_self (specchio l2 del test a riga 647; chiavi L3 valide).
+#[test]
+fn l4_check4_replay_non_action_with_l3_is_schema() {
+    let mut log = Log::genesis(1042).unwrap();
+    log.append(3, EventType::Sense, Payload::from_pairs([
+        ("l3", Value::Str("promote".into())),
+        ("origin_seq", Value::Int(0)),
+        ("id", Value::Str(
+            sha256_hex(&("{\"key\":\"A\",\"kind\":\"letter\"}".to_string())),
+        )),
+    ])).unwrap();
+    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
+}
+
 // [L4.3(d)/L4.7] errore dentro un evento letto da L4: id L3 non conforme
 // (non hex minuscolo a 64) → Schema.
 #[test]
```

Produzione **intoccata**: `git diff --stat 30adda8..HEAD -- src Cargo.toml notes .opencode` → **nessun output** (zero file). `src/identity.rs`, `src/lib.rs`, `src/memory.rs`, `src/selection.rs`, `Cargo.toml`, `notes/`, `.opencode/` identici a `30adda8`.

Secondo elemento atteso del perimetro — append del producer in `reports/l4_patch_replay_validation.md` — **non è nel commit**: il file è **untracked** (`?? reports/l4_patch_replay_validation.md` in `git status --short` del checkout principale, convenzione del repo per tutti i `reports/`). L'ho letto: la sezione «Aggiunta copertura ramo `l3` …» alle righe 43–49 riporta commit `8d82f47`, pattern post-gate dichiarato, 146/146 debug, modello del producer. Append presente e coerente. Nessun altro file tracciato o untracked nuovo attribuibile a questo task.

## C5. Esito per voce

| voce | esito | prova |
|---|---|---|
| 1. `cargo test` = 146/146 | VERDE | §C2, somma 146, `l4_identity` 24 ok |
| 2. `cargo test --release` = 146/146 | VERDE | §C3 |
| 3. perimetro `30adda8..HEAD` = solo `tests/l4_identity.rs` (+16) + append untracked in `reports/` | VERDE | §C4 |
| 4. `git rev-parse HEAD` = `8d82f47…` | VERDE (NOTA: comando negato, usato `git log -1 --format=%H`) | §C1 |
| 5. test nuovo `l4_check4_replay_non_action_with_l3_is_schema` verde in debug e release | VERDE | §C2/§C3 |
| 6. produzione `src/` intoccata (0 file) | VERDE | §C4 |

Non applicabile a questo mandato: ordine test-first del PRIMO commit (è un commit di **copertura post-gate** dichiarato tale nel messaggio e nel report producer, autorizzato dall'umano «test subito»; la patch era già in vigore da `30adda8`, quindi il verde alla stesura è l'esito atteso, non una violazione). Nessun giudizio di conformità legge↔codice: Fase C fuori mandato.

## VERDETTO copertura l3: **VERDE** (gate procedurale)

Conteggi esatti: **debug 146/146**, **release 146/146**. Perimetro conforme: solo `tests/l4_identity.rs` (+16 righe, 1 test nuovo) nel commit; produzione `src/` invariata; append del producer presente in `reports/l4_patch_replay_validation.md` (untracked, righe 43–49). HEAD `8d82f47dca9b1787f78fda28b1b93b452f39ad74`. Sessione `ses_58cdc7ec-36c5-409e-a783-eaf2249fba40`. Modello `opencode/deepseek-v4.1-flash-free`, classe `light`, variante nessuna, canale Free-first.

Nessun push, nessun commit, nessuna modifica fuori da questo report. Worktree di gate rimosso con `git worktree remove`, senza `--force`.

Nota di scope: questo verdetto **non** chiude la Fase C (conformità legge↔codice della copertura `l3`) e **non** riapre l'audit L4: è un gate di esecuzione e perimetro.
