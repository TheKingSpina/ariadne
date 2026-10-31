# Gate Tester — fix L2 R1/R2/R4 (event-local edge activation + config obbligatoria)

modello: `opencode/deepseek-v4.1-flash-free` (classe `light`, nessuna variante — default di catalogo). Session ID: `ses_34f1cbab-7a2d-4b10-aa89-55676ef84557`.
Policy: `.opencode/routing.md` v1 (stato ATTIVO, 31/10/2026), SHA-256 `bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177`.
Legge verificata: `notes/L2_legge_selezione.md` (emendamento L2.27, commit `51e69f6`, firma umana «Lettura A — L2.3/L2.13», righe 210–238), SHA-256 `99b31942c94402c4ab4ccd28a908ea87b0ff9132d1df81a8a9c4983012767f7d`.
Report univoco di questa unità: questo file. Nessun altro report toccato.

Memoria Mnemosyne: sessioni aperta (`memory_open_session`), richiamate `mem_8c87fe0c-133d-4715-9681-4dbcae9f6b0b`, `mem_ff84d6c9-caf5-4a4f-9866-f33379e2d042` (audit L3 con pattern "provenienza test-first", qui non applicabile: primo commit solo test, nessuno stub), `mem_f1b05e04-85bc-4a91-b506-3a2f7ec8293a`. Nessuna memoria proposta.

## Comandi eseguiti (una esecuzione ciascuno, nessun retry usato)

| # | comando (workdir `/Users/spina/Ariadne`) | esito |
| --- | --- | --- |
| 1 | `git rev-parse HEAD` | **NEGATO** dalla sandbox (`Permission denied: shell`) — sostituito con lettura di `.git/HEAD` (`ref: refs/heads/main`) + `.git/refs/heads/main` |
| 2 | `git log --oneline -5` | ok (sotto) |
| 3 | `git diff --stat 51e69f6..HEAD` | ok (sotto) |
| 4 | `git diff --stat 2a9e18e..HEAD` | ok (sotto) |
| 5 | `git status --porcelain` | ok (sotto) |
| 6 | `cargo test` | ok — 122/122 |
| 7 | `cargo test --release` | ok — 122/122 |
| 8 | `git show --stat 6cdf7b4`, `git show 6cdf7b4 -- tests/l2_selection.rs`, `git show --stat 610b43c`, `git show 610b43c -- src/selection.rs`, `git show 610b43c -- tests/l2_selection.rs` | ok |
| 9 | `shasum -a 256 .opencode/routing.md src/selection.rs tests/l2_selection.rs notes/L2_legge_selezione.md` | ok |

Metodo per i tre test del punto 8 del mandato: **output del run completo** (debug e release), non run filtrati; tutti e tre compaiono `... ok` in entrambi i run.

## Identità della revisione

`git rev-parse HEAD` negato; prova alternativa: `.git/HEAD` = `ref: refs/heads/main`, `.git/refs/heads/main` = `610b43ce57a13234c9e54b7d58967d9d0ff45577` → **HEAD = 610b43ce… (atteso, confermato)**. Traccia:

```
610b43c fix L2 event-local edge activation and require config
51e69f6 legge L2: emendamento L2.27 (riprova dell'arco, Lettura A)
6cdf7b4 test L2 edge locality and missing window config
2a9e18e legge L3: emendamento E3 (vince L3.8) + abilita subagent_depth 2
b2770bd routing: policy v1 attiva con Router MiMo
```

## Risultati suite

`cargo test` (debug) — totale **122 passed; 0 failed; 0 ignored**:

```
lib unittests ......... 4    l0_n3_line_canonical ... 10
bin l0_replay ......... 0    l0_prng ............... 7
l0_c1_genesis ......... 2    l0_replay_process ..... 6
l0_cycle .............. 11   l0_state .............. 9
l0_event_log .......... 18   l1_view ............... 11
l0_l1_l2_pipeline ..... 1    l2_selection .......... 16
l0_n3_canonical_string  13   l3_memory ............. 14
doc-tests ............. 0
```

`cargo test --release` — totale **122 passed; 0 failed; 0 ignored**, stessi conteggi per suite (l2_selection 16, l3_memory 14).

## I tre test richiesti

| voce | test | esito |
| --- | --- | --- |
| a | `distinct_edge_events_activate_only_their_own_endpoints` (`tests/l2_selection.rs:166`) | **VERDE** (debug e release) |
| b | `in_window_select_without_config_is_error` (`tests/l2_selection.rs:178`) | **VERDE** (debug e release) |
| c | `repeated_edge_reactivates_endpoints_on_each_event` (`tests/l2_selection.rs:186`, nome identificato leggendo il file) | **VERDE** (debug e release) |

Test (c) è il test di riprova reiterata L2.27: due eventi `edges` (seq 2 e seq 3) con lo stesso arco A→B, attesi `Select{at:3,tick:2,ups:[],downs:[2,2]}` e `Select{at:4,tick:3,...}`; conforme a L2.27 righe 213–218 («una dichiarazione successiva dello stesso arco è anch'essa una menzione»). È stato **committato insieme all'implementazione** (13 righe in `610b43c`), non nel commit test-first: dichiarato dal mandato come test di fissaggio e dalla legge (righe 235–238) come pin di non-regressione — coerente, non lo conteggio come prova di rosso.

## Perimetri

```
$ git diff --stat 51e69f6..HEAD
 src/selection.rs      | 32 +++++++++++++++++++++++++++++---
 tests/l2_selection.rs | 13 +++++++++++++
 2 files changed, 42 insertions(+), 3 deletions(-)
```

```
$ git diff --stat 2a9e18e..HEAD
 notes/L2_legge_selezione.md | 32 ++++++++++++++++++++++++++++++++
 src/selection.rs            | 32 +++++++++++++++++++++++++++++---
 tests/l2_selection.rs       | 33 +++++++++++++++++++++++++++++++++
 3 files changed, 94 insertions(+), 3 deletions(-)
```

Commit 2: **solo** `src/selection.rs` + `tests/l2_selection.rs` → perimetro rispettato. Totale mandato: solo selezione, suoi test e la nota di legge firmata → `src/memory.rs`, API pubbliche, policy e altre note invariate.

`git show --stat 6cdf7b4`: solo `tests/l2_selection.rs` (+20), messaggio con autorizzazione umana «autorizzo il fix col percorso consueto test-first, voglio un caso multi-arco», 31/10/2026. Nessuno stub, nessun export, nessun file di produzione: **ordine dei commit conforme**.

## git status --porcelain (integrale)

```
 D BLOCKER.md
?? notes/L3_domande_memoria.md
?? notes/mnemosyne_backlog.md
?? reports/L1_test_report.md
?? reports/L2_delta_pipeline_report.md
?? reports/L2_reaudit_0eb3be1.md
?? reports/L2_test_report.md
?? reports/L3_reaudit_4f353239.md
?? reports/L3_test_report.md
?? reports/L3_test_report_4f353239.md
?? reports/compact_review_l0_l3_b2770bd.md
?? reports/compact_test_l0_l3_b2770bd.md
?? reports/routing_activation_v1.md
?? reports/routing_compact_l0l3_b2770bd.md
?? reports/routing_fix_l2_r1_r2.md
```

Nessun file non tracciato fuori da `notes/` e `reports/`. Unica deviazione del working tree: **`BLOCKER.md` cancellato e non committato** (` D`), nota e non codice; non incide sulla suite ma va sanata (commit o ripristino) da chi ne ha titolo. Il file BLOCKER non è presente, come richiesto.

## SHA-256

```
bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177  .opencode/routing.md
62130ad4d2a733296117750a6e9598fea6ad996041a2b1c1181fabfca5100cb3  src/selection.rs
c6e7b81ee708f36e6b9e0203f19fd2a8efc44cf0decbe2432de82db22a78e011  tests/l2_selection.rs
99b31942c94402c4ab4ccd28a908ea87b0ff9132d1df81a8a9c4983012767f7d  notes/L2_legge_selezione.md
```

## Conformità ai rilievi (prova per diff, `git show 610b43c -- src/selection.rs`)

- R1 (event-local): `if let Some(v)=e.payload.get("edges"){mentioned.extend(edge_mentions(v,&view)?);}` sostituisce il ciclo su `&view.edges` che attivava tutti gli archi della vista a ogni evento `edges` → ogni evento attiva solo i propri endpoint, con `origin_seq` L1 preservato (provenienza `seq`).
- R2/R4 (config obbligatoria): `let tau=config.ok_or(L2Error::Config)?.1;` sostituisce `config.map(|x|x.1).unwrap_or(1)`, e l'attivazione usa `config.ok_or(L2Error::Config)?` → `build_active` su `select` in finestra senza `config` = `L2Error::Config` (test b).
- Nessuna modifica a `src/memory.rs` né all'API pubblica; nessuna nuova dipendenza.

## Verdetto GATE: **VERDE**

122/122 debug + 122/122 release, 0 falliti; i tre test (a/b/c) verdi in entrambi i run; perimetro dei due diff come atteso; primo commit solo test con autorizzazione citata. Resta ROSSO/non sanato solo lo scostamento ` D BLOCKER.md` (nota, non codice) e il fatto che l'ordine test-first è ricostruito staticamente, non rieseguito (vedi limitazioni): non toccano il codice sotto gate.

## Limitazioni dichiarate

1. `git rev-parse HEAD` è stato negato dalla sandbox (permesso sul comando, non giudizio sul codice): hash ricavato dai file di ref Git, non dal comando richiesto.
2. La suite è stata eseguita sul working tree di HEAD, **non** su un worktree separato pulito: il mandato vieta scritture oltre al report su questo checkout e non ho creato worktree. Il working tree differisce da HEAD solo per la cancellazione non committata di `BLOCKER.md` (nota).
3. Il rosso iniziale dei test del commit `6cdf7b4` è **ricostruito dal diff**, non rieseguito: ricostruzione statica — pre-fix `e.payload.get("edges")` attivava tutti gli `view.edges` (seq 2 attiverebbe A,B,C,D → il test a fallisce) e `unwrap_or(1)` non produceva `L2Error::Config` (test b fallisce). Per chiuderla a runtime: `git worktree add <scratch> 6cdf7b4` poi `cargo test --test l2_selection` lì (atteso rosso su a e b) e rimozione del worktree.
4. Nessuna esecuzione nocapture/filtrata per singolo test: metodo dichiarato = output dei run completi.
