# Audit L4 — implementazione del sé (legge-vs-codice, Fase C)

modello: `opencode/glm-5.3-flash-free` (classe `light`, variante: nessuna / default di catalogo, canale free «Free-first», 31/10/2026)
policy: `.opencode/routing.md` **v1 ATTIVO** (31/10/2026, working tree) — mandato Router; sessione reviewer indipendente.
mandato: HEAD `280412f9ee77a19c05859d3c6d89fa8102013725` (commit 2 sopra `302e7e2`); gate tester VERDE sulla STESSA revisione (`reports/gate_l4_impl.md`, tester `opencode/deepseek-v4.1-flash-free`, 144/144 debug, 144/144 release, perimetro integro).
session ID: `ses_66dea74a-9133-48b8-9bd0-687bf227a8c1` (Mnemosyne `opencode:Ariadne`, corpus `efda4e4f-fbc4-48b3-abd5-0fb9ee3a5dc4:65`).
memorie esaminate: nessuna memoria APPROVATA L4 (rootcheck `memory_context("L4 …")` → solo atti L3 approvati `mem_2affa11c`, `mem_f1b05e04`, nessun conflitto col codice); `memory_pending_review` → 12 proposte in attesa, di cui le L4 (`mem_12863c3c`, `mem_e096d17e`, `mem_d6812cd0`, `mem_7e21f9e0`, `mem_37f7833a`, `mem_df416075`, `mem_1eca95a3`, `mem_1fc3a41f`, `mem_8de51b0d`, `mem_ffe6a92e`) audite a posteriori: il loro contenuto coincide col codice di `280412f`; nessun conflitto memoria (0 irrisolti).
metro: `notes/L4_legge_identita.md` (sha256 `cd7e9ce6b3d1c9355c8fa0eb59b8c7002cc461dff738ea3afdb671e8cbf0d9a5`; base `2ec39a0` + precisazione `sources` 171–177 emendata in `302e7e2`). Limiti: `git status/git diff` negati dalla sandbox del ruolo (già coperti dal gate; hash HEAD e perimetro re-verificati via `git log`/`git show --stat`). Nessun `cargo` eseguito (vincolo di ruolo): i conteggi derivano dal gate tester.

## Verdetto: **CONFORME** — 1 rilievo minore (non bloccante, edge case non testato)

## Esito punto per punto

| punto | esito | evidenza |
| --- | --- | --- |
| **L4.1** schema chiuso | **CONFORME** | `narrate` solo `at/text/sources`: `src/identity.rs:457` (`exact_keys`); `snapshot` solo `at/self_hash`: `:476`; `l4`+`l2`/`l3` insieme → Schema: `:447–450`; valore `l4` fuori dominio/non-Str → Schema: `l4_tag` `:279–284` e `:492`; scrittura non-ACTION → Schema: `:444–446` (test `l4_check4_state_with_l4_is_schema`). |
| **L4.2** view::build + head_hash | **CONFORME** | `view::build` e confronto `e.hash == v.head_hash` → `HeadMatch`: `:431–436`; `view::build` non ha variante `HeadMatch` (`src/view.rs:169–173`, solo Schema/Endpoint/Empty) → mapping totale a Schema conforme alla regola «`*::HeadMatch`→HeadMatch, resto→Schema»; nessun re-parse oggetti interni: sola lettura a campi limitati (`:237–272`, `:291–346`); nessuna chiamata a `build_active`/`build_memory` (`grep` su `src/identity.rs`: assenti). |
| **L4.3** rilevanze + template | **CONFORME** | prime dichiarazioni via filtro `origin_seq == seq` su vista: `:357–388`; select/summary: `:294–317`; promote/archive/recall: `:319–343`; controlli config/barrier esclusi (→ vuoto): `:254–256`, `:267–268`; id L1 hex64: `:261`; ordine interno lessicografico nodi `(kind,key)` poi archi `(rel,src_id,dst_id)`: `:358–364`, `:372–378` (sort di tuple); congiunzione `\n`: `:573`; liste crescenti/dedup/virgola senza spazi/`[]`: `render` `:348–350` e `sort_dedup` `:168–172`; controllo letterale template → sezione sotto. |
| **L4.4** documento canonico | **CONFORME** | preimage `{"head_hash":…,"schema_version":"1","statements":[…]}` chiavi lessicografiche, nessuno spazio: `:216–224`; membro `{"at":…,"sources":[…],"text":…}`: `:202–208`; sort stabile per `at`: `:193–195`, `:519`; `self_hash` = SHA-256 minuscolo del preimage privo di `self_hash` (pattern D16/L3.8): `sha` `:87–90` + `doc_digest` `:226–228`; head_hash = hash ultimo evento: `:521–525`. |
| **L4.5** zero draw | **CONFORME** | importi di `src/identity.rs:24–28`: solo `sha2`, `crate::*`, `view`; nessun `rand`/`std::time`/I/O (lettura integrale 1–634). Test `l4_check8` (stato PCG64 intatto) verde nel gate. |
| **L4.6** cutoff e ordine | **CONFORME** | cutoff = seq ultimo `l4` (`last_l4_seq` `:607–614`), combinato con il prefisso base per «nessuna decisione per eventi nella base» L4.8: `max(...)` `:546` (letture dichiarate nel modulo e nei report producer/tester); per evento rilevante seq>cutoff UNA coppia Narrate→Snapshot in ordine: `:562–601`; `at` = seq evento rilevante `:570`; `tick` = tick dell'evento ancorato pattern L2.18 (tick dell'evento che genera la revisione, la stessa semantica di `Decision` L2/L3): `:571`; self_hash del snapshot a head_hash = hash del narrate, calcolato su apposizione SIMULATA deterministica (`Event::seal` `:582–599`) resa necessaria da L4.9 («coppia senza intermissioni») e coerente con l'apposizione reale del chiamante (test `l4_check4_replay` verifica sim↔reale). **Precisazione `sources` (171–177) IMPLEMENTATA**: unione dedup-crescente seq citate = `[seq] + ups/downs` `:296–307`, `[seq]+expulsions` `:309–315`, `[seq]` per promote/archive (il template non cita origin) `:321–332`, `[seq, origin]` per recall `:333–341`, `[seq]` per estrazioni `:387`. |
| **L4.7** replay autorevole | **CONFORME**¹ | narrate autorevole L2.11 (testo com'è scritto): `:457–472`; `at` esistente, NON-l4, storicamente anteriore (`at < seq narrate`): `:461–470`; appaiamento stretto (stesso `at`, snapshot non preceduto → Schema; narrate chiusa da evento non appaiato → Schema; due narrate/snapshot di fila → Schema): `:453–456`, `:474–491`, `:497–500`; `self_hash` verificato duramente `:486–489`; narrate solitario SOLO come ultimo evento tollerata (pending residuo non sanzionato a fine loop): `:510`; errori interni (chiavi extra, liste non canoniche, campi mancanti/malformati l2/l3) → Schema: `:457–460`, `:476–481`, `validate_decision_fields` `:237–272` (test 8–16 tutti verdi nel gate). |
| **L4.8** API | **CONFORME** | `SelfDocument{head_hash,statements,self_hash}` `:48–53`; `Statement{at,sources,text}` `:37–42`; `SelfError{Schema,HeadMatch}` `:58–62`; `SelfBase{head_hash,log_len}` `:67–71`; `build_self(&Log)` `:518–532`; `evaluate_self(&Log,&SelfBase)` con coerenza di prefisso (`log_len==0`/log corto/hash posizione `log_len−1` → HeadMatch) `:538–544`; `Decision4{Narrate{at,tick,text,sources}, Snapshot{at,tick,self_hash}}` `:77–81`; `encode4(&Decision4) -> Vec<(String,Value)>` con liste `Value::Str` JSON canonico e `at` `Value::Int` nell'ordine della legge `:620–634` (test `l4_encode4_*` verdi). |
| **L4.9** interazione L2/L3 | **CONFORME** | evento `l4` è ACTION non-`l2` → chiude la finestra L2 (L2.25) per costruzione di `src/selection.rs:45–65` (nessun caso speciale lato L2); nessun side-effect sul cutoff L2 (conta solo select/summary: `selection.rs:73`) né L3; file: `tests/l4_identity.rs` `l4_check6_l4_event_closes_l2_window_without_side_effects` (verde). Nessuna decisione per eventi già narrati (skip l4/tag e ≤cutoff): `identity.rs:563–565`. |
| **L4.10** zero persistenza | **CONFORME** | nessun I/O/file in `src/identity.rs` (lettura integrale); il sé vive solo nel log; `lib.rs` aggiunge solo `pub mod identity;` (perimetro gate). |
| Test-first | **CONFORME** | `git show --stat c8ae8de`: unico file `tests/l4_identity.rs` (+835), nessun codice di produzione; rosso a runtime E0432/E0433 (exit 101) provato dal gate su worktree a `c8ae8de` (`gate_l4_impl.md` §7). |

¹ punto L4.7 con precisazione: vedasi rilievo sotto (edge case non testato, non bloccante).

## Controllo letterale dei template (legge 103–109 vs `src/identity.rs`), byte per byte

| template | legge | codice | esito |
| --- | --- | --- | --- |
| nodo | `a seq {seq} ho dichiarato l'elemento {kind} "{key}"` | `:368` `a seq {} ho dichiarato l'elemento {} "{}"` | **IDENTICO** (virgole/parentesi/maiuscole/virgolette incluse: `(origin_seq {origin})` per recall confermato) |
| arco | `a seq {seq} ho dichiarato l'arco {rel} da {kind_s} "{key_s}" a {kind_d} "{key_d}"` | `:383` | **IDENTICO** |
| select | `a seq {seq} ho selezionato: entrate [{ups}], uscite [{downs}]` | `:302` | **IDENTICO** (due punti, virgola senza spazio, `[]` vuoto) |
| summary | `a seq {seq} ho consolidato: espulsi [{expulsions}]` | `:313` | **IDENTICO** |
| promote | `a seq {seq} ho promosso l'identità {id}` | `:326–329` («ho {} l'identità {}», ramo «promosso») | **IDENTICO** |
| archive | `a seq {seq} ho archiviato l'identità {id}` | `:326–329` (ramo «archiviato») | **IDENTICO** |
| recall | `a seq {seq} ho richiamato l'identità {id} (origin_seq {origin})` | `:337–339` | **IDENTICO** |
| id L1 | hex minuscolo 64 | `hex64` `:127–131`, val test `node_id` `tests/l4_identity.rs:123–125` | **IDENTICO** |
| ordine/bibliografia | unione dedup crescente delle seq citate | verificata nei test `get(…) &[1,2]` / `&[1,3]` / `&[4]` / `&[1,6]` / `&[1,2,3]` | **CONFORME** |

Confronto diretto dei letterali: nessuna differenza rilevata (i letterali del codice usano i placeholder `{}` nelle stesse posizioni dei rimpiazzi di legge; accenti e caratteri speciali «identità» ecc. sono coperti byte-per-byte dagli `assert_eq!` esatti di `tests/l4_identity.rs:309–324`, `:413–415`).

## Le 8 correzioni test autorizzate (esito)

| # | correzione | esito | evidenza |
| --- | --- | --- | --- |
| i | check3_select `d[0]`→`d[2]` | **APPLICATA** | `tests/l4_identity.rs:337` |
| ii | `.clone()` ~676 | **APPLICATA** | `:673` (`head_after.clone()`) |
| iii | `appose_revision` sources F1 `"[1]"` | **APPLICATA** | `:107–111` |
| iv | check4_replay indici `[4]/[6]`, `events()[4]`, `log_len:5` | **APPLICATA** | `:427,429,442–444` |
| v | `chunks(2)` in check3_all | **APPLICATA** | `:288` |
| vi | check8 `events()[0]` | **APPLICATA** | `:774,781` |
| vii | check2_ii parentesi quadre F1 nel `mk` | **APPLICATA** | `:230` |
| viii | check2_i `at=4`, testo italiano esatto, `sources &[4]`, assert aggiornato | **APPLICATA** | `:197–202,216` |

Nessuna differenza oltre le 8 rilevata tra `c8ae8de` e `280412f`: restanti hunk del diff test (`+38/−17` per gate §4) sono le medesime 8 corrige.

## Checklist di uscita 1–8 (legge 255–280): copertura test `tests/l4_identity.rs`

1. Genesi → `l4_check1_*` (`:156–168`): statements vuote, head_hash genesi, preimage ricostruito a mano. **COPERTA**
2. Divergenza scoped → `l4_check2_i_*` (`:178–220`, assert esplicito NON-iniettività su head_hash/self_hash conforme a Q9) + `l4_check2_ii_*` (`:225–243`). **COPERTA**
3. Tutti i template, ordine interno, `\n`, `sources` → `l4_check3_all_templates_*` (`:254–325`), `l4_check3_select_list_*` (`:328–344`), `l4_check3_control_rementions_*` (`:349–388`). **COPERTA**
4. Replay/errori → `l4_check4_replay_*` (`:400–446`) + 9 test d'errore (`:451–653`), `unwrap_err()==Schema`, nessun panic. **COPERTA**
5. Cutoff/prefisso → `l4_check5_*` (`:662–705`), base intera → vuoto, HeadMatch casi. **COPERTA**
6. Interazione L2/L3 → `l4_check6_*` (`:713–736`). **COPERTA**
7. Determinismo → `l4_check7_*` (`:744–763`). **COPERTA**
8. Zero draw + suite L0–L3 verde → `l4_check8_*` (`:771–787`) + run gate 144/144 debug e release (122 pre-esistenti). **COPERTA**

## Rilievi (1)

**RILIEVO MINORE (non bloccante, edge case non testato)** — `src/identity.rs:501–504` vs `:506–508` e `:562–566`: in `replay` gli eventi NON-ACTION portanti `l2`/`l3` scavalcano `validate_decision_fields` (`continue` a `:501–504`), mentre il commento in `:506–508` dichiara la validazione «su TUTTI gli eventi» ed `evaluate_self` (loop `:562–566`, `statements_for` `:403–411`) tratta i medesimi eventi come decisionali senza discriminare il tipo: su un log patologico con evento non-ACTION portante `l2` malformato, `build_self` esce Ok mentre `evaluate_self` esce Schema (divergenza fra le due API su input identico). La legge L4.7 (righe ~183, ~209–210) fonda la validazione sull'iterazione «su TUTTI gli eventi» — l'interpretazione di L4 rimane subordinata alla semantica L2/L3, che per `l2` non discrimina il tipo (`src/selection.rs:47`) e per `l3` lo fa (`src/memory.rs:85`); tutto ciò solo per eventi fuori dai produttori normali L0/L2/L3. Non è uno dei punti chiusi (non tocca E1/E3/preimage/provenienza/L2.27/R1/R2). Da decidere in governance: esplicitare il comportamento con emendamento firmato oppure lasciare la lettura dichiarata. Da segnalare, non risolto qui.

(Zero ulteriori rilievi: nessun superamento del perimetro ammesso — 3 file; nessuna stocastica; nessun I/O; nessuna riscrittura proposta.)

## ESITO FINALE

**CONFORME** — implementazione L4 su `280412f` conforme alla legge punto per punto L4.1–L4.10, controllo letterale dei template byte-per-byte positivo, le 8 correzioni test autorizzate integralmente applicate e nessuna differenza oltre le 8, checklist di uscita 1–8 integralmente coperta da test verdi (gate 144/144 debug e release), test-first e primo commit solo test verificati autonomamente. Accettazione del livello NON dichiarata da questo audit: resta la firma umana di fine ciclo.
