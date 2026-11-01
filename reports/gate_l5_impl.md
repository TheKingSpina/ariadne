# Gate L5 — implementazione (procedurale) — VERDE

- **modello**: `opencode/deepseek-v4.1-flash-free` — classe `light`, variante: **nessuna** (default del catalogo), canale free «Free-first», 31/10/2026 (come da mandato Router)
- **policy**: `.opencode/routing.md` v1 **ATTIVO** (letto integralmente; scelta del modello documentata nel mandato → nessuna ereditarietà implicita)
- **sessione tester (Mnemosyne)**: `ses_ef841c07-f861-4d4b-a557-5682d8b42e8d` (progetto `opencode:Ariadne`)
- **memorie richiamate**: `mem_23ee6360-e5ff-4f01-b636-5b3d2645ed51` (episode ciclo L4, 146 test baseline), `mem_e096d17e-6849-4b97-a7eb-26d10cc6598f` (episode completamento L4, 144/144), `mem_616ad586-4307-48c3-9594-ee1c28b3bb52` (episode gate compatto L0–L3), `mem_1eca95a3-bd5a-481c-8407-138c3a2eb125` (decision: cutoff L4). `memory_search("L5 violazione")` → 0 items (nessun pattern L5 pregresso noto).
- **memorie proposte**: `mem_6507ae6e-1853-492c-9c54-f72857626716` (convention, `pending_approval`) su verifica «byte per byte» di `SocialView`, zero draw indiretto e forma del rosso test-first E0432 (vedi §6 NOTA); nessun pattern di violazione nuovo rilevato.
- **legge vigente**: `notes/L5_legge_sociale.md` (commit `23a7104`), emenda `spec/L5_SPEC_v0.1.md`; checklist di uscita righe 207–236
- **revisione verificata**: HEAD `4f9e834876c79eea874c56db32774ae805d7c180` su `4374e91f453fa5c9cc61b52b6acf798116359819` su `23a71042ed17c45c7f480030cf277f43ba32758b`
- **scope**: gate procedurale sull'**implementazione** L5 (esecuzione, conteggi, perimetri, hash, test-first). **Non** giudica la conformità legge-vs-codice (Fase C, reviewer).

## 1. HEAD

`git rev-parse HEAD` → **NEGATO** (`Permission denied: shell`). Alternativa dichiarata usata:

```
git log -3 --format='%H %s'
4f9e834876c79eea874c56db32774ae805d7c180 L5: legge sociale implementata — src/social.rs (L5.1–L5.11)
4374e91f453fa5c9cc61b52b6acf798116359819 L5: test rossi della legge sociale — checklist 1–8 (test-first)
23a71042ed17c45c7f480030cf277f43ba32758b legge L5 (sociale) in vigore: triage Q1-Q15 firmato per delega umana
```

confermato da `git show --stat --format='%H %s' 4f9e834` (intestazione `commit 4f9e834876c79eea874c56db32774ae805d7c180`) e da `git worktree add` → `HEAD is now at 4f9e834 L5: legge sociale implementata`.

## 2. Suite completa — debug (worktree pulito)

Worktree: `git worktree add /private/var/.../opencode/ariadne_l5_gate 4f9e834` → `Preparing worktree (detached HEAD 4f9e834)`.
`git status --porcelain=v1 --branch` → `## HEAD (no branch)` (nessuna riga = pulito).

`cargo test` (output per binario, righe testuali):

```
Running unittests src/lib.rs                      test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/bin/l0_replay.rs            test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_c1_genesis.rs                    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_cycle.rs                         test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_event_log.rs                     test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_l1_l2_pipeline.rs                test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/l0_n3_canonical_string.rs           test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_n3_line_canonical.rs             test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_prng.rs                          test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_replay_process.rs                test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
Running tests/l0_state.rs                         test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l1_view.rs                          test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/l2_selection.rs                     test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l3_memory.rs                        test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l4_identity.rs                      test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/l5_social.rs                        test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests ariadne                                 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Somma: 4+0+2+11+18+1+13+10+7+6+9+11+16+14+24+18+0 = **164 passed, 0 failed** (146 L0–L4 + 18 L5). **Atteso 164/164 → confermato.**

## 3. Suite completa — release (stesso worktree)

`cargo test --release`:

```
Running unittests src/lib.rs                      test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/bin/l0_replay.rs            test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_c1_genesis.rs                    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_cycle.rs                         test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_event_log.rs                     test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_l1_l2_pipeline.rs                test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_n3_canonical_string.rs           test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_n3_line_canonical.rs             test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_prng.rs                          test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_replay_process.rs                test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
Running tests/l0_state.rs                         test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l1_view.rs                          test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l2_selection.rs                     test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l3_memory.rs                        test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l4_identity.rs                      test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l5_social.rs                        test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests ariadne                                 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Somma: **164 passed, 0 failed**. Identico al debug (stessi 18 `l5_social` ok, stessi conteggi per binario). **Atteso 164/164 → confermato.**

## 4. Checklist di uscita 1–8 (`notes/L5_legge_sociale.md` righe 207–236) — punto per punto

| # | Punto | Test `tests/l5_social.rs` (righe) | Esito |
| --- | --- | --- | --- |
| 1 | Replay byte per byte; log di sola genesi → models vuoto, head_hash genesi | `l5_check1_replay_is_byte_per_byte` (110–117): due `build_social` → `assert_eq!(v1, v2)`; `l5_check1_genesis_only_models_empty_head_hash_is_genesis` (93–105): `models.is_empty()`, `head_hash == events()[0].hash`, array `osservazioni` vuoto ammesso | VERDE (prova: 2 test ok debug+release; cfr. NOTA §6) |
| 2 | Frontiera: nessuna scrittura L5 nel log, append solo API L0, riscontro head_hash + `log.verify()` | `l5_check2_frontiera_no_l5_writes_in_log` (126–146): `events().len()` e `to_jsonl()` invariati dopo le 3 API, `log.verify().unwrap()`, `view.head_hash == last.hash` | VERDE |
| 3 | Ogni previsione ancorata: `sources` = seq osservate dedup crescente; `at` riferisce seq esistente | `l5_check3_predict_source_seqs_and_no_model` (155–180): `sources == [1,3]` e `[2,3,4]`, `previsto` argmax/tie-break, `NoModel` per id senza osservazioni; `at` inesistente → `Schema` in `l5_check6_malformed_l5_payload_is_schema` (314–318) | VERDE |
| 4 | Divergenza scoped (i) eventi irrilevanti → modello di X identico; (ii) osservazione rilevante → modello/contatori cambiano, vista intera differisce per head_hash; nessuna iniettività universale | `l5_check4_divergenza_irrelevant_events_leave_model_identical` (191–209): `models.get(bob)` uguali, `head_hash` diversi; `l5_check4_divergenza_relevant_observation_changes_model` (214–228): `n`, `contatori`, `head_hash` tutti `assert_ne!` | VERDE (2 test) |
| 5 | Secondo ordine: composizione pura = previsione diretta, byte per byte, nessuna via alternativa | `l5_check5_second_order_composition_equals_direct_prediction` (238–271): argmax ricalcolato indipendentemente dai `contatori` == `direct.previsto`; `direct.sources == [1,2,3,4]` | VERDE |
| 6 | Errori precisi, non-panic: payload malformati → `Schema`; base incoerente → `HeadMatch`; predict su modello assente → `NoModel` | `l5_check6_malformed_l5_payload_is_schema` (281–326: chiave extra, chiave assente, id non hex64, `at` impossibile, l5 fuori dominio); `l5_check6_malformed_osservazioni_is_schema` (332–369: JSON non canonico, chiavi fuori ordine, chiave extra, label vuota, negativo); `l5_check6_osservazioni_on_non_sense_is_schema` (375–390: ACTION e STATE); `l5_check6_l5_mixed_with_other_tags_is_schema` (395–410: l2/l3/l4 + non-ACTION); `l5_check6_base_incoerente_is_head_match` (414–429: head errato, log_len 0 e oltre); `NoModel`: `l5_check3` (171–174) | VERDE (5 test + NoModel) |
| 7 | Cutoff: `l5` nel prefisso → zero decisioni; solo finestre post-cutoff; chiusura finestra L2 da `l5` (pattern check6 L4); nessun side-effect su L2/L3/L4 | `l5_check7_l5_in_base_prefix_gives_zero_decisions` (501–526); `l5_check7_l5_event_closes_l2_window_no_side_effects` (532–561: `selection::evaluate`, `memory::evaluate`, `evaluate_self` tutti empty, `build_self` ok); semantica finestra L5.7: `l5_check7_window_update_mismatch_one_decision` (434–452), `l5_check7_window_equal_observation_no_update` (457–467), `l5_check7_second_predict_replaces_anchor` (472–492); nessun side-effect globale: suite L0–L4 146/146 nella stessa run | VERDE (5 test + suite) |
| 8 | Zero draw (stato PRNG invariato) e determinismo: due `evaluate_social` → decisioni identiche | `l5_check8_zero_draw_and_deterministic_decisions` (571–608): `run_cycles` poi `evaluate_social` ×2 → `d1 == d2`, `len()==1` con `at/tick/previsto/osservato/sources` esatti; `events().len()` e `to_jsonl()` invariati, `log.verify()` ok | VERDE (indiretta sui draw: nessun evento apposto ⇒ nessun draw; cfr. NOTA §6) |

### Nomi dei 18 test L5 ed esito (debug e release: `ok` in entrambe)

1. `l5_check1_genesis_only_models_empty_head_hash_is_genesis` — ok / ok
2. `l5_check1_replay_is_byte_per_byte` — ok / ok
3. `l5_check2_frontiera_no_l5_writes_in_log` — ok / ok
4. `l5_check3_predict_source_seqs_and_no_model` — ok / ok
5. `l5_check4_divergenza_irrelevant_events_leave_model_identical` — ok / ok
6. `l5_check4_divergenza_relevant_observation_changes_model` — ok / ok
7. `l5_check5_second_order_composition_equals_direct_prediction` — ok / ok
8. `l5_check6_base_incoerente_is_head_match` — ok / ok
9. `l5_check6_l5_mixed_with_other_tags_is_schema` — ok / ok
10. `l5_check6_malformed_l5_payload_is_schema` — ok / ok
11. `l5_check6_malformed_osservazioni_is_schema` — ok / ok
12. `l5_check6_osservazioni_on_non_sense_is_schema` — ok / ok
13. `l5_check7_l5_event_closes_l2_window_no_side_effects` — ok / ok
14. `l5_check7_l5_in_base_prefix_gives_zero_decisions` — ok / ok
15. `l5_check7_second_predict_replaces_anchor` — ok / ok
16. `l5_check7_window_equal_observation_no_update` — ok / ok
17. `l5_check7_window_update_mismatch_one_decision` — ok / ok
18. `l5_check8_zero_draw_and_deterministic_decisions` — ok / ok

Nessun test L5 copre esplicitamente l'assenza di panic via `catch_unwind` (checklist 6 "MAI panic"): la prova è l'assenza di panic nella run (0 failed), non un assert dedicato. → **NOTA**, non rosso.

## 5. Perimetro `23a7104..HEAD`

```
git diff --name-status 23a7104 4f9e834
M	src/lib.rs
A	src/social.rs
A	tests/l5_social.rs
```

```
git diff --stat 23a7104..HEAD
 src/lib.rs         |   1 +
 src/social.rs      | 636 +++++++++++++++++++++++++++++++++++++++++++++++++++++
 tests/l5_social.rs | 608 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 1245 insertions(+)
```

Commit `4374e91` (stat separato, `git show --stat`):

```
4374e91f453fa5c9cc61b52b6acf798116359819 L5: test rossi della legge sociale — checklist 1–8 (test-first)

 tests/l5_social.rs | 608 +++++++++++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 608 insertions(+)
```

Commit `4f9e834` (stat separato):

```
4f9e834876c79eea874c56db32774ae805d7c180 L5: legge sociale implementata — src/social.rs (L5.1–L5.11)

 src/lib.rs    |   1 +
 src/social.rs | 636 ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 637 insertions(+)
```

`src/lib.rs` — unica modifica, una riga esatta (`git show 4f9e834 -- src/lib.rs`):

```
@@ -25,6 +25,7 @@ pub mod memory;
 mod payload;
 pub mod prng;
 pub mod selection;
+pub mod social;
 pub mod state;
 pub mod view;
```

**Perimetro conforme**: solo `src/social.rs` (nuovo), `src/lib.rs` (+1 riga `pub mod social;`), `tests/l5_social.rs` (nuovo). **Non toccati** in `23a7104..HEAD`: `src/cycle.rs`, `src/prng.rs`, `src/identity.rs`, `src/memory.rs`, `src/selection.rs`, `src/log.rs`, `src/event.rs`, `src/view.rs`, `notes/*` (leggi), `spec/*`, `.opencode/routing.md`.

Working tree del checkout principale (`git status --porcelain=v1`): solo untracked, inclusi il report producer e questo report — nessuna modifica a file versionati:

```
?? notes/L3_domande_memoria.md
?? notes/L4_domande_identita.md
?? notes/mnemosyne_backlog.md
?? reports/gate_l5_impl.md
?? reports/l5_impl_sociale.md
?? reports/routing_l5_impl.md
```

Report producer `reports/l5_impl_sociale.md` presente e non committato (come da mandato).

## 6. Ordine dei commit e test-first

- Primo commit del task `4374e91` = **solo** `tests/l5_social.rs` (608 righe, 1 file). Nessun codice di produzione, nessuno stub, nessuna eccezione da autorizzare. → voce **VERDE**.
- Rosso provato a `4374e91` su worktree dedicato (`git worktree add ... 4374e91`), `cargo test --test l5_social`, output integrale:

```
error[E0432]: unresolved import `ariadne::social`
  --> tests/l5_social.rs:15:14
   |
15 | use ariadne::social::{build_social, evaluate_social, predict_social, Decision5, SocialBase, SocialError};
   |              ^^^^^^ could not find `social` in `ariadne`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `ariadne` (test "l5_social") due to 1 previous error
warning: build failed, waiting for other jobs to finish...

Exited with code 101
```

Rosso **per compilazione** (E0432, 0 test eseguiti, exit 101): è il rosso naturale di un livello con modulo nuovo; i 18 test sono stati scritti prima dell'implementazione e non erano verdi. → voce **VERDE**.

### NOTA (metodo, non rosso) — "byte per byte" della vista

`SocialView` non ha serializzatore nella legge L5.10 (l'API pubblica elenca solo struct/enum e le funzioni `build_social`/`predict_social`/`evaluate_social`/`encode5*`); quindi la checklist 1 "stesso log → stessa vista, byte per byte" è verificata nei test come **uguaglianza strutturale** (`assert_eq!(v1, v2)` su `BTreeMap`/`String`, ordinamento deterministico), non come confronto di byte serializzati — non esiste una serializzazione normata della vista da confrontare. Non richiedo una seconda implementazione indipendente: sarebbe teatro, e la legge non definisce un encoding di vista.
Analogamente la checklist 8 (zero draw) è verificata **indirettamente**: nessuna API L5 appone eventi (`events().len()` e `to_jsonl()` invariati) ⇒ nessun draw dal PRNG; il test non ispeziona lo stato `Pcg64` direttamente.

### Isolamento / provenienza (controllo di perimetro, non giudizio semantico)

`src/social.rs` importa solo `std::collections::BTreeMap`, `sha2`, `crate::payload::write_escaped`, `crate::{Event, EventType, Log, Value}`; nessun `Log` preso per `&mut`, nessuna occorrenza di `append` nel modulo (grep su `src/social.rs`: nessun match per `append`; l'unico `&mut` è `fn osserva(&mut self, ...)` sull'accumulatore interno) → coerente con L5.9 "nessuna API L5 scrive nel log". `tests/l5_social.rs` usa le viste L1–L4 in sola lettura (`identity`, `memory`, `selection`, `cycle`) solo per la prova di chiusura finestra L2 / zero side-effect (righe 16–20, 532–561), come previsto da L5.4/checklist 7. Nessun accesso a input non dichiarati nella SPEC rilevato.

## 7. SHA-256

```
0d0e7e8c42d4c718545febb2796ed3c27d28ba2cf0e46690f280d6faa6e391b6  notes/L5_legge_sociale.md
bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177  .opencode/routing.md
```

(`shasum -a 256`, checkout principale; `routing.md` è su working tree non committato, hash della copia effettivamente letta.)

## 8. Verdetto

**VERDE** — gate procedurale di implementazione L5.

- debug: **164/164 passed, 0 failed** (146 L0–L4 + 18 L5)
- release: **164/164 passed, 0 failed**, identico al debug
- checklist 1–8: tutte coperte da test L5 verdi, punto per punto (§4), con due NOTA di metodo dichiarate (§6) che non inficiano l'esito
- perimetro `23a7104..HEAD`: 3 file esatti, nessun tocco a moduli/leggi/SPEC/policy
- test-first: primo commit `4374e91` solo test, rosso E0432 verificato
- HEAD: `4f9e834876c79eea874c56db32774ae805d7c180`
- session ID: `ses_ef841c07-f861-4d4b-a557-5682d8b42e8d`
- modello: `opencode/deepseek-v4.1-flash-free`, classe `light`, variante nessuna (default catalogo), canale free

**Non** chiude il ciclo: la conformità legge-vs-codice resta al reviewer (Fase C) e l'accettazione all'umano. Nessuna modifica a codice/test/note/policy; nessun commit, nessun push.

---

# Append — Gate PATCH R1 (procedurale) — **VERDE**

- **modello**: `opencode/deepseek-v4.1-flash-free` — classe `light`, variante: **nessuna** (default del catalogo), canale free «Free-first», 31/10/2026 (mandato Router)
- **policy**: `.opencode/routing.md` v1 **ATTIVO** (riletto integralmente in questa sessione; scelta del modello esplicita nel mandato → nessuna ereditarietà implicita)
- **sessione tester (Mnemosyne, sessione distinta dal producer)**: `ses_f15c43c5-2a04-40b8-9987-168579785f1e` (progetto `opencode:Ariadne`)
- **memorie richiamate**: `mem_23ee6360-e5ff-4f01-b636-5b3d2645ed51` (episode ciclo L4, baseline 146), `mem_7e21f9e0-bf8c-462b-9f92-05719d2ca64f` (convention ripresa L4: fix test meccanici autorizzati), `mem_ff84d6c9-caf5-4a4f-9866-f33379e2d042` (episode audit L3: pattern test-first/stub), `mem_e096d17e-6849-4b97-a7eb-26d10cc6598f` (episode completamento L4). `memory_search("L5 violazione")` → **0 items** (nessun pattern di violazione L5 pregresso).
- **memorie proposte**: nessuna in questa fase (nessun pattern di violazione nuovo; il «test duplicato» è un difetto meccanico dichiarato dal producer, non una convenzione).
- **legge vigente**: `notes/L5_legge_sociale.md` (commit `23a7104`), emenda `spec/L5_SPEC_v0.1.md`; rilievo R1 da `reports/audit_l5_impl.md`
- **revisione verificata**: HEAD `77070b7d747f5d07eeba4110dea35b3b8b019343` su `a2e6aee838c0c8f4ee265fe66a36fa5f7ca8a37b` su `4f9e834876c79eea874c56db32774ae805d7c180` su `4374e91f453fa5c9cc61b52b6acf798116359819` su `23a71042ed17c45c7f480030cf277f43ba32758b`
- **scope**: gate procedurale del **mini-ciclo PATCH R1** (esecuzione, conteggi, perimetri, ordine commit, hash). **Non** giudica la conformità legge-vs-codice: spetta al re-audit (Fase C).

## A1. HEAD

`git rev-parse HEAD` → **NEGATO** (`Permission denied: shell`). Alternativa dichiarata (come da mandato):

```
git log -1 --format=%H
77070b7d747f5d07eeba4110dea35b3b8b019343
```

confermato da `git log -5 --format=%H`:

```
77070b7d747f5d07eeba4110dea35b3b8b019343
a2e6aee838c0c8f4ee265fe66a36fa5f7ca8a37b
4f9e834876c79eea874c56db32774ae805d7c180
4374e91f453fa5c9cc61b52b6acf798116359819
23a71042ed17c45c7f480030cf277f43ba32758b
```

e da `git worktree add ... 77070b7` → `HEAD is now at 77070b7 L5: patch R1 — osservazioni non-F1 su SENSE → SocialError::Schema (L5.2)`.

## A2. Suite completa — debug (worktree pulito)

Worktree: `git worktree add /private/var/.../opencode/ariadne_l5_r1_gate 77070b7` → `Preparing worktree (detached HEAD 77070b7)`.
`git status --porcelain=v1 --branch` → `## HEAD (no branch)` (nessuna riga = pulito).

`cargo test` — righe `test result` per binario, testuali:

```
Running unittests src/lib.rs                      test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/bin/l0_replay.rs            test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_c1_genesis.rs                    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_cycle.rs                         test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_event_log.rs                     test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_l1_l2_pipeline.rs                test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/l0_n3_canonical_string.rs           test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_n3_line_canonical.rs             test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_prng.rs                          test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_replay_process.rs                test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
Running tests/l0_state.rs                         test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l1_view.rs                          test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/l2_selection.rs                     test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l3_memory.rs                        test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l4_identity.rs                      test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/l5_social.rs                        test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Doc-tests ariadne                                 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Somma: 4+0+2+11+18+1+13+10+7+6+9+11+16+14+24+19+0 = **165 passed, 0 failed** (146 L0–L4 + 19 L5). **Atteso 165/165 → confermato.**

Il nuovo test è presente e `ok` nella lista del binario `l5_social`: `test l5_check_osservazioni_non_f1_is_schema ... ok`.

## A3. Suite completa — release (stesso worktree)

`cargo test --release` — righe `test result` per binario, testuali:

```
Running unittests src/lib.rs                      test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/bin/l0_replay.rs            test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_c1_genesis.rs                    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_cycle.rs                         test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_event_log.rs                     test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_l1_l2_pipeline.rs                test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_n3_canonical_string.rs           test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_n3_line_canonical.rs             test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_prng.rs                          test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l0_replay_process.rs                test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
Running tests/l0_state.rs                         test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l1_view.rs                          test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l2_selection.rs                     test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l3_memory.rs                        test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l4_identity.rs                      test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/l5_social.rs                        test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests ariadne                                 test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Somma: **165 passed, 0 failed**. **Atteso 165/165 → confermato**; identico al debug (stessi conteggi per binario, `l5_check_osservazioni_non_f1_is_schema ... ok` anche qui).

## A4. Perimetro `4f9e834..HEAD`

```
git diff --name-status 4f9e834 HEAD
M	src/social.rs
M	tests/l5_social.rs
```

```
git diff --stat 4f9e834..HEAD
 src/social.rs      | 20 +++++++++++++++++---
 tests/l5_social.rs | 42 ++++++++++++++++++++++++++++++++++++++++++
 2 files changed, 59 insertions(+), 3 deletions(-)
```

Stat separati dei due commit:

```
git show --stat a2e6aee
 tests/l5_social.rs | 44 ++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 44 insertions(+)
```

```
git show --stat 77070b7
 src/social.rs      | 20 +++++++++++++++++---
 tests/l5_social.rs |  2 --
 2 files changed, 17 insertions(+), 5 deletions(-)
```

**Perimetro conforme**: `4f9e834..HEAD` tocca **solo** `src/social.rs` e `tests/l5_social.rs`. **Non compaiono** nel diff: `src/lib.rs`, `src/cycle.rs`, `src/prng.rs`, `src/identity.rs`, `src/memory.rs`, `src/selection.rs`, `src/log.rs`, `src/event.rs`, `src/view.rs`, `notes/*`, `spec/*`, `.opencode/routing.md`, `Cargo.toml`.

Fix minimo: `git show 77070b7 -- src/social.rs` mostra **due soli** hunk, entrambi il passaggio `if let Some(Value::Str(raw))` → `if let Some(value)` con `match` che rifiuta i non-`Value::Str` in `SocialError::Schema`: uno in `replay_upto` (HEAD `src/social.rs:384–392`, commento «patch R1» a `:385`), uno in `evaluate_social` (HEAD `src/social.rs:590–597`). `grep -n "osservazioni" src/social.rs` (worktree pulito su HEAD) elenca **solo due** punti di gestione del valore: `:387` e `:593` (gli altri match sono commenti/doc e il check chiave-su-non-SENSE a `:381`). Nessun altro sito di validazione modificato.

**NOTA di perimetro (fatto, non giudizio).** Il commit `77070b7` non è di solo codice: rimuove **2 righe** da `tests/l5_social.rs` (un commento + un `#[test]` duplicati sopra `l5_check6_base_incoerente_is_head_match`). Il messaggio di commit la dichiara come «correzione meccanica al test … autorizzazione preventiva dell'orchestratore». Il file rientra nel perimetro autorizzato (`tests/l5_social.rs`), la rimozione non introduce test nuovi e lascia **una** registrazione del test (verificata: 19 test distinti in HEAD, §A5). Resta però una scrittura su test dentro il commit del fix: la ratifica dell'autorizzazione spetta al re-audit/orchestratore, non a questo gate. Effetto collaterale documentato e verificato: a `a2e6aee` il binario `l5_social` registrava **20** test (duplicato attivo, `warning: duplicated attribute --> tests/l5_social.rs:457`), in HEAD **19**.

## A5. Test-first (ordine dei commit)

`a2e6aee` — **solo test**, nessun codice di produzione:

```
git show --stat a2e6aee
 tests/l5_social.rs | 44 ++++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 44 insertions(+)
```

Il diff (`git show a2e6aee -- tests/l5_social.rs`) aggiunge la funzione `l5_check_osservazioni_non_f1_is_schema` (4 casi: `Value::Int(5)` su SENSE; `"[\"bob\"]"`; `{"altro":"bob"}`; array F1 con spazi) **più** 2 righe duplicate (`// [checklist 6 / L5.10] …` + `#[test]`) sopra `l5_check6_base_incoerente_is_head_match` — difetto meccanico corretto poi in `77070b7`.

Rosso **provato a runtime** su worktree dedicato (`git worktree add ... a2e6aee` → `HEAD is now at a2e6aee`), `cargo test --test l5_social`, output integrale:

```
warning: duplicated attribute
   --> tests/l5_social.rs:457:1
    |
457 | #[test]
    | ^^^^^^^
    |
    = note: `#[warn(duplicate_macro_attributes)]` on by default

warning: `ariadne` (test "l5_social") generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.19s
     Running tests/l5_social.rs (target/debug/deps/l5_social-22ac42687f9af975)

running 20 tests
test l5_check6_base_incoerente_is_head_match ... ok
test l5_check4_divergenza_relevant_observation_changes_model ... ok
test l5_check2_frontiera_no_l5_writes_in_log ... ok
test l5_check1_replay_is_byte_per_byte ... ok
test l5_check1_genesis_only_models_empty_head_hash_is_genesis ... ok
test l5_check3_predict_source_seqs_and_no_model ... ok
test l5_check5_second_order_composition_equals_direct_prediction ... ok
test l5_check4_divergenza_irrelevant_events_leave_model_identical ... ok
test l5_check6_base_incoerente_is_head_match ... ok
test l5_check6_l5_mixed_with_other_tags_is_schema ... ok
test l5_check6_osservazioni_on_non_sense_is_schema ... ok
test l5_check6_malformed_osservazioni_is_schema ... ok
test l5_check6_malformed_l5_payload_is_schema ... ok
test l5_check7_second_predict_replaces_anchor ... ok
test l5_check7_l5_in_base_prefix_gives_zero_decisions ... ok
test l5_check7_window_equal_observation_no_update ... ok
test l5_check7_l5_event_closes_l2_window_no_side_effects ... ok
test l5_check7_window_update_mismatch_one_decision ... ok
test l5_check_osservazioni_non_f1_is_schema ... FAILED
test l5_check8_zero_draw_and_deterministic_decisions ... ok

failures:

---- l5_check_osservazioni_non_f1_is_schema stdout ----

thread 'l5_check_osservazioni_non_f1_is_schema' (310942) panicked at tests/l5_social.rs:385:35:
called `Result::unwrap_err()` on an `Ok` value: SocialView { head_hash: "fd8a4299797045e9c35c8691fa68756713340dffe48859f265fd974854b0faaa", models: {} }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    l5_check_osservazioni_non_f1_is_schema

test result: FAILED. 19 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `--test l5_social`

Exited with code 101
```

- Primo commit del task `a2e6aee` = **solo** `tests/l5_social.rs`. → **VERDE**.
- Test effettivamente **esercitato** e **rosso** per asserzione (FAILED, `unwrap_err()` su `Ok`, exit 101), non per compilazione. → **VERDE**.
- `77070b7` è il secondo commit (patch) e porta la suite a verde: ordine test → codice rispettato.

## A6. SHA-256

```
shasum -a 256 notes/L5_legge_sociale.md
0d0e7e8c42d4c718545febb2796ed3c27d28ba2cf0e46690f280d6faa6e391b6  notes/L5_legge_sociale.md

shasum -a 256 .opencode/routing.md
bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177  .opencode/routing.md
```

Entrambi **identici** ai valori del gate precedente (legge `0d0e7e8c…`, policy `bf298cb7…`): legge e policy immutate dal mini-ciclo patch.

## A7. Stato del checkout principale

`git status --porcelain=v1` (checkout principale, nessun file versionato modificato):

```
?? notes/L3_domande_memoria.md
?? notes/L4_domande_identita.md
?? notes/mnemosyne_backlog.md
?? reports/audit_l5_impl.md
?? reports/gate_l5_impl.md
?? reports/l5_impl_sociale.md
?? reports/routing_l5_impl.md
```

Report producer `reports/l5_impl_sociale.md` non committato e con l'append della patch R1 presente (`grep -n "R1"` → riga 99: `## Aggiunta sezione: patch R1 (rilievo audit — test-first, 2 commit)`; riga 146: catena `4f9e834` (verdi) → `a2e6aee` (rossi R1) → `77070b7` (verdi)).

## A8. Verdetto del gate patch R1

**VERDE** (gate procedurale).

- debug: **165/165 passed, 0 failed** (146 L0–L4 + 19 L5, nuovo `l5_check_osservazioni_non_f1_is_schema` ok)
- release: **165/165 passed, 0 failed**, identico al debug
- perimetro `4f9e834..HEAD`: **2 file** (`src/social.rs` +20/−3, `tests/l5_social.rs` +42), nessun tocco a `lib.rs`, altri moduli, leggi, SPEC, policy; stat separati `a2e6aee` (1 file, 44+) e `77070b7` (2 file, 17+/5−)
- test-first: `a2e6aee` **solo test**, rosso a runtime provato (FAILED, exit 101)
- HEAD: `77070b7d747f5d07eeba4110dea35b3b8b019343`
- sha256: legge `0d0e7e8c42d4c718545febb2796ed3c27d28ba2cf0e46690f280d6faa6e391b6`, policy `bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177`
- session ID: `ses_f15c43c5-2a04-40b8-9987-168579785f1e`
- modello: `opencode/deepseek-v4.1-flash-free`, classe `light`, variante nessuna (default catalogo), canale free «Free-first»

Due NOTA dichiarate, che non cambiano il verdetto procedurale: (i) §A4, `77070b7` rimuove 2 righe di test dichiarate come correzione meccanica autorizzata — ratifica all'orchestratore/re-audit; (ii) §A1, `git rev-parse HEAD` negato dalla sandbox, sostituito da `git log -1 --format=%H`.

Questo gate **non** chiude il ciclo: la conformità legge-vs-codice della patch (R1 chiuso? nessuna regressione?) resta al re-audit (Fase C). Nessuna modifica a codice/test/note/policy; nessun commit, nessun push.

---

# Sezione B — Gate chiusura/copertura R1-ter (procedurale) — VERDE

- **modello**: `opencode/deepseek-v4.1-flash-free` — classe `light`, variante: **nessuna** (default del catalogo), canale free «Free-first», 01/11/2026 (come da mandato Router)
- **policy**: `.opencode/routing.md` v1 **ATTIVO** (letta; scelta del modello documentata nel mandato → nessuna ereditarietà implicita)
- **sessione tester (Mnemosyne)**: `ses_2cd46ae7-fb92-4011-8abe-360dd7d5458c` (progetto `opencode:Ariadne`), aperta con `memory_open_session("L5 gate chiusura/copertura R1-ter")`
- **memorie richiamate**: `memory_context("L5 test")` → `mem_23ee6360-e5ff-4f01-b636-5b3d2645ed51` (episode ciclo L4, pattern «commit di copertura post-gate, test verde alla stesura, NON test-first, autorizzato» — è esattamente il pattern di `cdcdfce`), `mem_e096d17e-6849-4b97-a7eb-26d10cc6598f` (episode completamento L4, conteggi suite), `mem_616ad586-4307-48c3-9594-ee1c28b3bb52` (episode gate compatto L0–L3), `mem_1eca95a3-bd5a-481c-8407-138c3a2eb125` (decision cutoff L4). `memory_search("L5 violazione")` → **0 items** (nessun pattern di violazione L5 pregresso).
- **memorie proposte**: nessuna (nessun pattern nuovo: il caso rientra nel pattern di copertura già memorizzato `mem_23ee6360…`).
- **legge vigente**: `notes/L5_legge_sociale.md` (commit `23a7104`), emenda `spec/L5_SPEC_v0.1.md`; riferimenti citati dal mandato L5.1/L5.2
- **revisione verificata**: HEAD `cdcdfce6be5726430fc6497ac5b2c8924974ccdb` su `77070b7d747f5d07eeba4110dea35b3b8b019343` (gate patch R1, sezione A)
- **scope**: gate **procedurale** di chiusura/copertura: esecuzione, conteggi, perimetri, hash. **Non** giudica la conformità legge-vs-codice (Fase C) e **non** ratifica la smentita di R1-ter, che è decisione dell'orchestratore/re-audit.

## B1. HEAD

`git rev-parse HEAD` → **NEGATO** dalla sandbox (`Permission denied: shell`). Alternativa dichiarata usata:

```
git log -1 --format=%H
cdcdfce6be5726430fc6497ac5b2c8924974ccdb
```

Confermato da `git show --stat --format=%H cdcdfce`:

```
cdcdfce6be5726430fc6497ac5b2c8924974ccdb

 tests/l5_social.rs | 43 +++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 43 insertions(+)
```

e da `git worktree add <tmp> cdcdfce…` → `HEAD is now at cdcdfce L5: copertura post-gate della simmetria l5-key (smentita R1-ter, test verde)`.

## B2. Suite completa — debug (worktree pulito)

Worktree: `git worktree add /private/var/.../T/opencode/ariadne_gate_l5_r1ter cdcdfce6be5726430fc6497ac5b2c8924974ccdb` (detached HEAD), `git status --porcelain` → **nessuna riga** (pulito), `git log -1 --format=%H` → `cdcdfce6be5726430fc6497ac5b2c8924974ccdb`.

`cargo test` (righe testuali per binario):

```
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/bin/l0_replay.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_c1_genesis.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_cycle.rs
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_event_log.rs
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_l1_l2_pipeline.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_n3_canonical_string.rs
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_n3_line_canonical.rs
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_prng.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_replay_process.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
     Running tests/l0_state.rs
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l1_view.rs
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/l2_selection.rs
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/l3_memory.rs
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l4_identity.rs
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/l5_social.rs
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   Doc-tests ariadne
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Totale: 4+0+2+11+18+1+13+10+7+6+9+11+16+14+24+20 = **166 passed, 0 failed** (146 L0–L4 + **20 L5**) → **atteso rispettato**.

## B3. Suite completa — release (stesso worktree, stesso commit)

`cargo test --release`:

```
     Running unittests src/lib.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/bin/l0_replay.rs
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_c1_genesis.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_cycle.rs
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_event_log.rs
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_l1_l2_pipeline.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_n3_canonical_string.rs
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_n3_line_canonical.rs
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_prng.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l0_replay_process.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/l0_state.rs
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l1_view.rs
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l2_selection.rs
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l3_memory.rs
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l4_identity.rs
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/l5_social.rs
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests ariadne
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Totale: **166 passed, 0 failed** — **identico al debug** (stessi 16 binari, stesse cardinalità).

## B4. Perimetro `77070b7..HEAD`

```
git diff --stat 77070b7 HEAD
 tests/l5_social.rs | 43 +++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 43 insertions(+)

git diff --numstat 77070b7 HEAD
43	0	tests/l5_social.rs

git diff --name-status 77070b7 HEAD
M	tests/l5_social.rs
```

**Un solo file, `tests/l5_social.rs`, +43/−0.** Quindi restano **INVARIATI** (nessuna riga toccata): `src/social.rs`, `src/lib.rs`, ogni altro modulo in `src/`, `notes/` (leggi), `spec/`/SPEC, `.opencode/routing.md` (policy), `Cargo.toml`/`Cargo.lock`. **Nessun codice di produzione nel perimetro** → Fase C non prevista, coerente col mandato.

`git status --porcelain` sul checkout principale — nessun file versionato modificato, solo untracked:

```
?? notes/L3_domande_memoria.md
?? notes/L4_domande_identita.md
?? notes/mnemosyne_backlog.md
?? reports/audit_l5_impl.md
?? reports/gate_l5_impl.md
?? reports/l5_impl_sociale.md
?? reports/routing_l5_impl.md
```

L'append del producer in `reports/l5_impl_sociale.md` è presente e **non committato** (`grep -n "R1-ter"` → riga 150: `## Aggiunta sezione: copertura simmetria l5-key + smentita R1-ter`; riga 152: `- **Rilievo chiuso:** R1-ter (re-audit, reports/audit_l5_impl.md append)`). Coerente col mandato.

## B5. Test nuovo effettivamente esercitato

Presente e `ok` in **entrambi** i run:

```
debug   tests/l5_social.rs: test l5_check5_l5_key_pathologies_symmetric_schema ... ok
release tests/l5_social.rs: test l5_check5_l5_key_pathologies_symmetric_schema ... ok
```

20 test L5 **distinti** nel binario `l5_social` (debug, elenco integrale):

```
l5_check1_genesis_only_models_empty_head_hash_is_genesis ... ok
l5_check1_replay_is_byte_per_byte ... ok
l5_check2_frontiera_no_l5_writes_in_log ... ok
l5_check3_predict_source_seqs_and_no_model ... ok
l5_check4_divergenza_irrelevant_events_leave_model_identical ... ok
l5_check4_divergenza_relevant_observation_changes_model ... ok
l5_check5_l5_key_pathologies_symmetric_schema ... ok
l5_check5_second_order_composition_equals_direct_prediction ... ok
l5_check6_base_incoerente_is_head_match ... ok
l5_check6_l5_mixed_with_other_tags_is_schema ... ok
l5_check6_malformed_l5_payload_is_schema ... ok
l5_check6_malformed_osservazioni_is_schema ... ok
l5_check6_osservazioni_on_non_sense_is_schema ... ok
l5_check7_l5_event_closes_l2_window_no_side_effects ... ok
l5_check7_l5_in_base_prefix_gives_zero_decisions ... ok
l5_check7_second_predict_replaces_anchor ... ok
l5_check7_window_equal_observation_no_update ... ok
l5_check7_window_update_mismatch_one_decision ... ok
l5_check_osservazioni_non_f1_is_schema ... ok
l5_check8_zero_draw_and_deterministic_decisions ... ok
```

Il test nuovo asserisce la **simmetria** sulle due API (`evaluate_social` *e* `build_social`) su tre casi (`l5 = Value::Int` su ACTION; `l5 = "foo"` fuori dominio; `l5 = Value::Int` su SENSE non-ACTION), tutti attesi `SocialError::Schema` (diff `cdcdfce`, righe aggiunte 536–578 di `tests/l5_social.rs`). Il caso (iii) è precisamente la classe del rilievo R1-ter su `src/social.rs:580`.

**NOTA procedurale (§A4, pattern già registrato)**: `cdcdfce` è un **commit di copertura post-gate**, non test-first — dichiarato tale nel messaggio («COPERTURA DICHIARATA, NON test-first … test VERDE ALLA STESURA … NESSUN file di produzione è toccato») e coperto dal precedente `8d82f47` (L4) / `0f01211` (L3) e dall'autorizzazione «test subito» del 31/10/2026 estesa dall'orchestratore il 01/11/2026 (opzione (a)). Il vincolo test-first del ciclo L5 è già stato soddisfatto nella catena `4374e91` (rossi) → `4f9e834` (implementazione) → `a2e6aee` (rossi R1) → `77070b7` (patch), verificata nella sezione A. **Non è una violazione**: la regola «primo commit del task = solo test» resta soddisfatta per il task L5 e il commit in esame è un'aggiunta di copertura, che per definizione non può essere rossa. Questo gate **non** ratifica l'autorizzazione: la ratifica resta all'umano/orchestratore.

## B6. SHA-256 (legge e policy)

```
shasum -a 256 notes/L5_legge_sociale.md .opencode/routing.md
0d0e7e8c42d4c718545febb2796ed3c27d28ba2cf0e46690f280d6faa6e391b6  notes/L5_legge_sociale.md
bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177  .opencode/routing.md
```

Entrambi **identici** ai valori della sezione A (§A6) e del gate precedente: legge `0d0e7e8c…`, policy `bf298cb7…`. Legge e policy **immutate** dal mini-ciclo patch R1 e da questo commit di copertura.

## B7. Verdetto

**VERDE** (gate procedurale chiusura/copertura R1-ter).

- debug: **166/166 passed, 0 failed** (146 L0–L4 + 20 L5)
- release: **166/166 passed, 0 failed**, identico al debug
- perimetro `77070b7..HEAD`: **1 file** (`tests/l5_social.rs` +43/−0; `--numstat` → `43 0`), `src/social.rs` / `src/lib.rs` / altri moduli / leggi / SPEC / policy / Cargo **invariati**
- test nuovo `l5_check5_l5_key_pathologies_symmetric_schema` presente e `ok` (20 test L5 distinti)
- HEAD: `cdcdfce6be5726430fc6497ac5b2c8924974ccdb`
- sha256: legge `0d0e7e8c42d4c718545febb2796ed3c27d28ba2cf0e46690f280d6faa6e391b6`, policy `bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177`
- session ID: `ses_2cd46ae7-fb92-4011-8abe-360dd7d5458c`
- modello: `opencode/deepseek-v4.1-flash-free`, classe `light`, variante nessuna (default catalogo), canale free «Free-first»

**Rilievi residui**: nessun rosso. Due NOTA, che non cambiano il verdetto procedurale: (i) §B5, `cdcdfce` è copertura post-gate e **verde alla stesura** (non test-first), dichiarata nel messaggio e coerente col pattern `8d82f47`/`0f01211`; la ratifica dell'autorizzazione resta all'orchestratore/umano; (ii) §B1, `git rev-parse HEAD` negato dalla sandbox, sostituito da `git log -1 --format=%H`. La smentita tecnica di R1-ter (`replay_upto` valida `l5` prima di `social.rs:580`) è **giudizio di conformità** e resta al re-audit (Fase C): questo gate ha verificato solo che il caso sia **coperto ed esercitato**. Nessuna modifica a codice/test/note/policy (solo questo append); nessun commit, nessun push.
