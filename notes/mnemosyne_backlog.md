# Backlog Mnemosyne — voci da registrare appena il server torna raggiungibile

Stato: Mnemosyne irraggiungibile per tutta la sessione del 23/10/2026 (timeout su
ogni chiamata MCP). Questa nota conserva ciò che va inscritto; il salvataggio
vera e propria resta da fare. Nessuna di queste voci è ancora persistata.

## 1. L1 accettatoUmano (23/10/2026)
- kind: decision, epistemicBasis: user_asserted
- L1 accettato dall'umano. Ciclo: test-first `c8c69d7`, implementazione `87bc9e9`.
- Gate tester verde: 11 test L1, 91 totali, report `reports/L1_test_report.md`.
- Audit reviewer conforme. Nessun rilievo di codice.

## 2. D14–D21 operative L1 (approvate con l'accettazione)
- kind: decision
- D14 head_hash = hash evento finale; D15 schema_version = "1";
  D16 view_hash = SHA-256 del documento canonico senza view_hash, no 0x1F;
  D17 nodo {id,key,kind,origin_seq}, arco {dst_id,origin_seq,rel,src_id};
  D18 arco unico per trip con seq minimo; D19 nodes prima di edges nello stesso
  evento; D20 builder presuppone log L0 verificato; D21 "vuoto" = genesi-only.

## 3. Cambio di pin agenti (23/10/2026)
- kind: decision, epistemicBasis: user_asserted
- Orchestratore → `opencode/glm-5.3-flash-free#high`; producer →
  `openai/gpt-6-luna#medium`. Autorizzato dall'umano per costo: GLM Free come
  producer principale, OpenRouter solo fallback, GPT coperto da sottoscrizione.
- Regola OpenRouter→produttore precedente rimossa dalle istruzioni: nessun
  override autorizzato per producer.
- Tester `opencode/nemotron-3-ultra-free`; reviewer
  `opencode/nemotron-3.5-super-vl-free` — invariati.

## 4. SPEC L1 v0.2 — addendum F1 (23/10/2026)
- kind: decision
- `nodes`/`edges` nel payload L0 sono **chiavi stringa** il cui valore è una
  stringa con array JSON canonico interno (UTF-8, chiavi in ordine di byte,
  nessuno spazio, escape minimo); il builder fa il parse; forma non canonica =
  errore; no estrazione implicita; eventi senza nodes/edges contribute zero.
  Schema L0 invariato (valori solo `i64`/stringa). File: `spec/L1_SPEC_v0.2.md`.
- Conformità: `src/view.rs` già implementa F1; nessun cambiamento di codice.

## 5. Rilievi reviewer da archiviare
- kind: episode
- R1: D14–D21 non pre-auditate — **risolto**: D14–D21 confermate dall'umano
  nella stessa sessione (vedi voce 2).
- R2: "modello tester ≠ pin reviewer" — **ritirato**: il reviewer scambio i pin
  dei due agenti; `reports/L1_test_report.md:3` dichiara correttamente il pin del
  tester (`nemotron-3-ultra-free`).

## 6. Backlog di sviluppo L2 (23–24/10/2026)
- Triage: `notes/L2_domande_selezione.md` (producer, commit `9c0f69442` grouse
  12 domande Q1–Q12 su tipi evento, salienza, IDLE_BARRIER, replay).
- Legge firmata: `notes/L2_legge_selezione.md` (L2.1–L2.12, commits
  `a3bff54` + `2414d4e`) su autorizzazione umana. Da audit del reviewer.
- Emendamenti risposta alle domande del producer (con delega umana):
  `80fe6d3` L2.13–L2.20, `1d58249` Decision Select/Summary,
  `c550132` L2.21 (chiavi extra/l2 ignoti = Schema/Config),
  `e162988` L2.22 (base in evaluate: coerenza di prefisso),
  `2ec9cb1` L2.23–L2.24 (cutoff decisionale; tie-break id).
- Implementazione: `c8f7ace` → fix `0eb3be1` → fix L2.24 test `ee68a5a` +
  implementazione `e7d4cc6`; suite L2 13 test verdi, totale 104.
- Gate tester `reports/L2_test_report.md` PASS (103/103) con NOTA L2.24;
  ri-audit reviewer `reports/L2_reaudit_0eb3be1.md` ROSSO su scala tie-break;
  fix producer test `ee68a5a` + implementazione `e7d4cc6` con test dedicato
  (`equal_score_eviction_prefers_newer_origin_then_greater_id`).
- Verifica finale reviewer **L2.24 Risolto: VERDE** (override autorizzato
  dall'umano, solo per quel dispatch: `openrouter/nvidia/nemotron-3.5-super-vl:free#thinking`,
  pin primario ripetutamente non partito). Audit loop chiuso; attesa
  accettazione umana del livello L2.

## 7. L2 ACCETTATO dall'umano (24/10/2026) — con estensione pipeline
- kind: decision, epistemicBasis: user_asserted
- Legge L2.1–L2.26 (commits `a3bff54` … `f9afda3`): emendamenti finali
  L2.25 (`716e971`, semantica finestra offline) e L2.26 (`7090ed2`,
  identità espulsi con origin_seq ripetuta); depennata la clausola
  abrogata di L2.23 (`f9afda3`).
- Implementazione: `c8f7ace` → `0eb3be1` → `e7d4cc6` (+test `ee68a5a`)
  → `9d378b9` (chiusura implicita emette Summary) → `78ba307`
  (replay downs in-window + deduzione id L2.26).
- Test integrazione L0→L1→L2: `tests/l0_l1_l2_pipeline.rs`
  (commits `53c9525`/`507e76f`, solo test).
- Gate tester: VERDE 105/105 (14 L2: 13 in `l2_selection.rs` + 1 di
  integrazione); reports `reports/L2_test_report.md` +
  `reports/L2_delta_pipeline_report.md`.
- Mini-audit finalizzato reviewer (24/10): CONFORME su tutti i 5 punti
  (L2.25, L2.26, fix `9d378b9`/`78ba307` senza toccare L0/L1, delta
  pipeline 105/105 verificato, tre NOTE — encode, D19, L2.21 — coperte);
  chiusura formale L2 CONFERMATA dal reviewer.
- **Accettazione umana registrata: 24/10/2026, il livello L2 è chiuso.**
- Verifica override L2.24 già proposta a Mnemosyne prima del blackout:
  `mem_2cd97de6-9cac-4931-be9d-25d0408598a2` (pending, `ses_b184e036…`).

## 8. SPEC federe consolidate (26/10/2026)
- kind: decision
- `spec/L0_SPEC_v0.4.md`, `spec/L1_SPEC_v0.3.md`, `spec/L2_SPEC_v0.3.md`
  (commit `f92bde3`): federe consolidate firmate che trascrivono
  integralmente la legge in vigore di ciascun livello implementato
  (L0 base+E1–E5; L1 v0.1+D1–D21+F1; L2 v0.1+articoli L2.1–L2.26).
  Nessun requisito nuovo; la v0.2 di L2 e le famiglie precedenti restano
  reperti storici con note di supersessione nel testo. Autorizzato
  dall'umano il 26/10/2026 («SPEC v0.3 per ogni livello implementato,
  traccia federe di quanto fatto»).

## 9. Cambio pin tester (30/10/2026)
- kind: decision, epistemicBasis: user_asserted
- `opencode/nemotron-3-ultra-free` non è più nel catalogo. Tester →
  **`nvidia/nvidia/nemotron-3-ultra-550b-a55b`** (provider Nvidia diretto,
  successione della famiglia ultra; emendato il 30/10 dopo primo tentativo
  errato con nemotron-3.5-super-vl-120b), autorizzato dall'umano il
  30/10/2026.
- Invariati: producer `openai/gpt-6-luna#medium`; reviewer
  `opencode/nemotron-3.5-super-vl-free`; orchestratore
  `opencode/glm-5.3-flash-free#high`. Nessun override/fallback autorizzato.

## Prossimo step sancito (quando Mnemosyne torna)
- Scriber: voci 1–6 come 4 memorie indipendenti (1,2 vanno in una solo),
  con arco "supersedes" a `mem_174f754c-…` (stato pre-chiarimi, superato);
  `mem_ad1d49b6-…` (fallback GLM→OpenRouter producer) è obsoleta per
  producer Luna: marked superseded.

## Esito 24/10 — il server è tornato, lavoro substantially completato
- **5 proposte pendenti**: già `accepted` a sistema
  (`mem_10142949`, `mem_12f72b9d`, `mem_cf9ce5ea`, `mem_949dc29c`,
  `mem_2cd97de6` + D14 `mem_e19f896f`).
- **Accettazione L2 iscritta e approvata**: `mem_7e387073-1f22-41fa-896d-b4e01130ddd5`
  (kind decision, lifecycle accepted, sessione `ses_fd126e0b`).
  Duplicato con refuso scartato: `mem_064dfe7a` (rejected).
- **Archi di supersessione APPROVATI**:
  `mem_10142949 → mem_174f754c` (RETRY L1/BLOCKER superato) e
  `mem_12f72b9d → mem_ad1d49b6` (fallback producer abrogato).
- **Risolto al 26/10 (server tornato operativo)**: i 3 auto-archi della
  proposta `mem_7e387073` verso legge/stato/verifica L2.24 sono stati
  ACCETTATI (`abe9d57c`, `a8671c0a`, `d8c00b20`); le due sessioni
  (`ses_fd126e0b` e `ses_dead8679`) sono state CHIUSE. È stata iscritta e
  approvata anche la memoria delle federe consolidate:
  `mem_337f73b9-ed1a-47bb-8905-7dd9f47d0a0d` con 3 archi verso
  `mem_10142949`/`mem_cf9ce5ea`/`mem_7e387073`. Backlog Mnemosyne
  **azzerato**: nessuna voce resta da iscrivere. Nota su copertura:
  D14–D21 e F1 erano già registrate in `mem_10142949` (accettazione L1),
  la voce 4 del vecchio elenco è quindi già coperta.
