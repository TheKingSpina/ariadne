# Report triage L6 — valutazione (esperimento appaiato)

- **redattore**: orchestratore (GLM-5.3-Flash dell'harness), canale
  «Triage da orchestratore» firmato dall'umano il 01/11/2026 (quota free
  OpenCode esaurita); sessione orchestratore: corrente.
- **contratto**: `spec/L6_SPEC_v0.2.md` (§1–§6) + leggi L0–L5 in vigore;
  HEAD `ab8b1af`.
- **deliverable**: `notes/L6_domande_meta.md` — **12 domande (Q1–Q12)**,
  tutte con proposta marcata; sezione «Non procedere prima delle risposte»
  (7 divieti); 10 termini indefiniti; 8 anomalie SPEC segnalate (Q12).

## Sintesi delle proposte (una riga per domanda)

- **Q1** L6 = harness (`src/bin/l6_experiment.rs`) + valutazione pura
  (`src/valuation.rs`); ZERO chiavi `l6` nel log.
- **Q2** task = SENSE `{"cue"}` + risposta ACTION `{"mossa"}` + SENSE
  `{"esito"}`; regola = tabella cue→classe per run dal seed; reward =
  milli-reward intero (1000/0).
- **Q3** braccio memoria = dichiara nodi/archi L1 + pipeline L2/L3 e predice
  per argmax d'attivazione (tie-break classe 0); controllo = sempre classe 0.
- **Q4** generazione OFFLINE dei task dal seed della coppia; zero draw nei
  run; 60 task per run (40 apprendimento + 20 test, da firmare).
- **Q5** timeout 5,0 s solo nell'harness; violazione = esperimento fallito.
- **Q6** Wilcoxon in `src/valuation.rs`: approssimazione normale con
  correzione di continuità, nessuna dipendenza esterna, test-first.
- **Q7** JSONL per run in `runs/l6/` (gitignored); replay bit-perfetto =
  100/100 con verify() + identità decisioni; report aggregato committato.
- **Q8** nessuna chiave nuova nel log (solo SENSE/ACTION + scritture
  vigenti L1–L5 del braccio memoria).
- **Q9** distinzione firmata: conformità del CODICE (gate/audit) vs esito
  EMPIRICO (p, Δ̄) registrato onesto, mai manipolato.
- **Q10** benchmark passivi: oracolo 1000; random ≈ 333 (1/3 con 3 classi).
- **Q11** naming: `valuation.rs`, `l6_experiment.rs`, `tests/l6_*.rs`.
- **Q12** anomalie SPEC: MT19937 (refuso, solo PCG64), random 0,50 vs 1/3,
  wall-clock, float, «stesso seed» chiarito, log unificato = formato unico.

## Voci «NON decido io» (per firma umana)

1. Numero di task per run e ripartizione apprendimento/test (Q4: proposta
   40+20).
2. Tabella della regola: per run (proposta) o unica per corpus.
3. Canale per l'IMPLEMENTAZIONE L6 (la quota free è esaurita: scelta
   obbligatoria in mano all'umano: attendere reset / orchestratore / canale
   a pagamento).
4. Committatura o meno dei JSONL di run (proposta: mai).
5. Eventuale emendamento della SPEC (MT19937, random 0,50): solo con firma.

## Prossimo passo

Firma umana sul pacchetto Q1–Q12 (+ le voci sopra) → trascrizione della
legge L6 (`notes/L6_legge_valutazione.md`) → commit → implementazione
test-first sul canale che l'umano firmerà.
