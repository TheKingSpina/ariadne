# L6 — Legge della valutazione (esperimento appaiato memoria vs controllo)
# (default firmati dall'orchestratore per delega umana esplicita, 01/11/2026)

Emenda operativamente `spec/L6_SPEC_v0.2.md`. Firma di delega umana:
firma di tutti i 12 default del triage `notes/L6_domande_meta.md` (redatto
dall'orchestratore per canale firmato «Triage da orchestratore», quota free
esaurita) + decisione puntuale: **120 task per run (80 apprendimento + 40
prove di test)**. L'IMPLEMENTAZIONE è firmata al canale «orchestratore»
(GLM-5.3-Flash dell'harness, provenienza dichiarata in ogni commit) — la
quota free era esaurita e l'umano ha firmato questo canale il 01/11/2026.
Gate e audit restano al tester e al reviewer quando il canale lo consentirà.
L'accettazione umana di fine ciclo è la ratifica della delega. Dove questa
legge tace vale il pattern L0–L5.

## Terminologia firmata (chiude i termini indefiniti del triage)

- **run** → una coppia appaiata di log con seed `s`: braccio memoria e
  braccio controllo; 100 coppie, `s ∈ [1000, 1099]` (§3).
- **task** → tripla percettiva: `SENSE {"cue": "<etichetta>"}` → risposta
  `ACTION {"mossa": <i64 0..2>}` → `SENSE {"esito": <i64 0..2>}` (la classe
  giusta rivelata). Flusso di un run: **120 task = 80 di apprendimento +
  40 prove di test finali** (firma puntuale).
- **cue** → etichetta da un pool di **12 etichette** per run
  (`cue:00` … `cue:11`), generate offline dal seed.
- **regola nascosta** → tabella `cue → classe_giusta (0|1|2)`, **bilanciata
  (4 cue per classe)**, generata offline dal seed del run, ignota ai bracci,
  STABILE per l'intero run; diversa da run a run (famiglia di regole).
- **esito** → la classe giusta, rivelata dal mondo a entrambe le braccia in
  modo simmetrico: è l'unico canale di apprendimento.
- **milli-reward** → reward intero: `1000` se `mossa == classe_giusta`,
  `0` altrimenti. Nessun float nei payload.
- **braccio controllo (C)** → politica memorialess fissa: risponde sempre
  `{"mossa": 0}`.
- **braccio memoria (M)** → politica che consulta il PROPRIO log per
  replay (L6.4).
- **valutatore** → `src/valuation.rs`: matematica pura (Wilcoxon,
  statistiche), testabile in unità.
- **harness** → `src/bin/l6_experiment.rs`: orchestrazione, I/O,
  wall-clock (SOLO qui).
- **frontiera draw** → la generazione offline consuma draw PCG64 dedicati
  (seed del run) FUORI dal ciclo L0; i run NON consumano alcun draw. Il
  budget L0 §4 (1 draw/tick al SENSE del ciclo) resta intatto e non è
  toccato da L6.
- **violazione di protocollo** → timeout superato, payload malformato,
  replay non identico, task stream non appaiato: UNA violazione su UN run →
  l'esperimento intero è FALLITO (§4, nessuna esclusione selettiva).

---

**L6.1 — Natura del livello (Q1/Q8).** L6 non aggiunge tipi logici né
chiavi payload al log: nessuna chiave `l6`. Il livello è: modulo di
valutazione puro `src/valuation.rs` (pub in `src/lib.rs`) + harness
`src/bin/l6_experiment.rs` (pattern `l0_replay.rs`). Un log di run contiene
SOLO eventi L0 standard e, nel braccio memoria, le scritture vigenti delle
leggi L1–L5 se mai servissero (in v0.1 NON servono: il flusso è solo
SENSE/ACTION). Zero modifiche a `src/*.rs` di produzione L0–L5.

**L6.2 — Forme esatte dei payload (Q2).** Stesse forme del ciclo L0 dove
esistono:
- `SENSE {"cue": "<etichetta>"}` — la domanda del task;
- `ACTION {"mossa": <i64 0..2>}` — la risposta del braccio;
- `SENSE {"esito": <i64 0..2>}` — la classe giusta rivelata.
Chiavi extra o forme deviate in un run → violazione di protocollo. Le forme
`{"stimolo"}`/`{"mossa"}` del ciclo L0 NON sono alterate: L6 usa
`"cue"`/`"esito"` come chiavi SENSE dedicate dell'esperimento, mai sovra
scritte sul ciclo.

**L6.3 — Appaiamento offline (Q4).** Per ogni seed `s`: la funzione
`generate_pair(s)` in `valuation.rs` genera, con PCG64 inizializzato a `s`,
UNA volta sola: la tabella della regola (12 cue, bilanciata 4+4+4) e la
lista di 120 task (80 apprendimento + 40 test: i cue dei test sono scelti
tra i 12 con distribuzione deterministica, la lista test copre tutti i cue
almeno 3 volte). Le due braccia attraversano la STESSA lista: zero draw nei
run, zero draw nelle braccia. `generate_pair` è pura e testata
deterministicamente (stesso seed → stessa coppia tabella/lista, byte per
byte).

**L6.4 — Le due braccia (Q3).** Entrambe deterministiche, zero draw:
- **Controllo (C):** risponde `0` a ogni cue; il suo log non contiene
  dichiarazioni né eventi l2/l3/l4/l5.
- **Memoria (M):** risponde per **maggioranza dal proprio log per replay**:
  conta le coppie (cue, esito) già percepite nel proprio log; previsione =
  classe col conteggio massimo per quel cue; parità → classe minore; cue
  mai visto → classe 0. «La memoria» è il log: questo è l'esperimento. L'uso
  delle API L1–L5 per accelerare è facoltativo e non fa parte del criterio.

**L6.5 — Reward e metrica (Q2/Q10).** Per ogni risposta: milli-reward `1000`
se corretta, `0` se errata. Metrica di un run = **media dei milli-reward
sulle 40 PROVE di test** (i task di apprendimento sono addestramento, il
loro reward si registra diagnostica ma non entra nel criterio). Benchmark
passivi dichiarati: **oracolo = 1000**; **random uniforme = 333** (media
teorica con 3 classi, 1/3 in unità milli-reward: la SPEC indica 0,50,
anomalia matematica segnalata, si registra 1/3 e si documenta).

**L6.6 — Corpus (Q6.6).** 100 coppie, seed `1000..=1099`. Nessun run
manomesso, nessun seed escluso.

**L6.7 — Statistica (Q6).** In `valuation.rs`, funzione pura su input
`Vec<i64>` (differenze memoria−controllo delle medie di test, in
milli-reward):
- Wilcoxon signed-rank appaiato UNA CODA (alternativa «memoria > controllo»):
  scarti gli zeri; ranghi dei valori assoluti con TIE-BREAK = rango medio;
  `W` = somma dei ranghi delle differenze positive;
- approssimazione normale con correzione di continuità:
  `z = (W − n(n+1)/4 − 0.5) / sqrt(n(n+1)(2n+1)/24)`, `n` = coppie
  non-zero;
- `p = 0.5 * erfc(z / sqrt(2))` (erf/erfc implementati in casa, formula
  Abramowitz–Stegun 7.1.26, zero dipendenze);
- `delta_bar = media_differenze / 1000` (unità reward, float di solo
  output).
Float ammessi SOLO in `valuation.rs` (output) e nel report; MAI nei
payload, MAI nelle viste L0–L5.

**L6.8 — Criterio e onestà (Q9).** Due piani separati:
- **conformità del CODICE** (test-first, gate, audit): il protocollo è
  implementato fedelmente, la statistica è corretta, il replay è
  dimostrato — è ciò che il ciclo firma;
- **esito EMPIRICO**: `p`, `Δ̄`, violazioni — si REGISTRA qualunque sia:
  successo se e solo se `p < 0.01` E `Δ̄ ≥ 0.30` (in unità reward) E zero
  violazioni su tutti i 100 run (§4). Un esito negativo è un esito onesto
  nel report: MAI manipolare run, seed o task per far passare il criterio
  (sarebbe violazione di protocollo).

**L6.9 — Replay bit-perfetto (Q7).** Per ogni run e per ogni braccio:
`to_jsonl → from_jsonl → verify()` e RICALCOLO dell'intera sequenza di
risposte e reward dal log ricostruito: identità byte per byte. Replay
divergente → violazione. Zero violazioni = 200 log (100 coppie × 2 braccia)
tutti passano.

**L6.10 — Artefatti (Q7).** L'harness scrive:
- `runs/l6/seed_<s>_memoria.jsonl` e `runs/l6/seed_<s>_controllo.jsonl`
  («log unificato» = FORMATO unico, un file per run per braccio); la
  directory `runs/` va in `.gitignore` e NON è committata;
- `reports/l6_esperimento.md`: report aggregato committato (per run: seed,
  reward memoria test, reward controllo test, esito replay; corpus: W, n,
  p, Δ̄, benchmark, violazioni, verdict).
I/O consentito SOLO nell'harness; `valuation.rs` è puro.

**L6.11 — Timeout (Q5).** Wall-clock (std::time) SOLO nell'harness: ogni
coppia di run ha tetto 5,0 s (§3); sforamento → violazione di protocollo →
esperimento fallito. Nessun wall-clock in `valuation.rs` né nei moduli
L0–L5.

**L6.12 — Intoccabilità dei livelli.** `src/{log,event,payload,prng,cycle,
view,selection,memory,identity,social}.rs` e le loro leggi NON sono toccati.
La suite L0–L5 resta VERDE (166 test) e cresce solo coi test L6.

---

## Checklist di uscita (gate tester)

1. `generate_pair`: determinismo byte per byte; tabella bilanciata
   (4+4+4); 120 task (80+40); stesso seed → stessa coppia.
2. **Appaiamento**: le due braccia vedono la STESSA sequenza cue/esito;
   verificato per struttura (stessa lista) e per log (stream percettivo
   identico).
3. **Matematica Wilcoxon**: casi noti con valori di riferimento
   (piccoli n calcolati a mano), tie-break rango medio, zeri scartati,
   correzione di continuità, `p` decrescente al crescere di W; zero
   dipendenze esterne.
4. **Protocollo end-to-end ridotto nei test**: corpus ridotto (es. 5 seed)
   con reward dichiarati, replay bit-perfetto verificato, nessun draw nei
   run (PRNG intatto), nessun panic.
5. **Criterio**: `p < 0.01` E `Δ̄ ≥ 0.30` E zero violazioni, valutato sul
   corpus COMPLETO 100/100 dall'harness (esecuzione documentata nel
   report); esito registrato onesto qualunque sia.
6. **Replay**: 200/200 log ricostruiti e verificati (`verify()` +
   identità decisioni/reward).
7. **Frontiera wall-clock/draw**: nessun wall-clock fuori dall'harness;
   nessun draw nei run; suite L0–L5 (166) intatta e verde.
8. **Artefatti**: `runs/l6/` gitignored; `reports/l6_esperimento.md`
   committato con l'esito.

## Firma

- [x] **orchestratore, per delega umana esplicita** — 01/11/2026: firma di
  tutti i 12 default del triage + 120 task per run (80+40) + canale
  «orchestratore» per l'implementazione (quota free esaurita). Ratifica
  umana a fine ciclo.
- [ ] **umano** — accettazione fine ciclo (implementazione → gate →
  audit → esito empirico → accettazione).
