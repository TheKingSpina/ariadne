# Re-review fix L2 R1/R2/R4 — revisione 610b43c

- **Revisione Git:** `610b43ce57a13234c9e54b7d58967d9d0ff45577` (= `610b43c`). `git rev-parse HEAD` negato dalla sandbox (permission denied, come già al gate); HEAD confermato con lettura di `.git/refs/heads/main`. `git log --oneline -7`: cima = `610b43c` → `51e69f6` (legge L2.27) → `6cdf7b4` (test) → `2a9e18e` (base E3).
- **modello:** `opencode/glm-5.3-flash-free` (classe `light`, nessuna variante: default del catalogo; assegnato dal Router, policy `.opencode/routing.md` v1 ATTIVA 31/10/2026). Session ID Mnemosyne (`memory_open_session`, progetto `opencode:Ariadne`): `ses_803e2cc6-da24-426c-b0eb-6dfd42ca98b4`, chiusa a fine audit. L'ID della sessione runtime OpenCode non è esposto a quest'agente da alcuno strumento disponibile: dichiarata la mancanza, non stimato l'id.
- **Gate tester (precondizione, stessa revisione 610b43c):** VERDE — `reports/gate_fix_l2_r1_r2.md` (tester `opencode/deepseek-v4.1-flash-free`, ses_34f1cbab): 122/122 debug + 122/122 release, tre test verdi, perimetro ok, sha256 policy/file. Schema SHA-256 policy citato dal gate `bf298cb7…c46a177`: **non ricalcolato da me** (shell limitata a letture Git).
- **Memorie esaminate (Mnemosyne):** via `memory_context`: `mem_8c87fe0c-133d-4715-9681-4dbcae9f6b0b` (legge L3, richiama il conteggio per-evento L2.3/L2.13 — coerente col fix, nessun conflitto), `mem_f1b05e04-85bc-4a91-b506-3a2f7ec8293a` (E1/E2, non toccate da questo fix). `memory_list_conflicts`: nessun conflitto irrisolto. `memory_pending_review`: 1 proposta pendente `mem_616ad586-4307-48c3-9594-ee1c28b3bb52` (episode del ciclo compatto su b2770bd con i rilievi R1–R4 qui sopra; in attesa di decisione umana/orchestratore, non la tipo).
- **Non eseguiti test; solo letture (git diff/show/log negati solo per `rev-parse`/`status`, ok per diff/show/log/read).**

## Verdetto: **CONFORME**

## R1 — CHIUSO
- Il ciclo su `view.edges` è sparito: il diff `6cdf7b4→610b43c` sostituisce `if e.payload.get("edges").is_some(){for edge in &view.edges{…}}` con `if let Some(v)=e.payload.get("edges"){mentioned.extend(edge_mentions(v,&view)?);}` (`src/selection.rs:67`, era riga 41). Grep confermato: nessuna occorrenza di `view.edges` in `src/selection.rs`; l'unico `for edge` residuo è la tokenizzazione del payload dell'evento (`src/selection.rs:33`, `tokens.chunks_exact(10)`).
- Attivazione **solo** degli archi dichiarati nell'evento stesso: `edge_mentions` legge l'array `edges` del payload corrente (`src/selection.rs:21-44`) e gli endpoint di quell'evento sono gli unici menzionati; attivazione pushata a `e.seq` per evento, dedup dentro lo stesso evento (`src/selection.rs:68`, `mentioned.dedup_by(|a,b|a.0==b.0)`) — L2.13 righe 79-84.
- Mapping endpoint→id vista con errore per non mappabili: `…find(…).ok_or(L2Error::Schema)?` (`src/selection.rs:35-39`) — Schema, come richiesto; identità = id L1 canonico (confronto con encoder canonico di escape, `src/selection.rs:19-20,36-38`).
- Test multi-arco: `distinct_edge_events_activate_only_their_own_endpoints` (`tests/l2_selection.rs:166-175`) VERDE al gate; sotto la vecchia semantica l'evento seq 4 (C–D) avrebbe attivato anche A–B e il test cadrebbe → il test discrimina davvero.

## Punto 2 — L2.27 applicata effettivamente (non per caso residuo)
La legge L2.27 (`notes/L2_legge_selezione.md:210-238`, firmata «Lettura A — L2.3/L2.13») vince: ogni evento che dichiara l'arco (originale o riprova) attiva gli endpoint di **quell'evento** a seq dell'evento. Il meccanismo nuovo è l'estrazione per-evento dal payload (`src/selection.rs:44,67`), non il vecchio ciclo sulla vista: la riprova attiva perché ogni evento riprovato produce `mentioned` propri → attivazione con `seq=dell'evento` (`src/selection.rs:68`).
- Test di riprova `repeated_edge_reactivates_endpoints_on_each_event` (`tests/l2_selection.rs:186-196`, arco A→B riprovato a seq 3 e 4, attesi `Select{at:3,tick:2,downs:[2,2]}` e `{at:4,tick:3,…}`): passa **soltanto** perché l'estrazione per-evento dal payload è in atto (e non dal ciclo residuo sulla vista); il vecchio ciclo lo avrebbe fatto passare per caso residuo (vista con un solo arco), ma è il test multi-arco (sopra) a escludere la semantica vecchia. Insieme fissano "ogni evento che dichiara l'arco è anch'esso una menzione" — semantica voluta da L2.27, non accidentale.
- Attenzione all'ordine: i `downs=[2,2]` sono le `origin_seq` L1 dei nodi A e B (dichiarati a seq 2), l'attivazione avviene invece alle seq dell'evento (3 e 4): coerente con la distinzione L2.27 tra regola d'identità/origin (D18, righe 222-225) e regola d'attivazione.

## R2 — CHIUSO
- `let tau=config.map(|x|x.1).unwrap_or(1)` eliminato; sostituito in finestra da `let tau=config.ok_or(L2Error::Config)?.1;` (`src/selection.rs:51`). Config mancante al ramo valutativo su menzioni → `config.ok_or(L2Error::Config)?` (`src/selection.rs:69`). Grep: nessun residuo `unwrap_or` su config; gli altri `unwrap_or` sono la provenienza fallback dei nodi già presente pre-fix (`src/selection.rs:66`, fuori da questo rilievo) e la selezione del cutoff (`:73`).
- Test: `in_window_select_without_config_is_error` (`tests/l2_selection.rs:178-183`, barrier senza config + select in finestra con `downs`) → `L2Error::Config`, VERDE al gate. Conforme a L2.2 (`notes/L2_legge_selezione.md:14-17`: errore, mai default impliciti).

## R4 — CHIUSO
- Diff: `-use crate::{EventType,Log,Value};` → `use crate::{Log,Value};` (`src/selection.rs:3`); grep su `src/selection.rs`: zero occorrenze di `EventType` — rimosso davvero, non commentato. `EventType` resta legittimamente usato dai test (`tests/l2_selection.rs:1,5`).

## Punto 5 — coerenza L3 e API
- `git diff 2a9e18e..HEAD -- src/memory.rs`: **output vuoto** → `src/memory.rs` invariato in tutto l'arco del mandato; la coerenza con L3.3 (conteggio per eventi distinti) era già conforme e L2.27 la preserva (`notes/L2_legge_selezione.md:231-234`).
- API pubblica di selection invariate e conformi a L2.20 (`notes/L2_legge_selezione.md:123-129`): `ActiveSet`, `Decision`, `build_active`, `evaluate`, `encode`, `L2Error` mantengono le stesse firme (il diff tocca un `use`, aggiunge la fn privata `edge_mentions` e riscrive le due linee interne; nessuna firma `pub` mutata).
- Perimetro `git diff --stat 2a9e18e..HEAD`: solo `notes/L2_legge_selezione.md` (+32), `src/selection.rs` (+32/−3), `tests/l2_selection.rs` (+33). Nessuna area già chiusa toccata: **E1, preimage, archiviazione futura, provenienza sono non toccati** e non riaperti in questo audit.

## Ordine dei commit
`6cdf7b4` = solo `tests/l2_selection.rs` (+20), messaggio con autorizzazione umana «autorizzo il fix col percorso consueto test-first, voglio un caso multi-arco», nessuno stub/production code → conforme. `610b43c` = fix + test di riprova (+13) insieme: ammesso dalle righe 235-238 di L2.27, che fissano i due test come pin di non-regressione che «accompagna il fix R1», e dichiarato dal mandato — legalmente coperto, non violazione del test-first.

## Rilievi (in ordine di gravità: 2, nessuno bloccante)
1. **MINORE (procedurale, non di codice).** Nessuna prova del rosso a runtime su `6cdf7b4`: il gate la dichiara **ricostruita staticamente** dal diff (`reports/gate_fix_l2_r1_r2.md:131`, limitazione 3), non rieseguita. Coerente con i mandati passati, ma resta un'evidenza ricostruttiva: se si vuole chiudere a runtime servirrebbe `git worktree add <scratch> 6cdf7b4` + run (già descritto dal tester), su nuovo mandato.
2. **MINORE (documento, fuori dalla revisione).** Working tree con `BLOCKER.md` cancellato e non committato (` D`, dal gate `reports/gate_fix_l2_r1_r2.md:106`): nota, non incide sul codice auditato (la revisione git è invariata), resta da sanare da chi ne ha titolo.

## Limitazioni
1. Nessun test eseguito (fuori competenza): il gate tester verde sulla medesima revisione è assunto come precondizione.
2. `git rev-parse HEAD` e `git status --porcelain` negati dalla sandbox: hash di HEAD confermato via `.git/refs/heads/main`; working tree non confrontato integrale con HEAD (letture Git per diff/show/log permesse).
3. SHA-256 di policy e file citati dal gate non ricalcolati indipendentemente.
4. Sessione Mnemosyne di audit aperta e chiusa; nessuna proposta di memoria fatta e nessuna decisione di review emessa (competenza dell'orchestratore).

Nessuna accettazione di livello dichiarata: la firma resta umana.
