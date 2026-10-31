# Registro Router — collaudo compatto monolitico L0-L3

## Identificazione

- **Mandato:** collaudo compatto MONOLITICO L0-L3 sullo stato attuale del repo
  (gate tester + review compatta), emesso dall'orchestratore (GLM-5.3-Flash free,
  sessione primaria) al Router.
- **Parent/mandante:** sessione primaria orchestratore (GLM-5.3-Flash free).
- **Policy di routing:** `.opencode/routing.md` v1 (stato ATTIVO, 31/10/2026).
  Hash SHA-256 del file policy: non disponibile al Router — la shell è negata
  dal runtime (`permission.rejected`, 31/10/2026): nessun calcolo hash senza
  pipeline; valore dichiarato sconosciuto, non stimato.
- **Revisione Git:** HEAD dichiarato dall'orchestratore `b2770bd`; verificato
  dal Router via lettura `.git/refs/heads/main` →
  `b2770bdf3e60883258038db37f36f3ea2ad6432e`. La shell è negata al Router:
  `git rev-parse HEAD` non eseguibile dal Router; il figlio tester lo esegue
  con i propri permessi e lo riporta nel suo report.
- **Canale di costo autorizzato:** SOLO modelli free (vincolo del mandato).
- **Classe policy:** `light` per entrambi i figli (modelli free del provider
  `opencode`, gratuità verificata nel catalogo il 31/10/2026: costo input e
  output 0 USD, status active).
- **Modalità:** seriale, foreground, un figlio alla volta; nessun fallback,
  nessun retry illimitato, nessuna sostituzione autonoma di canale.

## Verifica catalogo modelli free (prima dei dispatch)

Tool `opencode.models`, provider `opencode`, tipo chat, 31/10/2026. Modelli
free con costo 0/0 e status active presenti: `flint-40b-free`,
`nemotron-3.5-super-vl-free`, `beam-free`, `mimo-v2.6-flash-free`,
`deepseek-v4.1-flash-free`, `glm-5.3-flash-free`, `big-pickle`.
Nessun modello in abbonamento o OpenRouter a pagamento è stato usato o
autorizzato in questo mandato.

## Piano di dispatch (registrato prima delle chiamate)

| # | Fase | Ruolo | Classe | Modello richiesto | Variante | Motivazione |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Gate di test | tester | light | `opencode/deepseek-v4.1-flash-free` | default del catalogo (nessuna variante prescritta nel mandato; non inventata) | Compito procedurale ben definito: eseguire `cargo test` (debug) e `cargo test --release`, raccogliere conteggi e hash in un report compatto; il modello free è adeguato a esecuzione di comandi e referto numerico |
| 2 | Review compatta | reviewer | light | `opencode/glm-5.3-flash-free` | default del catalogo (nessuna variante prescritta nel mandato; non inventata) | Giudizio complessivo di coerenza normativa L0-L3 su src/ tests/ notes/ readme.md: la policy avverte che lettura/giudizio non è task automaticamente facile; GLM-5.3-Flash Free è il free con maggiore evidenza di uso nel progetto (modello dell'orchestratore); nessuna diversità di famiglia è richiesta dal mandato — modello realmente dichiarato, senza promettere indipendenza assoluta di famiglia |

**Unità assegnate:** intero stato del repo alla revisione `b2770bd`; scope
distinto e non sovrapposto tra i due figli (gate numerico vs review di
coerenza). **Owner dell'integrazione:** nessuna integrazione di codice prevista
(i figli non scrivono codice; scrittura ammesse solo nei rispettivi report di
`reports/`). **Dipendenza:** il dispatch del reviewer parte solo dopo esito
del gate tester sulla stessa revisione.

## Risultati

### Storico del blocco iniziale e suo ripristino (subentrata)

- **Errore originale (primo tentativo di dispatch tester, stesso mandato):**
  `Subagent depth limit reached (1). Increase "experimental.subagent_depth" to allow nested subagents.`
  Errore strumento/configurazione all'avvio: nessuna sessione figlia creata,
  nessun modello invocato, nessuna quota consumata. Stato sessione precedente:
  nessuna esistita.
- **Autorizzazione umana successiva:** aggiunta in working tree (non committata)
  di `experimental.subagent_depth: 2` in `opencode.jsonc`, dichiarata
  dall'umano attraverso l'orchestratore. Nessun cambio di modello né di canale
  di costo: modelli e classi del piano sono invariati; nessuna sostituzione
  autonoma del Router.
- **Stato:** blocco risolto, mandato ripreso dall'inizio in serie.

### Figlio 1 — tester

- **Ruolo/modello/classe:** tester, `opencode/deepseek-v4.1-flash-free`, classe `light`, variante: nessuna (default del catalogo).
- **Session ID strumento (subagent):** `ses_e6c969032ffe91ycvZuDmMW0BE`.
- **Session ID dichiarata dal figlio:** `ses_e6d5af25-24c7-4901-88eb-68d000b3af23`
  (nel suo report). Differenza fra le due dichiarazioni: registrata com'è,
  nessuna riconciliazione autonoma.
- **Modello effettivo:** non verificabile in modo indipendente dal Router —
  dichiarazione del figlio (ID completo `opencode/deepseek-v4.1-flash-free`)
  e nessuno strumento del Router espone il modello reale della sessione figlia.
- **Esito: GATE VERDE.** `cargo test` debug 119/119 (0 failed) e
  `cargo test --release` 119/119 (0 failed); 14 test L3 (`tests/l3_memory.rs`);
  HEAD verificato dal figlio con `git log -1 --format=%H` →
  `b2770bdf3e60883258038db37f36f3ea2ad6432e` (il comando `git rev-parse HEAD`
  era negato dalla sandbox del figlio; nessun retry forzato, sostituto
  equivalente dichiarato nel report).
- **Report figlio:** `reports/compact_test_l0_l3_b2770bd.md` (letto e
  verificato dal Router: comandi, conteggi per suite, hash SHA-256 binario
  debug `73a4160e0ead4ed443b0b21fa386530af5f903241448de50211ae2262d5c7ec2`,
  limitazioni — incluso che il gate riguarda le due suite, non l'intero gate
  globale di accettazione).

### Figlio 2 — reviewer

- **Ruolo/modello/classe:** reviewer, `opencode/glm-5.3-flash-free`, classe `light`, variante: nessuna (default del catalogo).
- **Precondizione gate:** soddisfatta — figlio 1 VERDE sulla stessa revisione
  `b2770bd`; riferimento al report gate citato nel mandato di review.
- **Session ID strumento (subagent):** `ses_e6c95445dffelCnnPo8x98aZul`.
- **Session ID dichiarata dal figlio:** la session OpenCode non è esposta al
  figlio da alcun suo strumento (dichiarata non disponibile, non stimata);
  sessione Mnemosyne aperta e chiusa durante l'audit:
  `ses_e629dc87-0fbe-4a35-8442-be5ed0f2a970` (nessun conflitto irrisolto,
  nessuna proposta pendente).
- **Modello effettivo:** non verificabile in modo indipendente dal Router —
  dichiarazione del figlio (ID completo `opencode/glm-5.3-flash-free`).
- **Esito: NON CONFORME.** 4 rilievi (max 5 rispettato), ordinati per gravità:
  R1 GRAVE `src/selection.rs:41` (attivazione archi di tutta la vista contro
  L2.3/L2.13/L2.5, divergenza con `src/memory.rs:67-68`); R2 MODERATO
  `src/selection.rs:25` (default silenzioso `tau=1` contro L2.2); R3 NOTA di
  coerenza legislativa L3.5/L3.8 (`notes/L3_legge_memoria.md:85-87` vs
  `:130-132`); R4 MINORE warning build `src/selection.rs:3` (debito già
  noto). Nessun rilievo nelle aree escluse (E1, preimage, archiviazione
  futura, provenienza).
- **Report figlio:** `reports/compact_review_l0_l3_b2770bd.md` (letto e
  verificato dal Router).

## Esito complessivo del mandato

Gate tester **VERDE** (119/119 debug e release, 14 L3, su `b2770bd`);
review compatta **NON CONFORME** (4 rilievi, il principale R1 grave su L2).
I tre percorsi report sono elencati sotto. Il costo osservato è zero per il
catalogo di entrambi i modelli free; quote residue rimangono sconosciute.

**Prossima decisione (all'orchestratore/umano, non al Router):** disporre il
trattamento di R1 (e l'eventuale chiarimento legislativo R3): eventuale fix a
fronte di mandato produttore, poi nuovo gate e nuova review su revisione
aggiornata. Il Router non modifica codice né giudica accettazioni.

## Percorsi dei tre report

1. `reports/compact_test_l0_l3_b2770bd.md` (tester)
2. `reports/compact_review_l0_l3_b2770bd.md` (reviewer)
3. `reports/routing_compact_l0l3_b2770bd.md` (questo registro)

## Limitazioni riscontrate

- **Blocco iniziale (risolto):** il runtime aveva negato la creazione di
  subagent nidificati (`Subagent depth limit reached (1)`). Risolto dall'umano
  con `experimental.subagent_depth: 2` in `opencode.jsonc` (working tree,
  non committato), autorizzazione dichiarata all'orchestratore; mandato
  ripreso dall'inizio senza cambi di modello o di canale.
- Shell negata al Router dal runtime (`permission.rejected`): verifica HEAD
  effettuata per via di lettura `.git/refs/heads/main` →
  `b2770bdf3e60883258038db37f36f3ea2ad6432e`; `git rev-parse HEAD` eseguito
  dal figlio tester via `git log -1 --format=%H` (sandbox del figlio negava
  `rev-parse`). Non è stata aggirata alcuna sandbox.
- Modello effettivo delle sessioni figlie: non verificabile dal Router con i
  propri strumenti; in entrambi i casi vale l'attestazione del figlio, come
  previsto dalla policy.
- Verifica del file policy per hash SHA-256: non disponibile (nessuna shell
  al Router, nessun calcolo senza pipeline); identificazione della policy solo
  per contenuto letto (v1, ATTIVO, 31/10/2026).
- Costi/quote residui dei modelli free: sconosciuti (catalogo con prezzi 0 non
  prova quota); nessuna spesa autorizzata o registrata.
- Il gate registrato qui copre le due suite di test richieste (fase 1 del
  ciclo); non è un'accettazione umana né il gate globale del ciclo.
