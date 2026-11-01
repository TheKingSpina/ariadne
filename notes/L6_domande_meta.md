# Triage L6 — domande da risolvere prima dei test

Documento di triage in sola lettura: non modifica né interpreta in via
normativa la SPEC vigente. Le proposte indicate come default sono scelte da
sottoporre a firma; non autorizzano da sole implementazione o deviazione
dalle leggi L0–L5. Segue il pattern `notes/L5_domande_sociale.md`. HEAD al
momento del triage: `ab8b1af` (verificata; ciclo L5 chiuso, accettato e
pushato, 166/166 debug+release).

**Provenienza (dichiarata):** triage redatto dall'ORCHESTRATORE (GLM-5.3
dell'harness corrente), non da un producer free — la quota free OpenCode era
esaurita e l'umano ha firmato il canale «Triage da orchestratore»
(01/11/2026). Nessun altro passaggio fuori canale; la firma sul triage resta
all'umano.

Contratto di perimetro: `spec/L6_SPEC_v0.2.md` §1–§6 (32 righe: §1 perimetro
valutazione, §2 disegno appaiato, §3 parametri fissati, §4 statistica e
criterio, §5 validità, §6 checklist). Ogni «§N» si riferisce a quel file; le
citazioni delle leggi riportano le intestazioni come in
`notes/L{2,3,4,5}_legge_*.md`.

## Q1 — Che cos'è L6 nel codice: libreria, harness, o nuovo livello di log?

- **L6:** §1 «esperimento appaiato memoria vs controllo»; §2 disegno; §5
  «Replay bit-perfetto di ogni run dal suo log».
- **Leggi:** L0 §2 enum `SENSE | ACTION | STATE` chiuso; pattern L2.1/L3.1/
  L4.1/L5.1: ogni livello ha scritto eventi propri con discriminante
  (`l2`/`l3`/`l4`/`l5`). Ma la SPEC L6 NON menziona scritture proprie nel
  log: parla di run, braccia, reward, statistica — è un PROTOCOLLO
  SPERIMENTALE sopra la libreria, non una vista derivata.
- **Indeterminato:** L6 = (a) nuovo modulo di produzione con eventi `l6`;
  (b) harness sperimentale (bin) + modulo di valutazione, ZERO nuove
  scritture nel log; (c) solo analisi fuori dal crate.
- **Default da deliberare — proposta:** (b) L6 = harness sperimentale
  (`src/bin/l6_experiment.rs`, pattern `l0_replay.rs`) + modulo di
  valutazione puro (`src/valuation.rs`: statistiche su input interi); ZERO
  chiavi `l6`, ZERO nuovi tipi nel log; i run usano soltanto eventi L0
  standard (SENSE/ACTION) e i bracci usano le API pubbliche L1–L5 esistenti.
  La variante (a) è comunque possibile ma non richiesta dalla SPEC: se
  firmata, va trattata col pattern completo (schema chiuso, cutoff, replay).
- **Tocca:** `src/bin/l6_experiment.rs` (nuovo), `src/valuation.rs` (nuovo),
  `Cargo.toml` (dichiarazione bin). NON tocca `src/*.rs` di produzione.

## Q2 — Task, cue, regola nascosta, reward: definizioni operative

- **L6:** §2 «regola nascosta stabile», «le prove finali verificano la
  regola che i cue consentivano di inferire»; §4 «reward» con oracolo 1,00 e
  random 0,50.
- **Leggi/codice:** L0 §3 payload flat solo `i64`/stringhe; `cycle.rs`:
  `stimolo` = stringa decimale; reward float VIETATO nei payload.
- **Indeterminato:** SPEC non definisce: che cos'è un task; lo spazio dei
  cue; la forma della regola; lo spazio del reward.
- **Default da deliberare — proposta (tutto intero, zero float nei
  payload):**
  - **task** = una coppia di eventi percepiti: `SENSE {"cue": "<etichetta>"}`
    (lo stimolo) e, dopo la risposta del braccio, `SENSE {"esito": <i64 0..2>}`
    (la classe giusta RIVELATA dal mondo: feedback simmetrico, entrambe le
    braccia vedono gli stessi esiti);
  - **cue** = etichetta da un pool FISSO di 12 etichette generate
    deterministicamente dal seed della coppia (offline);
  - **regola nascosta** = tabella fissa `cue → classe_giusta (0|1|2)`,
    generata dal seed della coppia PRIMA dei run, ignota ai bracci, stabile
    per tutti i task del run (e identica su tutte le 100 coppie? NO: proposal
    = tabella PER RUN dal seed, così l'esperimento non memorizza una sola
    tabella ma la FAMIGLIA di regole inferibili per accumulo; la SPEC dice
    «stabile durante tutto il run»: per run, non per corpus);
  - **reward** = milli-reward intero `i64`: risposta corretta → `1000`,
    errata → `0` (oracolo = 1000; random uniforme ≈ 333, non 500: vedi
    anomalia SPEC in Q12);
  - **risposta** = `ACTION {"mossa": <i64 0..2>}` (stessa forma del ciclo
    L0, riusata).
- **Tocca:** solo harness e legge; nessun payload nuovo.

## Q3 — Le due braccia: memoria vs controllo, chi consulta cosa

- **L6:** §2 «braccio memoria vs braccio controllo (no memoria)».
- **Leggi:** viste L1–L5 pubbliche e pure; il controllo non deve consultare
  storia.
- **Indeterminato:** che strategia esatta abbiano le due braccia.
- **Default da deliberare — proposta:**
  - **braccio memoria:** dopo ogni `esito`, APPONE nel proprio log
    dichiarazioni L1: nodo `cue:<etichetta>` (kind `cue`), nodo
    `classe:<n>` (kind `classe`), arco `giusta` da cue a classe osservata;
    e gestisce la pipeline L2 (finestre/select/summary) e L3 (promozioni)
    secondo le leggi vigenti. La risposta al cue successivo = argmax sui
    contatori d'attivazione degli archi `giusta` uscenti dal nodo del cue
    (se il nodo non esiste o parità: tie-break classe 0, deterministico);
  - **braccio controllo:** risposta FISSA `{"mossa": 0}` (funzione
    memorialess dichiarata; il suo log NON contiene dichiarazioni né eventi
    l2/l3/l4/l5); reward atteso ≈ 333 se le classi sono bilanciate nella
    tabella;
  - le due braccia vedono IDENTICHE sequenze cue/esito (Q4); la differenza
    sta solo nelle strategie e nei log che producono.
- **Tocca:** harness; nessuna modifica alle viste.

## Q4 — Appaiamento e PRNG: come restano identici i task

- **L6:** §2 «stessa sequenza di task, stesso seed»; §3 «seed pre-fissati
  1000–1099».
- **Rischio:** se i task fossero generati online dai draw del log, il
  braccio memoria (che appende più eventi) divergerebbe immediatamente.
- **Default da deliberare — proposta:** generazione OFFLINE: per ogni seed
  `s ∈ [1000,1099]`, l'harness genera PRIMA (funzione pura su PCG64 con seed
  `s`) la lista dei task: N_task = 60 task per run (coda di 20 PROVE di test
  sulle ultime posizioni: proposal 40 cue di apprendimento + 20 di test, da
  firmare), ogni task = (cue, classe_giusta da tabella). POI i due run
  attraversano la STESSA lista. ZERO draw dentro i run (l'append non
  consuma PRNG; il draw budget L0 §4 non è toccato). La PCG64 serve SOLO
  all'harness per generare tabella e sequenza.
- **Tocca:** harness (`valuation` non tocca PRNG).

## Q5 — Timeout 5,0 secondi: dove vive e che cosa sanziona

- **L6:** §3 «Timeout per run: 5,0 secondi».
- **Leggi:** L0 §7 zero wall-clock IN PRODUZIONE (libreria).
- **Default da deliberare — proposta:** il timeout vive SOLO nell'harness
  (`l6_experiment.rs`): misura il tempo di ESECUZIONE del run (wall-clock
  consentito lì, dichiarato); un run che sfora 5,0 s è marcato VIOLAZIONE di
  protocollo → l'esperimento intero è FALLITO (criterio §4: «zero violazioni
  su tutti i 100 run»; nessuna esclusione selettiva). Nessun wall-clock in
  `valuation.rs` né in nessun modulo di produzione. Nella libreria: mai.
- **Tocca:** bin; niente altro.

## Q6 — Wilcoxon appaiato a una coda: implementazione e dipendenze

- **L6:** §4 «Wilcoxon appaiato a una coda … α = 0,01»; «Δ̄ ≥ 0,30».
- **Vincoli:** dipendenze minime (nessuna crate esterna di statistica,
  salvo firma); float ammessi SOLO nel modulo di valutazione, mai nei
  payload né nei visti L0–L5.
- **Indeterminato:** chi calcola la statistica; con quale precisione;
  Wilcoxon esatto o approssimato.
- **Default da deliberare — proposta:** modulo `src/valuation.rs` puro:
  input = differenze di milli-reward `Vec<i64>` (memoria − controllo per
  run); output = `W` (somma dei ranghi positivi), `n`, e `p_value: f64` via
  approssimazione normale con correzione di continuità (n=100: standard; la
  legge fissa la formula), `media_differenze: f64` in unità reward
  (`Δ̄ = media/1000`). Zero I/O, deterministico. Test-first sulla matematica
  (casi noti con tabelle di riferimento). Nessuna dipendenza esterna: la
  formula è ~30 righe.
- **Tocca:** `src/valuation.rs` (nuovo), `tests/l6_valuation.rs` (nuovo).

## Q7 — Artefatti dei run: JSONL, replay bit-perfetto, dove vivono

- **L6:** §3 «Log unificato JSONL per ogni run»; §5 «Replay bit-perfetto di
  ogni run dal suo log»; §6 checklist 3 «zero violazioni d'integrità».
- **Codice:** `Log::to_jsonl`/`from_jsonl` esistono (`src/log.rs:189/206`).
- **Default da deliberare — proposta:** l'harness scrive per ogni run un
  JSONL in `runs/l6/` (directory NON committata, aggiunta a `.gitignore`;
  la committatura si decide se mai in futuro); il replay bit-perfetto =
  per ogni run: `from_jsonl → verify() → ripetere la sequenza di decisioni
  del braccio sul log ricostruito → stesso esito reward byte per byte`;
  zero violazioni = 100/100 passano (replay + nessun timeout + nessun
  payload malformato). I 100 JSONL sono l'evidenza; il REPORT aggregato
  (`reports/l6_esperimento.md`, generato dall'harness) riporta per-run:
  seed, reward memoria, reward controllo, esito replay, e aggregati W, p,
  Δ̄, benchmark.
- **Tocca:** bin; `.gitignore`; `reports/l6_esperimento.md` (commit).

## Q8 — L'esperimento scrive nel log? (nessuna chiave l6)

- **L6:** nessuna menzione di eventi propri.
- **Default da deliberare — proposta:** L6 NON scrive chiavi nuove nel log
  in v0.1: i run contengono solo SENSE (cue/esito), ACTION (mossa) e — solo
  nel braccio memoria — le scritture delle leggi L1–L5 vigenti (dichiarazioni
  F1, `l2`, `l3`). Nessuna chiave `l6`; se mai servisse, emendamento firmato
  col pattern completo (schema chiuso, cutoff, replay).
- **Tocca:** nulla nel log di produzione.

## Q9 — Criterio di successo §4: esito EMPIRICO vs conformità del CODICE

- **L6:** §4 «Successo solo con: p < 0,01, Δ̄ ≥ 0,30, zero violazioni».
- **Rischio di governance:** il gate tester e l'audit NON possono «garantire»
  un p-value: verificheranno la CONFORMITÀ del codice (protocollo, replay,
  statistiche corrette, zero violazioni), non l'esito.
- **Default da deliberare — proposta:** la legge L6 distingue: (i)
  **conformità del codice** (test-first, gate, audit: protocollo
  implementato fedele, Wilcoxon corretto, replay dimostrato) — è ciò che il
  ciclo firma; (ii) **esito empirico** (p, Δ̄) — l'esperimento lo registra
  qualunque sia: se p ≥ 0.01 o Δ̄ < 0.30, il ciclo di CODICE resta valido ma
  l'esperimento dichiara ESITO NEGATIVO onesto nel report (mai manipolazione
  dei run per far passare il criterio: violazione di protocollo). La
  checklist §6 dell'accettazione umana valuta l'esito così com'è.
- **Tocca:** legge; report.

## Q10 — Benchmark passivi §4: come si registrano

- **L6:** §4 «oracolo (reward 1,00) e random uniforme (reward 0,50)».
- **Default da deliberare — proposta:** diagnostica dichiarata nel report:
  `oracolo = 1000` milli-reward (la tabella è nota all'oracolo);
  `random_uniforme = media su molte risposte uniformi → ≈ 333` con N_task
  note (NON 500: con 3 classi il random puro fa 1/3, non 1/2 — vedi
  anomalia Q12(d)). La SPEC li dice «registrati», non «criterio»: entrano
  solo nel report come confronto.
- **Tocca:** report.

## Q11 — Naming e collocazione dei file

- **Default da deliberare — proposta:** modulo `src/valuation.rs` (l'oggetto
  del livello è «valutazione», §1; «meta» resta alla SPEC come nome di
  livello, il codice non usa `meta.rs`); bin `src/bin/l6_experiment.rs`;
  test `tests/l6_valuation.rs` (matematica) + `tests/l6_experiment.rs`
  (protocollo end-to-end, con N_task ridotto per velocità dei test e 100
  run SOLO nell'harness manuale); directory `runs/l6/` (gitignored);
  report `reports/l6_esperimento.md` (committato). Una riga
  `pub mod valuation;` in `src/lib.rs`.
- **Tocca:** come sopra.

## Q12 — Rischi e anomalie SPEC (senza risolverle, elenco per la legge)

- (a) **«PCG64/MT19937»** (§3): il codice ha SOLO PCG64 (`Pcg64`);
  MT19937 non esiste nel repo: proposta = PCG64 sola, MT19937 = refuso SPEC
  da non implementare (eventuale emendamento SPEC separato);
- (b) **timeout 5,0 s** (§3): wall-clock ammesso solo nell'harness (Q5);
- (c) **float** (§4): solo in `valuation.rs` e nel report; payload interi
  (Q2);
- (d) **random uniforme = 0,50** (§4): con 3 classi il random fa 0,333: la
  SPEC presume 2 classi o sbaglia; proposta = registrare 1/3 ≈ 0,333 come
  benchmark (onestà matematica dichiarata) e segnalarlo;
- (e) **100 run × timeout 5 s**: tempo massimo teorico 500 s per il corpus;
  i run reali sono millisecondi (task sintetici): la verificherà il gate;
- (f) **«stesso seed»** (§2): chiarito in Q4 (tabella+sequenza per coppia,
  generate offline dal seed del run; i log dei due bracci DIVERGONO per
  contenuto — le viste di testa differiscono — e questo non è una violazione
  d'integrità: la sequenza percepita è identica);
- (g) **«Log unificato»** (§3): proposal = un JSONL per run per braccio
  (2 file per coppia, `runs/l6/seed_<s>_memoria.jsonl`,
  `runs/l6/seed_<s>_controllo.jsonl`): «unificato» = formato unico, non file
  unico;
- (h) **checklist §6 item 5** «Replay bit-perfetto dimostrato»: definito in
  Q7 (100/100 con `verify()` + identità decisioni/reward).

## Termini nuovi non definiti in `spec/L6_SPEC_v0.2.md`

- task / cue / prova di test / pool di cue
- regola nascosta (forma, spazio, per run o per corpus)
- reward (spazio, intero/float, scala)
- braccio / politica / risposta
- differenza di reward appaiata
- run / violazione di protocollo / corruzione d'integrità
- oracolo / random uniforme (spazio con 3 classi)
- log unificato (un file o un formato)
- replay bit-perfetto di un run (procedura)
- Wilcoxon appaiato (esatto/approssimato, formula)

## Non procedere prima delle risposte

- Non introdurre chiavi `l6` né tipi logici nuovi senza firma (Q1/Q8).
- Non scrivere wall-clock in moduli di produzione: solo harness (Q5).
- Non usare float nei payload: milli-reward interi (Q2); float solo in
  `valuation.rs` (Q6).
- Nessuna dipendenza esterna di statistica (Q6): formula fissa in legge.
- Non generare i task online nei run: generazione offline appaiata (Q4).
- Non manipolare i run per il criterio di successo: esito onesto (Q9).
- Non committare i JSONL dei run (Q7) senza firma.

---

**Footer di provenienza.**

- Redattore: ORCHESTRATORE (GLM-5.3-Flash dell'harness), canale firmato
  «Triage da orchestratore» (quota free OpenCode esaurita), 01/11/2026.
- Modello assegnato dal Router per i PROSSIMI dispatch: da decidere
  (implementazione L6: canale ancora da firmare dall'umano).
- Data: 01/11/2026. HEAD: `ab8b1af` (verificata).
- **Documento di TRIAGE, non di decisione:** ogni «proposta» indica un
  default da sottoporre a firma; nessun deliverable qui autorizza
  implementazione, legge o deviazione da L0–L5. NESSUN COMMIT: questo file
  e `reports/l6_triage.md` restano non committati (l'orchestratore
  commetterà dopo la firma umana).
