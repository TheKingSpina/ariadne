# Registro Router — fix L2 R1/R2 + R4 (test-first) da `2a9e18e`

## Identificazione

- **Mandato:** fix dei rilievi R1, R2 e R4 del review compatto precedente con
  percorso test-first (due commit su main), gate tester e re-review; emesso
  dall'orchestratore (sessione primaria) al Router.
- **Parent/mandante:** sessione primaria orchestratore (GLM-5.3-Flash free).
- **Policy di routing:** `.opencode/routing.md` v1 (ATTIVO, 31/10/2026),
  verificato prima dei dispatch per lettura. Hash SHA-256 policy: non
  calcolabile dal Router (shell negata dal runtime): dichiarato non
  disponibile, da raccogliere con `shasum` dal figlio tester (mandato di Fase B).
- **Revisione Git:** HEAD dichiarato `2a9e18e`; verificato via lettura
  `.git/refs/heads/main` → `2a9e18ed4ada8aa64ab1790b56f0e5b5078ca7a8`.
- **Contesto:** legge L3 con emendamento E3 (vinces L3.8; nessun codice da
  cambiare per E3); `opencode.jsonc` con `experimental.subagent_depth: 2`.
  Rilievi da chiudere (da `reports/compact_review_l0_l3_b2770bd.md`):
  R1 grave `src/selection.rs:41`, R2 moderato `src/selection.rs:25`,
  R4 minore warning `src/selection.rs:3`. R3 (nota legislativa) e aree già
  chiuse (E1, preimage, archiviazione futura, provenienza) fuori scope.
- **Autorizzazione umana (firma 31/10/2026, «percorso consueto»):** producer
  in classe `medium` con `openai/gpt-6.1-terra`; tester e reviewer in classe
  `light` con modelli free a scelta del Router. Nessun altro canale/spesa.

## Verifiche catalogo (prima dei dispatch)

- `openai/gpt-6.1-terra`: presente, status active, varianti
  none/low/medium/high/xhigh/max — nessuna variante prescritta dall'umano →
  si usa e si dichiara il default del catalogo (nessuna variante inventata).
  Listino vuoto nel catalogo: la gratuità non è provata; il canale è coperto
  dall'autorizzazione di abbonamento nel mandato.
- Modelli free già verificati nel catalogo 31/10/2026 (costo 0/0, active):
  `opencode/deepseek-v4.1-flash-free`, `opencode/glm-5.3-flash-free`.

## Piano di dispatch (registrato prima delle chiamate)

| # | Fase | Ruolo | Classe | Modello richiesto | Variante | Motivazione |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | A — fix test-first (2 commit) | producer | medium | `openai/gpt-6.1-terra` | default del catalogo (nessuna prescritta) | Autorizzazione umana esplicita del mandato (pin storico del ruolo); lavoro intermedio con rischio su codice L2 e due commit |
| 2 | B — gate test | tester | light | `opencode/deepseek-v4.1-flash-free` | default del catalogo (nessuna prescritta) | Task procedurale (due run, conteggi, check scope, hash); stesso free già usato con esito VERDE nel gate precedente |
| 3 | C — re-review | reviewer | light | `opencode/glm-5.3-flash-free` | default del catalogo (nessuna prescritta) | Giudizio di chiusura rilievi su legge/codice; stesso free del review precedente, già validato sul progetto |

- **Serialità:** un figlio alla volta, foreground, un solo tentativo per
  dispatch; nessun fallback autonomo; se Terra non carica/risponde → stop e
  segnalazione all'orchestratore senza sostituzioni.
- **Dipendenze:** B solo dopo esito A; C solo dopo gate B VERDE.
- **Scritture ammesse:** solo i due commit del producer (test-first, con
  citazione dell'autorizzazione nel messaggio del primo commit) e i report in
  `reports/`. Nessun push; nessuna modifica di policy, agenti o config.
- **Owner integrazione:** il producer è proprietario dei due commit; nessuna
  fase di stitching separata.

## Piano di ripresa (mandato 2 — registrato prima dei nuovi dispatch)

Aggiornamento legislativo/organizzativo ricevuto dall'orchestratore con firma
umana (31/10/2026), verificato dal Router prima del dispatch:
- **L2.27 in vigore** in `notes/L2_legge_selezione.md` (commit `51e69f6` =
  HEAD attuale `51e69f6ecf0006df9bae8e2444c22d4127c505fb`, letto integralmente
  dal Router): **Lettura A** — ogni evento L0 che dichiara un arco (originale
  o riprova reiterata) attiva gli endpoint di quell'evento, una sola
  attivazione per elemento per seq (dedup intra-evento, L2.13); «seq di
  dichiarazione minimo» in L2.5 è identità/origin L1 D18, non attivazione;
  «senza doppio ricalcolo per riprova» = dedup nello stesso evento. Coerente
  con L3.3 e con `src/memory.rs` accettato. Il blocco precedente è risolto
  per via legislativa.
- **`BLOCKER.md` rimosso** dall'orchestratore (verificato dal Router: assente
  alla root). Non va ricreato; il contenuto è già documentato in questo
  registro.
- **Comunicazione di ruolo al producer:** era errore del figlio rifiutare il
  report; il producer DEVE produrre `reports/fix_l2_r1_r2_2a9e18e.md`
  (esplicitamente concesso).

| # | Fase | Ruolo | Classe | Modello richiesto | Variante | Stato/azione |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | A2 — proseguimento sessione producer (stessa sessione `ses_e6c8386afffe2JOt1mFaltgPOY`) | producer | medium | `openai/gpt-6.1-terra` | default del catalogo | test multi-arco coerente a Lettura A; test di riprova reiterata (fissaggio, nel commit 2); commit 2 implementazione R1/R2/R4; report obbligatorio; no push |
| 2 | B — gate test | tester | light | `opencode/deepseek-v4.1-flash-free` | default del catalogo | dopo commit 2: debug + release, verifica esplicita dei 3 test, `git diff --stat 51e69f6..HEAD` perimetro, sha256 policy, `rev-parse HEAD` |
| 3 | C — re-review | reviewer | light | `opencode/glm-5.3-flash-free` | default del catalogo | dopo gate VERDE: chiusura R1/R2/R4 con file e riga, applicazione effettiva L2.27 (non solo citata), nessuna area già chiusa riaperta, coerenza L3 (`src/memory.rs` invariato); max 3 rilievi |

Serialità invariata: un figlio alla volta, foreground, nessun fallback; blocco
di un figlio → stop e segnalazione con session ID.

## Risultati

### Figlio 1 — producer (FASE A) — esito finale dopo ripresa

Modello/sessione invariati su tutta la Fase A (ripristino autorizzato con
L2.27): `openai/gpt-6.1-terra`, classe medium, variante nessuna (default);
sessione strumento `ses_e6c8386afffe2JOt1mFaltgPOY` (stessa sessione, ripresa
del mandato 2), sessioni Mnemosyne dichiarate `ses_4a6a1ed0-…` (mandato 1) e
`ses_73189cec-c731-44f3-8aca-638ac1dfefff` (mandato 2).

- **Commit 1 (test, mandato 1):** `6cdf7b49c2fe54b6a078d797d12a8df602815ab5`
  — test rossi dichiarati: suite L2 13 passati / 2 falliti
  (`distinct_edge_events_activate_only_their_own_endpoints`,
  `in_window_select_without_config_is_error`).
- **Commit 2 (implementazione, mandato 2, post-L2.27):**
  `610b43ce57a13234c9e54b7d58967d9d0ff45577` — HEAD verificata per lettura
  `.git/refs/heads/main` = questo hash. Contiene: R1 (attivazioni solo da
  archi dell'evento, endpoint non mappabili → `L2Error::Schema`), R2 (nessun
  `tau` default → `L2Error::Config`), R4 (`use EventType` rimosso), test
  multi-arco (se rettificato) e test di riprova reiterata dichiarato dal
  figlio come **non rosso / fissaggio L2.27** (verde anche sul vecchio codice).
- **Dopo commit 2 (dichiarati dal figlio):** `cargo test` 122/122 passati,
  0 falliti; `cargo test --release` 122/122 passati, 0 falliti (baseline 119
  + 3 nuovi). Verifica indipendente demandata al gate Fase B.
- **Scostamento ripetuto (NON blocco operativo):** il figlio ha DI NUOVO
  rifiutato di produrre `reports/fix_l2_r1_r2_2a9e18e.md`, invocando il
  «divieto del ruolo» benché il mandato (ripetuto esplicitamente nella
  ripresa) lo concedesse e lo imponesse. La sostanza documentale è nella
  risposta del figlio e in questo registro; nessun terzo tentativo è stato
  fatto (nessun retry automatico). Decisione su come recuperare il deliverable
  → all'orchestratore.
- Dichiarazioni del figlio su ambiguità: nessuna residua dopo L2.27.

### Figlio 2 — tester (FASE B)

- Modello: `opencode/deepseek-v4.1-flash-free`, classe light, variante: nessuna (default catalogo).
- **Session ID strumento (subagent):** `ses_e6c78f5ecffeG2UkKQh104Ih4W`;
  session ID dichiarata dal figlio: `ses_34f1cbab-7a2d-4b10-aa89-55676ef84557`.
  Modello effettivo: non verificabile dal Router — attestazione del figlio.
- **Esito: GATE VERDE.** `cargo test` 122/122 e `cargo test --release`
  122/122 (0 falliti; l2_selection 16, l3_memory 14). Tre test verdi in
  entrambi i run: `distinct_edge_events_activate_only_their_own_endpoints`
  (`tests/l2_selection.rs:166`), `in_window_select_without_config_is_error`
  (`:178`), `repeated_edge_reactivates_endpoints_on_each_event`
  (`:186`, test di fissaggio L2.27, dichiarato non-rosso). Perimetro:
  `git diff --stat 51e69f6..HEAD` = solo `src/selection.rs` +32/−3 e
  `tests/l2_selection.rs` +13; `2a9e18e..HEAD` = + `notes/L2_legge_selezione.md`
  (la legge dell'umano); `src/memory.rs`, API e policy invariate.
  HEAD confermata `610b43ce…` (rev-parse negato dalla sandbox del figlio;
  prova da `.git/refs/heads/main`). Comando di test-first ricostruito
  staticamente (limitazione dichiarata).
- **Hash SHA-256 raccolti in Fase B (richiesti dal mandato):**
  - `.opencode/routing.md` (policy v1): `bf298cb7428c1b7926e81dcf9fb56f31d3e2e873ac98bd17d14a149f9c46a177`
  - `notes/L2_legge_selezione.md` (L2.27): `99b31942c94402c4ab4ccd28a908ea87b0ff9132d1df81a8a9c4983012767f7d`
  - `src/selection.rs` (post-fix): `62130ad4d2a733296117750a6e9598fea6ad996041a2b1c1181fabfca5100cb3`
  - `tests/l2_selection.rs` (post-fix): `c6e7b81ee708f36e6b9e0203f19fd2a8efc44cf0decbe2432de82db22a78e011`
- **Scostamento residuo al gate:** `git status --porcelain` = ` D BLOCKER.md`
  (cancellato dal worktree, ancora tracciato in HEAD: cancellazione non
  committata) più i soliti non tracciati in `notes/`/`reports/`. Non tocca il
  codice sotto gate; resta da sanare da chi ne ha titolo.
- Report: `reports/gate_fix_l2_r1_r2.md` (letto e verificato dal Router).

### Figlio 3 — reviewer (FASE C)

- Modello: `opencode/glm-5.3-flash-free`, classe light, variante: nessuna (default catalogo).
- **Precondizione gate:** soddisfatta (gate VERDE sulla stessa revisione `610b43c`).
- **Session ID strumento (subagent):** `ses_e6c763caeffeDWcAd14Pcn2NYi`
  (la sessione ha chiuso senza risposta testuale: il deliverable è nel
  report, che è stato trovato e letto); sessione Mnemosyne dichiarata
  `ses_803e2cc6-da24-426c-b0eb-6dfd42ca98b4` (aperta e chiusa);
  session OpenCode dichiarata dal figlio non esposta dai suoi strumenti.
  Modello effettivo: non verificabile dal Router — attestazione del figlio.
- **Esito: CONFORME.** R1 **CHIUSO** (ciclo su `view.edges` sparito;
  `edge_mentions` per-evento `src/selection.rs:21-44,67`; Schema per non
  mappabili `:35-39`; dedup intra-evento `:68`; test multi-arco che
  discrimina). Punto 2 **L2.27 effettivamente applicata** (estrazione
  per-evento dal payload, non caso residuo; coerenza identità D18 vs
  attivazione per-evento). R2 **CHIUSO** (`config.ok_or(L2Error::Config)?`
  `src/selection.rs:51,69`; test `:178`). R4 **CHIUSO** (`EventType` rimosso
  da `:3`, zero occorrenze). `src/memory.rs` invariato (`git diff
  2a9e18e..HEAD` vuoto), API L2.20 invariate, aree già chiuse (E1, preimage,
  archiviazione futura, provenienza) non toccate e non riaperte. Ordine dei
  commit conforme; test di riprova nel commit 2 legalmente coperto da
  L2.27:235-238. **2 rilievi minori** (max 3 rispettato, nessuno bloccante):
  (1) rosso iniziale ricostruito staticamente, non rieseguito a runtime;
  (2) ` D BLOCKER.md` working tree da sanare.
- Report: `reports/re_review_fix_l2_r1_r2.md` (letto e verificato dal Router).

## Stato complessivo e decisioni residue

Ciclo A → B → C **completato** dopo la ripresa legislativa (L2.27, Lettura A,
commit `51e69f6`, letto dal Router prima dei dispatch):
- **A:** due commit — `6cdf7b4` (test rossi, autorizzazione citata) e
  `610b43c` (implementazione minima R1/R2/R4). Nessun push, perimetro rispettato.
- **B:** gate **VERDE** (122/122 debug e release, tre test verdi, perimetro e hash).
- **C:** re-review **CONFORME** (R1/R2/R4 chiusi, L2.27 effettivamente
  applicata, `src/memory.rs` invariato, aree chiuse non riaperte).

Nessun modello sostituito, nessun fallback, nessuna spesa fuori classe
(producer in abbonamento come firmato; tester/reviewer free).

**Decisioni residue per l'orchestratore/umano (non mie):**
1. **Deliverable mancante:** `reports/fix_l2_r1_r2_2a9e18e.md` nonostante
   doppia richiesta esplicita (il producer lo rifiuta per «divieto di ruolo»).
   Nessun terzo tentativo fatto (nessun retry automatico): serve tuo
   provvedimento (dispaccio dedicato con chiarimento, oppure altra via).
2. **Sanatoria ` D BLOCKER.md`:** il file è cancellato dal worktree ma risulta
   ancora tracciato in HEAD (cancellazione non committata): va committata la
   rimozione o ripristinato, da chi ne ha titolo — mai amend/rebase.
3. **Memoria pendente** segnalata dal reviewer (`mem_616ad586-…`, episode del
   ciclo compatto): in attesa di tua decisione, non tipizzata dal Router.
4. Rilievi minori di C (rosso iniziale ricostruito staticamente, non
   rieseguito): opzionale eseguirlo a runtime su worktree scratch, su nuovo
   mandato (il tester l'ha già descritto nel suo report).

## Percorsi dei report

1. `reports/fix_l2_r1_r2_2a9e18e.md` (producer) — **MANCA** (rifiuto ripetuto del figlio; vedi decisione 1)
2. `reports/gate_fix_l2_r1_r2.md` (tester) — **PRODOTTO**
3. `reports/re_review_fix_l2_r1_r2.md` (reviewer) — **PRODOTTO**
4. `reports/routing_fix_l2_r1_r2.md` (questo registro) — prodotto
- Artefatti git prodotti: commit `6cdf7b4` e `610b43c` (entrambi autorizzati, nessun push).
- Residuo fuori `reports/`: `BLOCKER.md` tracciato in HEAD e cancellato dal
  worktree (creato dal producer nel mandato 1, poi rimosso dall'orchestratore
  — vedi decisione 2).

## Limitazioni

- Shell negata al Router (`permission.rejected`): HEAD verificato per
  lettura `.git/refs/heads/main`; hash SHA-256 policy/file raccolti dal
  figlio tester in Fase B (riportati sopra, attestazione). Nessuna sandbox
  aggirata.
- Modello effettivo dei figli: non verificabile dal Router — attestazioni dei
  figli, dichiarate come tali.
- Costi/quote: canale abbonamento OpenAI coperto dalla firma del mandato;
  quota residua sconosciuta. Modelli free: costo catalogo 0, quote sconosciute.
- Il reviewer ha chiuso la sessione senza risposta testuale: il verdetto è
  letto dal suo report (fonte primaria del deliverable).
