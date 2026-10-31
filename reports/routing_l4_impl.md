# Registro Router — implementazione L4 Identità (test-first A/B/C)

## Identificazione

- **Mandato:** implementazione L4 Identità a percorso test-first completo
  (FASE A producer due commit → FASE B gate tester → FASE C audit reviewer),
  emesso dall'orchestratore al Router.
- **Policy di routing:** `.opencode/routing.md` v1 (ATTIVO, 31/10/2026),
  verificato per lettura prima dei dispatch. Hash SHA-256 policy: non
  ricalcolato in questo mandato (shell negata al Router); ultimo noto da
  gate precedente: `bf298cb7…c46a177` (da riconfermare in Fase B con
  `shasum`, come richiesto dal mandato).
- **Revisione Git:** HEAD dichiarato `2ec39a0`; verificato via lettura
  `.git/refs/heads/main` → `2ec39a0d1ddeaabc9e92aff996ffab7369f9e67c`.
- **Contratto di fase:** `notes/L4_legge_identita.md` (legge L4, commit
  `2ec39a0`, delega umana «scegli tu» 31/10/2026) — letta integralmente dal
  Router prima di ogni dispatch: L4.1–L4.10, terminologia firmata, checklist
  di uscita 1–8, rischi aperti, firma. Nessuna interpretazione autonoma
  consentita a nessun figlio: la legge tace → il figlio si ferma e chiede.
- **Canale di costo (firma umana «Free-first», 31/10/2026):** SOLO modelli
  free, classe `light` per tutti i figli. Nessun passaggio a Terra/Sol in
  autonomia: producer free inadeguato → fermata e resoconto.

## Verifiche catalogo (modelli free provider `opencode`, costo 0/0, active, 31/10/2026)

`glm-5.3-flash-free`, `deepseek-v4.1-flash-free`, `mimo-v2.6-flash-free`,
`flint-40b-free`, `nemotron-3.5-super-vl-free`, `beam-free`, `big-pickle`.

## Piano di dispatch (registrato prima delle chiamate)

| # | Fase | Ruolo | Classe | Modello richiesto | Variante | Motivazione |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | A — test-first (2 commit) | producer | light | `opencode/glm-5.3-flash-free` | nessuna prescritta → default del catalogo | Lavoro pesante di implementazione fedele a template ESATTI e a 10 articoli: GLM free ha già mostrato su questo progetto la massima precisione filologica (triage L4 con citazioni file:riga, review e re-review con rilievi puntuali). Alternativa free considerata: DeepSeek (più portato al gate procedurale). |
| 2 | B — gate test | tester | light | `opencode/deepseek-v4.1-flash-free` | nessuna prescritta → default del catalogo | Ruolo procedurale (run, conteggi, perimetri, hash): stesso free usato con esito VERDE nei gate precedenti. |
| 3 | C — audit | reviewer | light | `opencode/glm-5.3-flash-free` | nessuna prescritta → default del catalogo | Audit legge-vs-codice punto per punto: stesso free già validato nelle review; sessione separata dal producer. Nessuna diversità di famiglia richiesta dal mandato: dichiarata esplicitamente, produttore e auditore sono lo stesso modello in sessioni indipendenti. |

- **Serialità/foreground:** un figlio alla volta; nessun fallback; blocco di
  un figlio → fermata e resoconto con session ID; nessun retry.
- **Perimetri scrittura figli:** producer SOLO `src/identity.rs` (+ una riga
  `pub mod identity;` dove l'esposizione è già normale), `tests/l4_identity.rs`,
  `reports/l4_impl_identita.md`; due commit su main, nessun push.
  Tester/reviewer: soli rispettivi report in `reports/`. Nessuna modifica a
  `src/memory.rs`, `src/selection.rs`, leggi, policy o config.
- **Owner integrazione:** il producer (nessuno stitching separato: è lui il
  proprietario dei due commit).
- **Precondizioni di fase:** B solo dopo A (due commit + test verdi); C solo
  dopo gate VERDE sulla stessa revisione.

## Risultati

### Figlio 1 — producer (FASE A, ripresa — esito)

- Ruolo/modello/classe: producer, `opencode/glm-5.3-flash-free`, classe light, variante: nessuna (default catalogo). Stessa sessione del mandato 1: `ses_e6c53a29affea4AdUkfEiSmo0M`.
- Sessione Mnemosyne: `ses_0d20fc46-005f-4ee9-90ab-02f51f41c393` — **chiusa** dal figlio (job `job_6a8c8b49-…`). Proposte memoria in attesa (5, non tipizzabili dal Router, non accettate): `mem_8de51b0d-…` (sources con parentesi), `mem_1fc3a41f-…` (simulazione self_hash), `mem_1eca95a3-…` (cutoff), `mem_df416075-…` (replay appaiamento/digest), `mem_37f7833a-…` (blocco + contenuto del report non scrivibile).
- **Esito: SECONDO BLOCCO, corretto e conforme al mandato** (punto 2: «altri test incoerenti → fermati e riporta, nessuna correzione in autonomia»). Punto 0 ESERCITATO: `examples/l4scratch.rs` cancellato (verificato).
- **Commit 2: NON effettuato.** HEAD invariata `c8ae8deed8b9b95cd2c0cee5e005b383102b579e`.
- **Stato working tree (non committato, dichiarato dal figlio):** `src/identity.rs` COMPLETO e dichiarato legge-conforme (senza debug residuo), riga `pub mod identity;` in `src/lib.rs`, i soli 2 fix test autorizzati (`match &d[2]` a 325-339; `.clone()` a 676). Suite `l4_identity` debug: **16 verdi / 6 rossi**; suite completa debug+release non eseguita (fuori portata finché i test non sono sanati).
- **Esito check8 (richiesto dalla restituzione): IL TEST È IN DIFETTO** — legge `events().last()` dove la chiave `state` vive solo nel payload di genesi (panic inevitabile per qualsiasi implementazione che non muti il log, che è ciò che il test deve verificare); serve `events()[0]`. L4.5 zero-PRNG rispettata dall'implementazione.
- **NUOVE INCOERENZE TEST↔LEGGE (4), fermata obbligatoria, NON decise:**
  1. `appose_revision` appone `sources: "1"` senza parentesi; legge
     L4.1/L4.6/L4.7 e il test verde `l4_encode4_…` (`"[5]"`, `"[1,2,5]"`)
     esigono `"[1]"` → le apposizioni sono Schema (rossi: check2_i,
     check2_ii, check5);
  2. `l4_check4_replay`: indici snapshot `[3]/[5]` invece di `[4]/[6]`
     (genesi non contata) + `base_prefix` stessa numerazione; l'uguaglianza
     self_hash apposto↔simulato impossibile senza violare L4.6 e il test
     encode4 verde;
  3. `l4_check3_all_templates`: `windows(2)` con assert `w[1]`→Snapshot
     sanziona la finestra legale (Snapshotᵢ, Narrateᵢ₊₁) — serve `chunks(2)`;
  4. check8 (sopra): `events()[0]`.
  Correzioni proposte dal figlio: parentesi in `appose_revision`; indici
  `[4]/[6]` + `events()[4]`/`log_len:5`; `chunks(2)`; `[0]`. Con queste,
  l'implementazione attuale porterebbe la suite a verde (dichiarazione del
  figlio, non verificata: nessuna run completa).
- **Report `reports/l4_impl_identita.md` NON scritto per ERRORE STRUMENTO:**
  il layer dei permessi ha negato la scrittura due volte
  (`permission rejected`) — il permessio di scrittura in `reports/` non è
  concesso al ruolo producer a livello harness, nonostante l'autorizzazione
  del mandato. Nessun workaround. Contenuto integrale del report conservato
  dal figlio nella memoria `mem_37f7833a-cf35-4c0c-b86a-05d5866b82ce`.
- **Ambiguità residua dichiarata:** default «sources per statement = unione
  delle seq citate nel template» da firmare (coerente con L4.1 ma mai detto
  esplicitamente dalla legge).

### Figlio 1 — producer (FASE A, terza ripresa — esito)

- Ruolo/modello/classe: producer, `opencode/glm-5.3-flash-free`, classe
  light, variante: nessuna (default catalogo). Stessa sessione
  `ses_e6c53a29affea4AdUkfEiSmo0M` (terzo mandato su stessa sessione).
- Sessioni Mnemosyne: questa ripresa `ses_61a2d358-7734-4941-aee4-52af0b2fa2ab`
  **chiusa** (job `job_a57a35cd-d8f0-4f2c-b927-5984ee7156d4`); precedenti
  `ses_0d20fc46-…` già chiusa. Proposte memoria in attesa (non mie da
  accettare): `mem_7e21f9e0-bf8c-462b-9f92-05719d2ca64f`, `mem_d6812cd0-85a2-4caf-ab9f-40a98ab17c15` (questa ripresa) + le 5
  precedenti (`mem_8de51b0d…`, `mem_1fc3a41f…`, `mem_1eca95a3…`,
  `mem_df416075…`, `mem_37f7833a…`).
- **Esito: TERZO BLOCCO, conforme al mandato** (punto 4: altre incoerenze
  oltre i 6 → fermata e resoconto).
- **Commit 2: NON effettuato.** HEAD invariata
  `302e7e2b267829347b57268dda9a229df67d5c95` (verificata).
- **Progresso effettivo:** tutte le **6 correzioni autorizzate applicate**
  nel working tree; `src/identity.rs` completo e dichiarato
  legge-conforme (nessuna strumentazione residua); riga
  `pub mod identity;` in `src/lib.rs`; scratch cancellato. Suite
  **debug: 142/144 verdi** (122 L0–L3 tutti verdi + L4 20/22). Release
  non eseguito (gate non dichiarabile con 2 rossi).
- **Le 2 ULTERIORI incoerenze semantiche trovate (NON nei 6, fermata
  obbligatoria), con fix minimi proposti NON applicati:**
  1. `l4_check2_ii_…` (~riga 230, closure `mk`): payload `nodes` senza
     parentesi quadre (`{"key":…}` non è stringa F1) → `view::build`
     rifiuta → Schema per costruzione. Fix: `[{…}]`.
  2. `l4_check2_i_…` (~riga 199): secondo `appose_revision` con `at=2`
     ma l'evento generatore dell'arco è a seq 4 → `at=2` punta al
     narrate stesso → autoreferenzialità → Schema (L4.7). Fix: `at=4`.
  Con questi 2 fix minimi il figlio dichiara la suite raggiungibile
  144/144 in debug e release (dichiarazione del figlio, non verificata:
  run release non eseguita).
- **Report producer:** contenuto integrale **CONSEGNATO NEL TESTO FINALE**
  della risposta del figlio (provvedimento opzione 2; scrittura in
  `reports/` negata dall'harness, come previsto). Da trascrivere.
- **check8: test NON in difetto per l'implementazione** — L4.5 zero-PRNG
  rispettata e verificata dal test ora verde; la correzione (vi) era
  necessaria perché la chiave `state` vive solo nel payload di genesi.
- Ambiguità «sources»: **CHIUSA** dalla precisazione emendata nella legge
  (righe 171–177, committata in `302e7e2`) e implementata.

### Figlio 1 — producer (FASE A, quarta ripresa — esito)

- Ruolo/modello/classe: producer, `opencode/glm-5.3-flash-free`, classe
  light, variante: nessuna (default catalogo). Stessa sessione
  `ses_e6c53a29affea4AdUkfEiSmo0M` (quarto mandato su stessa sessione).
- Mnemosyne di questa ripresa: `ses_6d99e3e8-bc15-41ab-b936-ee06ca9d3bbd`
  **chiusa** (job `job_0968419a-d10f-4de7-b67c-297c1904fa40`); proposte:
  `mem_e096d17e-6849-4b97-a7eb-26d10cc6598f`, `mem_12863c3c-2c70-4c0f-a863-03d3d85e357c`.
- **Esito: COMMIT 2 COMPLETATO — 144/144 debug E 144/144 release** (16
  harness tutti verdi; 122 L0–L3 intatti + 22 L4). **Nessuna nuova
  incoerenza.**
- **Commit 2: `280412f9ee77a19c05859d3c6d89fa8102013725`** sopra
  `302e7e2` (HEAD verificata per lettura). Contiene esattamente:
  `src/identity.rs` (nuovo, 634 righe), 1 riga `pub mod identity;` in
  `src/lib.rs`, le 8 correzioni test in `tests/l4_identity.rs`. Esclusi
  dal commit (residuo non committato atteso): `notes/*`, `reports/**`,
  `.DS_Store`. Nessun push, nessun altro commit.
- Le 8 correzioni: 6 della ripresa 3 (d[2], `.clone()`, sources F1,
  indici check4_replay, chunks(2), events()[0]) + 8ª (check2_ii parentesi
  F1) + 9ª (check2_i at=4, testo italiano esatto, sources &[4], assert
  aggiornato) — tutte applicate, verdi, citate nel messaggio del commit.
- Report producer: **consegna integrale nel testo finale** (divieto
  harness `producer.md:10-12`, come previsto): da trascrivere.
- check8: test verde, L4.5 zero-PRNG verificata. Ambiguità sources: chiusa
  dalla legge emendata. Nessuna interpretazione decisa in autonomia oltre
  le correzioni autorizzate.

### Figlio 2 — tester (FASE B)

- Ruolo/modello/classe: tester, `opencode/deepseek-v4.1-flash-free`, classe light, variante: nessuna (default catalogo).
- Session ID: nessuna.
- **Esito: NON LANCELLO** — precondizione (commit 2 + suite verde) assente.
- Report: `reports/gate_l4_impl.md` non prodotto.

- Report: `reports/gate_l4_impl.md` — **PRODOTTO DAL TESTER** (sha256
  `85725f021d72676f6ffea90ee6ad22c62475bb43e9d01a82045bd9adb50eaf42`).
- **Esito: VERDE.** Conteggi: debug **144/144**, release **144/144**; 22
  test l4 tutti ok. Perimetro `302e7e2..HEAD` integro: solo
  `src/identity.rs` (634+, nuovo), 1 riga `pub mod identity;` in
  `src/lib.rs`, `tests/l4_identity.rs` (38 righe = 8 correzioni
  autorizzate in 7 hunk); esclusi confermati. Test-first verificato:
  `2ec39a0..c8ae8de` solo file test, rosso E0432/E0433 ricostruito.
  `git rev-parse HEAD` = `280412f9ee77a19c05859d3c6d89fa8102013725`.
  sha256 legge L4: `cd7e9ce6b3d1c9355c8fa0eb59b8c7002cc461dff738ea3afdb671e8cbf0d9a5`;
  sha256 policy routing v1: `bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177`.
  Checklist 1–8 dichiarata coperta punto per punto dai test l4.
  Sessione Mnemosyne tester: `ses_84ee8331-0315-4074-91c5-d20faf3ed59e`
  (chiusa); nessuna memoria proposta.

### Figlio 3 — reviewer (FASE C — esito)

- Ruolo/modello/classe: reviewer, `opencode/glm-5.3-flash-free`, classe
  light, variante: nessuna (default catalogo), canale free «Free-first».
- Sessione: subagent `ses_e6c1b5760ffeHmc4Wk1DPWOvS8`; Mnemosyne
  dichiarata `ses_66dea74a-9133-48b8-9bd0-687bf227a8c1` (nessuna memoria
  approvata L4 in conflitto; proposte pending coerenti).
- **Esito: CONFORME** — audit sulla stessa revisione integrata
  `280412f9ee77a19c05859d3c6d89fa8102013725` (gate VERDE preesistente).
- **Punto per punto L4.1–L4.10: TUTTI CONFORME** (citazioni file:riga nel
  report). Checklist uscita 1–8: tutta coperta dai 22 test l4.
- **Controllo letterale template L4.3 (byte per byte): TUTTI E 7 I
  TEMPLATE IDENTICI alla legge** (nodo, arco, select, summary, promote,
  archive, recall), più id L1 hex64, ordine lessicografico interno,
  congiunzione `\n`, bibliografia `sources` conforme alla precisazione.
- **8 correzioni test**: tutte e 8 applicate, nessuna differenza oltre
  loro (verificato per esclusione, con righe).
- **Rilievi: 1** (minore, NON bloccante, entro il max 5):
  `src/identity.rs:501–504` vs `:506–508` e `:562–566` — eventi non-ACTION
  portanti `l2`/`l3` scavalcano `validate_decision_fields` in `replay`
  mentre `evaluate_self` li tratta senza discriminare il tipo →
  `build_self` Ok vs `evaluate_self` Schema su log patologico identico;
  da decidere in governance. Aree chiuse (E1/E3/preimage/provenienza/
  L2.27/R1/R2) NON riaperte.
- Limiti dichiarati: `git status`/`git diff` negati alla sandbox del
  reviewer (perimetro già certificato dal gate, re-verificato via
  `git log`/`git show --stat`); nessun cargo eseguito (vincolo di ruolo).
- **Report: `reports/audit_l4_impl.md` — PRODOTTO DAL REVIEWER**
  (scrittura riuscita).

## Riepilogo percorsi (aggiornato, quarta ripresa)

- Report A: `reports/l4_impl_identita.md` — **MANCA nel repo** (permesso
  harness negato al producer): contenuto integrale consegnato testualmente
  dal producer nel suo resoconto (riportato integralmente al mandato 3 e
  4 dall'orchestratore, per trascrizione).
- Report B: `reports/gate_l4_impl.md` — **PRODOTTO** (tester, VERDE).
- Report C: `reports/audit_l4_impl.md` — **PRODOTTO** (reviewer, CONFORME).
- Registro: `reports/routing_l4_impl.md` (questo; nessun record sovrascritto).
- Deliverable git: commit `c8ae8de` (test, 22 rossi) + commit
  `280412f` (implementazione + 8 correzioni test). NESSUN push.

## Stato finale del ciclo

Fase A completata al quarto tentativo (commit `280412f`, 144/144 debug e
release); Fase B gate VERDE (tester deepseek-v4.1-flash-free); Fase C
audit CONFORME con 1 rilievo minore non bloccante (reviewer
glm-5.3-flash-free). Nessun modello sostituito, nessun fallback, nessun
canale fuori Free-first, nessun push. Resta a te (orchestratore):
1. accettazione umana di fine ciclo (la firma non è dei figli né mia);
2. trascrizione del report producer in `reports/l4_impl_identita.md`
   (contenuto integrale nei resoconti testuali del producer);
3. decisione di governance sul rilievo minore dell'audit (1 su 5 ammesse);
4. 11 proposte memoria in attesa (5 + 2 + 2 + 2 dalle quattro riprese).

## Riepilogo percorsi

- Report A: `reports/l4_impl_identita.md` — **MANCA** (figlio bloccato prima del commit 2)
- Report B: `reports/gate_l4_impl.md` — **NON PRODOTTO** (fase non lanciata)
- Report C: `reports/audit_l4_impl.md` — **NON PRODOTTO** (fase non lanciata)
- Registro: `reports/routing_l4_impl.md` (questo, nuovo; nessun record sovrascritto)
- Deliverable git: commit `c8ae8de` (test, 22 rossi) — UNO DEI DUE. Nessun push.
- Residui working tree NON committati: `src/identity.rs` (draft con move-bug noto), `src/lib.rs` (riga mod), fix `.clone()` nei test committati, `examples/l4scratch.rs` (scratch).

## Piano di ripresa (mandato 2 — registrato prima del nuovo dispatch)

Decisioni ricevute dall'orchestratore (31/10/2026, canale invariato
«Free-first»), che sostituiscono le decisioni residue sotto solo dove
indicate:
1. Scratch `examples/l4scratch.rs` → DA CANCELLARE (fuori perimetro).
2. Anomalia (1) **risolta dall'orchestratore**: in
   `l4_check3_select_list_rendering_sorted_dedup_and_empty`
   (`tests/l4_identity.rs:325-339`) `match &d[0]` → `match &d[2]` (prima
   dichiarazione node_a_then_b a seq 1 è rilevante e genera la revisione
   prima; testo e `sources [1,2,3]` validi). **Correzione AUTORIZZATA nel
   commit 2, sola questa presa.**
3. Fix `.clone()` meccanico a `tests/l4_identity.rs:676` → **AUTORIZZATO nel
   commit 2, solo quello**. Qualsiasi ALTRO test incoerente con la legge →
   fermata e resoconto, nessuna correzione in autonomia.
4. **Commit 2 = un solo commit**: `src/identity.rs` completo + riga
   `pub mod identity;` in `src/lib.rs` + i 2 fix test autorizzati. Messaggio
   con citazioni («scegli tu» legge 2ec39a0, «Free-first» canale, 31/10/2026)
   e summary dei 2 fix. NESSUN push.
5. Indicazioni tecniche del digest (catena append simulata con
   `crate::event::compute_hash` pubblico da `src/log.rs:27`; self_hash =
   preimage con `head_hash` = hash narrate simulato; statement PER REVISIONE;
   build_self su narrate REALI; L4.3 via `view::build`; sources dedup
   crescente; cutoff L4.6; prefix-base L2.22; L4.5 zero; L4.9 non toccata).
6. `l4_check8`: se il TEST contraddice la legge → fermata e resoconto; se
   coerente → va verde col codice.
7. Dopo commit: `cargo test` + `cargo test --release` TUTTI VERDI (122 + 22
   l4); **report obbligatorio** `reports/l4_impl_identita.md` (deliverable
   certificato dal mandato).

Le fasi B e C restano come da piano, con i modelli free già autorizzati
(`deepseek-v4.1-flash-free` gate, `glm-5.3-flash-free` audit).

## Piano di terza ripresa (mandato 3 — registrato prima del dispatch)

**Precondizioni verificate dal Router (lettura diretta, shell negata):**
- Policy `.opencode/routing.md` v1: **ATTIVO** (verificato).
- HEAD: `302e7e2b267829347b57268dda9a229df67d5c95` (letta da
  `.git/refs/heads/main`) — commit dell'orchestratore sopra `c8ae8de`:
  *«provvedimento divieto-producer (diagnosi corretta) + precisazione
  L4.6 sources»* (da reflog).
- **Precisazione L4.6 sources LOCALIZZATA e letta**: `notes/L4_legge_identita.md`
  righe 171–177 — «sources della revisione = l'unione, dedup, in ordine
  crescente delle seq CITATE nei template delle affermazioni della
  revisione: la seq dell'evento che genera ciascuna affermazione più le
  seq originarie che compaiono nei rimpiazzi `{ups}`, `{downs}`,
  `{expulsions}`, `{origin}`…», firmata dall'orchestratore 31/10/2026
  sotto delega. La legge è dunque la versione emendata dentro `302e7e2`;
  il producer la rileggerà per intero.

**Decisioni dell'orchestratore (mandato 3), da trasmettere letteralmente:**
1. SEI correzioni test AUTORIZZATE nel commit 2 (elenco completo al
   dispatch; ognuna citata nel messaggio del commit).
2. Il producer NON scrive in `reports/` (permesso harness reale negato):
   CONSEGNA il report integrale **nel testo finale** (opzione 2 del
   provvedimento); nessun tentativo di riscrittura in `reports/`.
3. `src/identity.rs` working tree già dichiarato legge-conforme: revisione
   + commit.
4. Nessuna nuova decisione attesa: altro test incoerente oltre i 6 →
   fermata e resoconto.
5. COMMIT 2 = un solo commit sopra `302e7e2`: `src/identity.rs` + riga
   `pub mod identity;` + 6 correzioni test; messaggio con autorizzazioni
   umane citate. NESSUN push.
6. Verifica finale: `cargo test` + `cargo test --release` → 144/144
   attesi in entrambe (122 pre-esistenti + 22 l4), contegge esatta.
7. Resoconto testuale finale: 22 nomi test l4, hash commit 1 e 2, modello
   completo + variante, session ID, aspetti interpretativi, stato finale
   working tree.

**Fasi successive (seriali, foreground, solo free):**
- FASE B: tester `opencode/deepseek-v4.1-flash-free` (light, variante
  default catalogo) — run complete debug+release, perimetro
  `302e7e2..HEAD` (solo `src/identity.rs`, 1 riga `src/lib.rs`,
  `tests/l4_identity.rs`), `git rev-parse HEAD`, sha256 legge+policy,
  verdetto VERDE/ROSSO. Report `reports/gate_l4_impl.md` (stessa regola
  consegna testo se permesso negato).
- FASE C: reviewer `opencode/glm-5.3-flash-free` (light, variante default
  catalogo) — audit L4.1–L4.10 + checklist 1–8, controllo letterale
  template, max 5 rilievi, aree chiuse non riaperte. Report
  `reports/audit_l4_impl.md` (stessa regola consegna).
- Serialità, foreground, nessun fallback, nessun canale fuori Free-first;
  blocco reale → fermata e resoconto con session ID.

## Piano di quarta ripresa (mandato 4 — registrato prima del dispatch)

**Precondizioni verificate (lettura diretta, shell negata):** policy v1
ATTIVO; HEAD `302e7e2b267829347b57268dda9a229df67d5c95`. Canale
invariato «Free-first».

**Decisioni dell'orchestratore (mandato 4), da trasmettere letteralmente:**
1. **8ª correzione test AUTORIZZATA** — `l4_check2_ii` (closure `mk`,
   ~riga 228-231): payload `nodes` → array F1 con parentesi quadre:
   `Value::Str(format!("[{{\"key\":\"{key}\",\"kind\":\"letter\"}}]"))`.
2. **9ª correzione test AUTORIZZATA** — `l4_check2_i`, fix COERENTE a 4
   punti: (a) riga 199 `at` 2→4; (b) riga 200 testo → `a seq 4 ho
   dichiarato l'arco knows da letter "A" a letter "B"` (template L4.3,
   {seq} = seq evento generatore); (c) riga 201 `sources` `&[2]`→`&[4]`
   (precisazione 302e7e2); (d) riga 216 assert con lo stesso testo
   italiano esatto.
3. Commit 2 = un solo commit sopra `302e7e2`: `src/identity.rs` + riga
   `pub mod identity;` + le 8 correzioni test; esclusi dal commit
   `notes/mnemosyne_backlog.md`, `notes/L3_domande_memoria.md`,
   `reports/**`, `.DS_Store`; messaggio con autorizzazioni umane, elenco
   8 correzioni con motivazione, 22 rossi convertiti. NESSUN push.
4. Verifica finale: `cargo test` + `cargo test --release` → 144/144
   attesi in entrambe; se release non torna 144/144 → fermata, niente
   gate.
5. Report producer: consegna INTEGRALE nel testo finale (divieto harness
   `producer.md:10-12`); il Router lo riporta tale e quale.
6. Dopo commit 2 + 144/144: FASE B tester
   `opencode/deepseek-v4.1-flash-free` (light, variante default) →
   report `reports/gate_l4_impl.md` (perimetro `302e7e2..HEAD`, checklist
   punto per punto, rev-parse, sha256 legge+policy, VERDE/ROSSO).
7. Poi FASE C reviewer `opencode/glm-5.3-flash-free` (light, variante
   default) → report `reports/audit_l4_impl.md` (L4.1–L4.10 + checklist
   1–8, controllo letterale template, max 5 rilievi, aree chiuse non
   riaperte, 8 correzioni elencate come uniche differenze ammesse).

## Risultati — esiti

Stato dopo la terza ripresa (mandato 3): terzo blocco del producer
(fermata conforme alle sue istruzioni), nessun modello sostituito, nessun
fallback, nessun passaggio di canale, nessuna fase B/C lanciata, nessun
push; HEAD invariata `302e7e2`. Le 6 correzioni autorizzate sono TUTTE
APPLICATE (working tree), la legge è emendata (`302e7e2`), la suite debug
è a 142/144. Restano, per te:

1. **Due rettifiche test da autorizzare** (il figlio si è fermato per
   vincolo, le ha proposte ma non applicate):
   - `l4_check2_ii_…` (~riga 230, closure `mk`): chiudere il payload
     `nodes` con parentesi quadre (`Value::Str(format!("[{{\"key\":\"{key}\",\"kind\":\"letter\"}}]"))`)
     — oggi non è una stringa F1 e `view::build` (L1, intoccabile) la
     rifiuta di diritto → Schema per costruzione;
   - `l4_check2_i_…` (~riga 199): secondo `appose_revision` `at=2` →
     `at=4` (l'evento generatore dell'arco è a seq 4; `at=2` punta al
     narrate stesso → autoreferenzialità vietata da L4.7 → Schema).
   Con questi 2 fix il figlio dichiara la suite raggiungibile 144/144
   debug e release. Poi: commit 2 (contenuto e messaggio come da mandato,
   con tutte le 8 correzioni citate) → Fase B → Fase C.
2. **5+2 proposte memoria** del figlio in attesa (elenco sopra, sezione
   figlio 1): non mie da accettare.
3. Il report producer è consegnato testualmente nel resoconto del mandato
   3: va trascritto dall'orchestratore (permesso harness negato al
   producer).

## Limitazioni

- Shell negata al Router: verifiche HEAD/policy via lettura file; hash
  richiesti al tester (Fase B). Nessuna sandbox aggirata.
- Quote free residue: sconosciute; costo catalogo 0. Nessuna spesa extra.
