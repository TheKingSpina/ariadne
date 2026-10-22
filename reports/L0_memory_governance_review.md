# L0 Memory Governance Audit Report

**modello:** opencode/nemotron-3-ultra-free

**Data:** 2026-10-22

**HEAD audito:** `a02b66a` (implementazione L0), commit di chiusura `19a0816`

---

## Sommario Esecutivo

| Punto | Verdetto | Note |
|---|---|---|
| 1. Accettazione L0 (commit `a02b66a`, 80 test, replay, audit) | **CONFORME** | Tutte le prove citate esistono e tornano; il report del reviewer è del 22 ottobre *prima* della firma v0.3, l'accettazione umana è posteriore. |
| 2. Opzione A → E1 v0.3 (payload compensativo a 3 chiavi, v0.3 prevale) | **CONFORME** | `spec/L0_SPEC_v0.3.md` E1 codifica esattamente le 3 chiavi; `src/log.rs:145-149` le implementa; la v0.2 ha nota di rimando in testa. |
| 3. Debiti noti (B4, `from_jsonl` senza chiamanti, CRLF, seed > i64::MAX) | **CONFORME** | Tutti e quattro verificati nel repo con `git show` e `grep`; corrispondono a quanto dichiarato. |
| 4. Ancora di richiamo v4 (legge v0.2 emendata da v0.3 firmata, budget ~1200 token, campo `context`, pending non entrano, orchestratore non committa, MCP sequenziali) | **PARZIALMENTE VERIFICABILE** | Firme, date, documenti, commit deroga: **verificati nel repo**. Budget token, campo `context`, pending invisibili, MCP sequenziali: **NON verificabili da qui** (dipendono dal servizio Mnemosyne, in timeout). |
| 5. Due deroghe al commit (`07abca5`, `19a0816` con messaggi che registrano la deroga) | **CONFORME** | Entrambi i commit esistono, i messaggi contengono la registrazione esplicita; `notes/L0_accettazione.md:53-56` e `RUN_L0.md:149-156` confermano esattamente due deroghe. |
| 6. Correzioni dell'ancora (v3 per meccanica falsa, v4 per togliere "unica per progetto") | **NON VERIFICABILE DA QUI** | Vivono nel servizio Mnemosyne; il repo riporta la narrazione in `RUN_L0.md:173-184` ma non posso controllare il servizio. La pratica è coerente con la delega; il registro del run la riporta correttamente. |

**Verdetto complessivo: CONFORME** su tutti i punti verificabili nel repository. I punti 4 (parzialmente) e 6 non sono verificabili da qui per dipendenza dal servizio Mnemosyne — non sono falsi, sono **non verificabili**.

---

## 1. Accettazione di L0

**Affermazione:** «L0 accettato dall'umano il 22 ottobre 2026 al commit `a02b66a`; 80 test verdi ESEGUITI dal tester, debug e release; replay bit-perfetto a due processi; audit del reviewer senza violazioni dopo la firma della v0.3.»

**Evidenza:**

- **Commit `a02b66a` esiste:** `git log --oneline` mostra `a02b66a L0 N3-bis: canonicità di RIGA...` come HEAD del codice L0.
- **80 test documentati con output integrale:** `reports/L0_n3_n4_test_report.md` §12.1 riporta l'output verbatim di `cargo test` e `cargo test --release` su worktree pulito a `a02b66a`.
  - Conteggio per binario (debug): lib 4, `l0_c1_genesis` 2, `l0_cycle` 11, `l0_event_log` 18, `l0_n3_canonical_string` 13, `l0_n3_line_canonical` **10**, `l0_prng` 7, `l0_replay_process` 6, `l0_state` 9 = **80**.
  - Conteggio per binario (release): identico, **80**. Exit code 0 in entrambi.
- **Replay bit-perfetto a due processi:** `tests/l0_replay_process.rs:34-63` lancia **due processi distinti** (`Command::new(BIN)`) e confronta `a.stdout == b.stdout` byte per byte (`:59-62`); `:81-95` ricarica con `Log::from_jsonl`, `verify()` e asserisce `replay.to_jsonl() == s`. Il report §12.2 mostra 6/6 verdi, exit 0.
- **Audit del reviewer:** `reports/L0_review_report.md` (modello `nemotron-3-ultra-free`, sessione `ses_8ffaa936...`, data 2026-10-22, HEAD `a02b66a`) conclude **RESPINTO** per una sola violazione: §3 della v0.2 (payload compensativo a 3 chiavi vs 2 prescritte). La violazione **si scioglie con la firma della v0.3** (E1), come nota il report §6: «L'addendum v0.3 proposto [...] non è firmato [...] Rispetto alla legge vigente (v0.2), la violazione è aperta e confermata.» La firma della v0.3 è del 22 ottobre (`spec/L0_SPEC_v0.3.md:121-122`), **posteriore** all'audit. `notes/L0_accettazione.md:36` cita correttamente: «Audit del reviewer: nessuna violazione *dopo* la firma della v0.3.»
- **Firma umana:** `notes/L0_accettazione.md:64` «- [x] umano — 22 ottobre 2026» con nota che la firma è dell'umano, la trascrizione dell'orchestratore.

**Conclusione:** CONFORME. Tutte le prove citate esistono, i numeri tornano (80 = 70 precedenti + 10 nuovi `l0_n3_line_canonical`), l'audit del reviewer è onesto sulla violazione pre-v0.3 e l'accettazione umana è successiva alla firma v0.3 che la risolve.

---

## 2. Opzione A → E1 della v0.3

**Affermazione:** «La decisione C1 (opzione A dichiarata dall'umano) è diventata E1 della SPEC v0.3, che prevale sulla v0.2; il payload compensativo ha TRE chiavi e la v0.3 è la legge.»

**Evidenza:**

- **SPEC v0.2 §3 (riga 22):** `spec/L0_SPEC_v0.2.md:22` prescrive `payload {compensates: seq, reason}` — **due chiavi**.
- **SPEC v0.3 E1 (righe 10-23):** `spec/L0_SPEC_v0.3.md:10-23` emenda esplicitamente: «Ogni errore si corregge con evento compensativo di tipo STATE: `payload {compensates: seq, reason, state}`, dove `state` è lo snapshot del PRNG copiato verbatim dall'ultimo STATE precedente... Il payload di §3 è da leggersi come *almeno* queste chiavi: `state` è aggiuntiva e obbligatoria.» — **tre chiavi**.
- **Nota di rimando in v0.2:** `spec/L0_SPEC_v0.2.md:3-4` «Emendata da `spec/L0_SPEC_v0.3.md` (firmato, 22 ottobre 2026). In caso di divergenza prevale la v0.3.»
- **Codice che genera il payload:** `src/log.rs:145-149` (funzione `compensate`):
  ```rust
  let payload = Payload::from_pairs(vec![
      ("compensates", Value::Int(comp_int)),
      ("reason", Value::Str(reason.to_string())),
      ("state", Value::Str(snapshot)),  // TERZA CHIAVE
  ]);
  ```
- **Test che lo confermano:** `tests/l0_event_log.rs:354-357` (`test_compensating_event_payload_and_history_intact`) asserisce il payload canonico con tre chiavi; `:390-392` (`test_compensating_payload_carries_most_recent_state_snapshot_exact`) idem; `:413-415` (`test_compensating_state_on_genesis_only_log_copies_genesis_snapshot`) idem.

**Conclusione:** CONFORME. La v0.3 E1 codifica esattamente l'opzione A (tre chiavi), la v0.2 ha la nota di rimando, il codice implementa tre chiavi, i test le verificano. La v0.3 firmata prevale.

---

## 3. Debiti noti

**Affermazione:** Quattro debiti: (a) B4: coppia `08b1536`→`b2790e9` con test che non ha mai compilato contro la sua implementazione, verde solo in `a298cf9` modificando il test; (b) `from_jsonl` non ha chiamanti di produzione; (c) qualunque CR grezzo rifiutato dal parser; (d) seed > `i64::MAX` rifiutato con `Error::Seed`.

### 3(a) B4 — rosso processuale

**Evidenza:**
- `git show --stat 08b1536`: commit di **solo test** (`tests/l0_cycle.rs` +429 righe), nessun `src/`. Il test usa `use ariadne::{cycle, ... Pcg64 ...}` ma a quel commit `Pcg64` non era re-exportato da `src/lib.rs` (vedi `git show 08b1536^:src/lib.rs` — nessun `pub use prng::Pcg64`).
- `git show --stat b2790e9`: implementazione che aggiunge `src/cycle.rs` e modifica `src/lib.rs` per esportare `pub mod cycle`; **non** re-esporta `Pcg64`.
- `git show a298cf9`: fix import nel test — cambia `use ariadne::{cycle, EventType, Log, Pcg64, State}` in `use ariadne::prng::Pcg64; use ariadne::{cycle, EventType, Log, State}`. **Solo il test modificato**, l'implementazione `b2790e9` non è toccata.
- `git show 45ad98d`: secondo test corretto dopo l'implementazione — aspettativa aritmetica errata (`comp.seq == 2` → `3`) nel test `test_compensating_payload_carries_most_recent_state_snapshot_exact`, codice corretto.

**Conclusione:** CONFORME. Il test di `08b1536` non compilava contro `b2790e9` (mancava `Pcg64` in scope); il verde arriva in `a298cf9` modificando **solo il test**. Due test corretti dopo: `a298cf9` (import) e `45ad98d` (aspettativa). La proprietà test-first è indebolita, non falsificata — registrata onestamente.

### 3(b) `from_jsonl` non ha chiamanti di produzione

**Evidenza:**
- `grep -r "from_jsonl" src/` → solo `src/log.rs:206` (definizione) e `src/log.rs:581` (unit test `genesis_line_parses`).
- `src/bin/l0_replay.rs` **non chiama** `from_jsonl` (emette solo `to_jsonl()`).
- `notes/L0_accettazione.md:47-48`: «`from_jsonl` non ha chiamanti di produzione. Il parser severo è per ora una funzione senza clienti: la severità è verificata dai test, non esercitata in esercizio.»
- `reports/L0_n3_n4_test_report.md:1025-1026`: «Chiamanti, letto tutto il repository (`grep from_jsonl`): `src/`: **nessun chiamante di produzione**.»

**Conclusione:** CONFORME. Verificato per ispezione completa del crate `src/`.

### 3(c) Qualunque CR grezzo rifiutato dal parser

**Evidenza:**
- `src/log.rs:207-209`: guardia all'ingresso di `from_jsonl`:
  ```rust
  if input.contains('\r') {
      return Err(Error::Schema("CR vietato: nel log canonico solo LF".to_string()));
  }
  ```
- `src/log.rs:524-528`: in `parse_string`, carattere di controllo grezzo `< 0x20` (incluso `\r` = 0x0D) rifiutato:
  ```rust
  if raw < 0x20 {
      return Err(Error::Schema(format!("carattere di controllo grezzo 0x{raw:02x} in stringa")));
  }
  ```
- Test: `reports/L0_n3_n4_test_report.md:976-978` mostra tre casi CRLF (2a/2b/2c) **RIFIUTATI** con `Schema("CR vietato: nel log canonico solo LF")`; caso 2d (payload con `\r` **escapato** come `\u000d`) **ACCETTATO** e riserializzato identico.

**Conclusione:** CONFORME. Il parser rifiuta ogni `\r` grezzo (sia come terminatore di riga che dentro stringhe non escapate), accetta solo `\r` escapato canonico (`\u000d`). Coerente con E4.

### 3(d) Seed > `i64::MAX` rifiutato con `Error::Seed`

**Evidenza:**
- `src/log.rs:75-81` in `Log::genesis`:
  ```rust
  let seed_int = i64::try_from(seed).map_err(|_| {
      Error::Seed(format!(
          "seed {seed} oltre i64::MAX: nel payload canonico il seed resta intero i64 \
           (contatore, non entropia), nessun cast silenzioso"
      ))
  })?;
  ```
- `src/bin/l0_replay.rs:113-115`: validazione CLI esplicita prima di chiamare `Log::genesis`:
  ```rust
  if seed > i64::MAX as u64 {
      eprintln!("l0_replay: seed {seed} oltre i64::MAX: Log::genesis lo rifiuta ...");
      std::process::exit(1);
  }
  ```
- Test: `tests/l0_c1_genesis.rs:27-59` — `test_genesis_accepts_seed_i64_max` (verde) e `test_genesis_rejects_seed_beyond_i64_max_no_event_no_wrap` (verde, verifica che `Err(Error::Seed)` e **nessun evento creato** nel percorso `Err`).
- SPEC v0.3 E3: `spec/L0_SPEC_v0.3.md:87-88`: «`seed`: intero `i64`. `Log::genesis` **rifiuta** `seed > i64::MAX` con errore esplicito e nessun evento creato. Nessun cast silenzioso, in nessun punto.»

**Conclusione:** CONFORME. Il rifiuto avviene in libreria (`Error::Seed`) e nel CLI (exit 1, stdout vuoto), con messaggio esplicito, nessun cast silenzioso, nessun evento creato.

---

## 4. Ancora di richiamo, contenuto v4

**Affermazione:** «La legge è v0.2 emendata da v0.3 firmata il 22 ottobre (umano + orchestratore), prevale la v0.3; `memory_context` ha budget ~1200 token, campo utile `context`, le pending non entrano, l'orchestratore non committa, le chiamate MCP di governance sono sequenziali.»

### Verificabile nel repository (CONFERMATO):

| Affermazione | Evidenza repo |
|---|---|
| v0.2 emendata da v0.3 firmata 22 ottobre | `spec/L0_SPEC_v0.2.md:3-4` (nota rimando); `spec/L0_SPEC_v0.3.md:121-122` (firme umano + orchestratore, 22 ottobre) |
| v0.3 prevale su v0.2 | `spec/L0_SPEC_v0.2.md:4` «In caso di divergenza prevale la v0.3»; `spec/L0_SPEC_v0.3.md:3-4` «Dove i due testi divergono, prevale questo» |
| Deroga orchestratore registrata | `git show --format=%B 07abca5` e `19a0816` contengono registrazione esplicita; `notes/L0_accettazione.md:53-56`, `RUN_L0.md:149-156` confermano esattamente due deroghe il 22 ottobre |
| Commit separati legge/infrastruttura/documentazione | `git log`: `a314d54` (legge v0.3), `4469f0f` (infrastruttura), `886ba57` (documentazione), `07abca5` (chiusura run), `19a0816` (accettazione) — nessuno tocca `src/` |

### NON verificabile da qui (dipende da servizio Mnemosyne, in timeout):

| Affermazione | Stato |
|---|---|
| `memory_context` ha budget ~1200 token | **Non verificabile** — parametro del servizio |
| Campo utile `context` (non `memories`/`items`/`results`) | **Non verificabile** — forma risposta del servizio |
| Le pending non entrano in `memory_context` | **Non verificabile** — logica del servizio (`RUN_L0.md:82-89` lo descrive) |
| L'orchestratore non committa (regola) | **Parzialmente verificabile**: la regola è dichiarata nei doc e nei messaggi di commit; le due deroghe sono registrate. Il rispetto *effettivo* fuori dalle deroghe non è auditabile dal repo. |
| Chiamate MCP di governance sequenziali | **Non verificabile** — comportamento runtime del server (`RUN_L0.md:122-124` riporta che 13 in parallelo hanno fallito, 13 in sequenza passate) |

**Conclusione:** PARZIALMENTE VERIFICABILE. Tutto ciò che lascia traccia nel repo (documenti firmati, commit, note) è confermato. I parametri runtime del servizio Mnemosyne non sono verificabili da qui — **dichiarati come non verificabili, non come falsi**.

---

## 5. Due deroghe al commit

**Affermazione:** «L'orchestratore ha committato due volte su istruzione esplicita dell'umano (`07abca5` e `19a0816`), e ogni deroga è registrata nel messaggio di commit; la regola resta che il commit è un gesto umano.»

**Evidenza:**

- **Commit `07abca5`:** `git show --format=%B 07abca5`:
  ```
  notes: chiusura del run L0 — firma v0.3, decisioni Mnemosyne, commit separati

  Registra: l'addendum firmato e verificato contro il codice; le 13 decisioni
  Mnemosyne (10 approvate, 3 respinte superate, 1 riscritta perche' conteneva una
  lezione valida); il protocollo di revisione delle pending come primo passo di
  ogni run; il golden doppio tenuto; i tre commit separati dal codice; la deroga
  dell'orchestratore alla propria whitelist read-only.
  ```
  → Contiene «la deroga dell'orchestratore alla propria whitelist read-only».

- **Commit `19a0816`:** `git show --format=%B 19a0816`:
  ```
  notes: accettazione di L0 — firma umana, richiamo verificato, minimax rimosso

  SECONDA DEROGA alla regola <<chi orchestra non scrive nella storia>>: commit
  eseguito dall'orchestratore su istruzione diretta dell'umano, come il primo
  (07abca5). La regola resta quella scritta in questo stesso file; la deroga e'
  registrata qui perche' il registro non menta su chi ha premuto il tasto.
  ```
  → Contiene «SECONDA DEROGA», riferimento esplicito al primo commit, regola ribadita.

- **Note confermano due deroghe:**
  - `notes/L0_accettazione.md:53-56`: «Deroga registrata: l'orchestratore ha committato su istruzione esplicita dell'umano, in due occasioni il 22 ottobre (i commit di chiusura e questo).»
  - `RUN_L0.md:149-156`: «Deroga dichiarata e registrata — decisione dell'umano: l'orchestratore ha eseguito i commit fuori dalla propria whitelist git read-only, su istruzione esplicita, in due occasioni il 22 ottobre (i quattro commit di legge/infrastruttura/documentazione, poi l'accettazione).»

**Conclusione:** CONFORME. Entrambi i commit esistono, entrambi i messaggi registrano esplicitamente la deroga (il secondo cita anche il primo), le note confermano esattamente due deroghe il 22 ottobre. La regola «il commit è un gesto umano» è ribadita in entrambi i messaggi e nelle note.

---

## 6. Correzioni dell'ancora (v3 e v4)

**Affermazione:** «L'orchestratore ha corretto due volte una memoria già approvata: v3 per correggere una meccanica falsa, v4 per togliere l'affermazione "unica per progetto".»

**Evidenza nel repo:**
- `RUN_L0.md:173-184` narra la vicenda:
  - v3 (`mem_d5d97885`): correzione della «diagnosi falsa» su `memory_context` (credeva servisse solo memorie `activation = "always"`, in realtà il campo utile è `context` con budget ~1200 token, mescola `always` e `on_demand`).
  - v4: correzione per togliere l'affermazione «unica per progetto».
  - `RUN_L0.md:181-184`: «Ancora di richiamo (`mem_d5d97885`, `always`, approvata dall'umano e corretta due volte — v3 per la meccanica falsa, v4 per l'unicità): legge vigente, meccanica del richiamo misurata, protocollo di revisione, regole processuali (MCP in sequenza; l'orchestratore non committa).»

**Stato:** **NON VERIFICABILE DA QUI**. Le memorie (`mem_d5d97885` v3, v4) vivono nel servizio Mnemosyne, che è in timeout. Il repo riporta la narrazione ma non posso ispezionare il servizio per confermare:
- L'esistenza delle versioni v3 e v4
- Il contenuto esatto di ciascuna versione
- Che la v3 correggesse una meccanica falsa
- Che la v4 togliesse «unica per progetto»

**Valutazione di coerenza:** La pratica **è coerente** con la nuova delega (l'orchestratore può correggere memorie approvate via `memory_correct`, con approvazione umana). Il registro del run (`RUN_L0.md`) la riporta correttamente come narrazione storica, inclusa l'ammissione dell'errore di lettura del campo `context` vs `memories` (`RUN_L0.md:165-172`).

---

## Sezione: Non verificabile da qui

Le seguenti affermazioni dipendono dal servizio Mnemosyne (server MCP), che al momento dell'audit non risponde (timeout). **Non sono false — sono non verificabili da questa posizione.**

1. **Budget token di `memory_context` (~1200 token, `tokensEstimated` ~1100, `excludedResults`, `requiredContextComplete` sempre true)** — `RUN_L0.md:169-171` lo riporta come misurato, ma non posso interrogare il servizio.
2. **Campo utile `context` nella risposta di `memory_context`** (non `memories`/`items`/`results`) — `RUN_L0.md:167-168` ammette l'errore di lettura del campo.
3. **Le pending non entrano in `memory_context`** (restituisce solo memorie approvate) — `RUN_L0.md:82-89`, `141-142` lo descrivono come meccanismo.
4. **Chiamate MCP di governance sequenziali** (13 in parallelo → 404, 13 in sequenza → OK) — `RUN_L0.md:122-124`.
5. **Esistenza e contenuto delle memorie corrette (v3, v4 dell'ancora `mem_d5d97885`)** — punto 6 sopra.
6. **Le 13 decisioni Mnemosyne (10 approvate, 3 respinte, 1 riscritta)** — `RUN_L0.md:135-139`, `notes/L0_accettazione.md` non le elenca nel dettaglio.
7. **Protocollo: revisione delle pending come primo passo di ogni run** — `RUN_L0.md:140-142`, `notes/L0_accettazione.md` non lo cita esplicitamente.

---

## Sezione: Rischi

Rischi concreti derivanti dalla governance delegata all'orchestratore, basati su quanto osservato nel repository:

| Rischio | Evidenza / Scenario concreto |
|---|---|
| **Approvazione delle proprie proposte senza contraddittorio** | L'orchestratore può chiamare `memory_propose` + `memory_review_decision` (accettare) in sequenza. `RUN_L0.md:122-124` mostra che le chiamate MCP vanno fatte in sequenza (parallelo fallisce), ma nulla impedisce all'orchestratore di proporre *e* approvare da solo se l'umano non interviene. Le 13 decisioni del 22 ottobre (`RUN_L0.md:135-139`) sono state «prese» dall'orchestratore — non è chiaro dal repo quante abbiano avuto intervento umano esplicito vs tacito. |
| **Correzione di memorie approvate senza traccia immutabile** | `memory_correct` modifica il contenuto; `RUN_L0.md:173-184` mostra due correzioni (v3, v4) sulla stessa memoria. Se il servizio non versiona in modo immutabile (append-only), la storia della correzione sparisce. Il repo narra la correzione, ma il servizio potrebbe non conservare le versioni precedenti. |
| **Pending accumulate = assenza operativa** | `RUN_L0.md:81-89`: «tredici proposte in attesa valgono zero, qualunque cosa dicano. Non sono decorazione in attesa di approvazione: sono, oggi, assenza.» Se l'orchestratore non porta le pending all'umano al primo passo del run successivo, restano invisibili per sempre. Il protocollo adottato (`RUN_L0.md:140`) lo mitiga, ma dipende dalla disciplina dell'orchestratore. |
| **Deroga al commit non tecnicamente impedita** | La whitelist git dell'orchestratore è read-only (`RUN_L0.md:58`), ma l'orchestratore ha committato due volte (`07abca5`, `19a0816`) su istruzione umana. La regola è solo documentale («la regola resta», `19a0816` messaggio). Nessun guard-rail tecnico impedisce all'orchestratore di committare codice di produzione. |
| **Falsa sicurezza dal richiamo** | `memory_context` restituisce un contesto tagliato a budget, mescolato per rilevanza (`RUN_L0.md:169-171`). L'orchestratore/agenti potrebbero credere di avere «tutta la memoria» quando ne hanno solo una fetta prioritaria. `requiredContextComplete: true` non significa «tutto», significa «il budget è stato riempito secondo la politica». |
| **Dipendenza da servizio esterno non auditabile** | Mnemosyne è un servizio MCP separato. Questo audit non può verificare: budget token, forma risposta, visibilità pending, sequenzialità MCP, versionamento memorie corrette. Se il servizio cambia comportamento, la governance dichiarata nei doc (`notes/`, `RUN_L0.md`) diventa stantia senza che il repo lo segnali. |

---

## Conclusione

Tutti e 6 i punti dell'audit sono **CONFORMI** rispetto a ciò che è verificabile nel repository git (codice, spec, report, note, commit, messaggi di commit). Due punti (4 parzialmente, 6 interamente) contengono affermazioni che dipendono dal servizio Mnemosyne — **dichiarate esplicitamente come non verificabili, non come violazioni**.

Nessuna discrepanza rilevata tra ciò che l'orchestratore afferma e ciò che il repository documenta. I debiti noti sono onestamente registrati. Le deroghe sono esplicitamente tracciate. La SPEC v0.3 firmata risolve l'unica violazione formale rilevata dal reviewer (payload compensativo a 3 chiavi).

**Verdetto finale: CONFORME** (su base repository; riserva su punti dipendenti da Mnemosyne).

---

**Firma del reviewer:** Questo report è un audit in sola lettura. Non costituisce accettazione. La firma spetta all'umano.