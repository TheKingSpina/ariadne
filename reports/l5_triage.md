# Triage L5 Sociale — resoconto (fase unica, ZERO codice)

- **Mandato:** TRIAGE L5 (settimo mandato, fase unica, nessun codice).
- **Repo:** `/Users/spina/Ariadne` — HEAD al momento del triage: `1a479c6`
  (`1a479c6d4e093e36b1eb1bc4dfde5f6ae8f9147b`, verificato; ciclo L4 chiuso e
  accettato, 146/146 debug + 146/146 release).
- **Modulo atteso (solo nomenclatura, in triage NON si scrive):**
  `src/social.rs`.
- **Modello completo:** `opencode/glm-5.3-flash-free` (provider `opencode`,
  variante: **nessuna** — default del catalogo), classe `light`.
- **Canale:** «Free-first», policy `.opencode/routing.md` v1 (ATTIVA).
  Nessun canale differeziante o a pagamento richiesto dal triage: nessuna
  spesa, nessun segnale di attraversamento non-free → **nessuno STOP**.
- **Sessione Mnemosyne:** `ses_e0cf60b3-c9a6-4638-b65a-8853f6988c36`
  (progetto `opencode:Ariadne`).
- **SPEC di perimetro:** `spec/L5_SPEC_v0.1.md` — letta integralmente
  (28 righe, §1–§6); nessuna modifica, nessuna estensione, nessuna
  interpretazione normativa.

## Deliverable

- `notes/L5_domande_sociale.md` — triage completo, **15 domande Q1–Q15**
  (numerato e verifica: Q1 tipi evento; Q2 schema payload/bibliografia;
  Q3 id canonico altri; Q4 fonte osservazioni SENSE; Q5 statistiche;
  Q6 PREDICT; Q7 trigger MODEL_UPDATE; Q8 cutoff L5; Q9 teoria della mente
  di secondo ordine; Q10 divergenza scoped; Q11 determinismo/zero RNG;
  Q12 interazione con L4; Q13 errori/frontiera; Q14 naming; Q15 rischio
  SPEC), ciascuna con «Default da deliberare — proposta» marcata, sezione
  «Termini nuovi non definiti» (13 voci), «Non procedere prima delle
  risposte» (7 voci) e footer di provenienza. **NON committato.**
- `reports/l5_triage.md` — questo resoconto (compatto). **NON committato.**

## Segnali di canale

Nessun punto del triage richiede canali differenzianti, signali a pagamento
o API non libere: la totalità delle scelte è testuale/strutturale e
ripercorre i pattern vigenti L2.1/L3.1/L4.1. Il canale «provider L»
lasciato dall'umano a L4 è interpretato dall'orchestratore secondo policy
v1 (free-first) — non è una decisione di triage e resta invariatamente
soggetto a firma per qualunque spesa.

## Anomalie GENERALI della SPEC L5 v0.1 (segnalate, non risolte)

(a) «comportamento» osservabile e spazio dei valori non definiti;
(b) forma delle osservazioni nel payload SENSE non definita — lo stub
`src/cycle.rs` (`{"stimolo": "<draw PCG64>"}`) non ha alcun concetto di
«altro»: punto di rischio massimo (Q4), tocca potenzialmente L0;
(c) statistiche del modello (Q5), forma della previsione (Q6), trigger di
MODEL_UPDATE (Q7), cutoff L5 (Q8), criterio del test di secondo ordine
(Q9), errori (Q13), API pubbliche (Q14) non scritti dalla SPEC;
(d) «rappresentazioni predittive» senza chiarimento su determinismo/stoca</s>
stica in v0.1 (Q11);
(e) «terzo»/«terza mente» (§5) non definito;
(f) «viste L1–L4» come input: contratto SelfDocument della legge L4 da
riportare in sola lettura, senza ri-interpretazioni — massima attenzione
nel momento in cui la legge L5 dipenderà dal contratto L4.

## Voci esplicitamente «NON decido io» (dal triage)

- Canale/provider per l'implementazione L5 (firma umana; policy v1).
- **Fonte delle osservazioni SENSE (Q4)** — opzioni (a) forma firmata dalla
  legge L5 / (b) v0.1 solo stub / (c) multistanza fuori scope: selettore
  umano, MASSIMO rischio del ciclo (tocco potenziale L0/cycle.rs).
- Semantica di business: spazio dei «comportamenti» (Q5/Q6), criterio
  esatto di «disaccordo» (Q7), tie-break dell'argmax (Q11), criterio del
  test di secondo ordine (Q9).
- Scritture dirette multi-log/altri istanze: fuori scope v0.1.
- Qualunque spesa/canale a pagamento: STOP senza firma umana.
- Modifiche alla SPEC L5: la via unica è una legge firmata; impossibili.
- Nomi esatti di chiavi payload, dominio `l5`, errori, API (Q2/Q6/Q13/Q14).

## Stato procedurale

Nessun commit, nessun push, nessuna scrittura in `src/` o `tests/`: solo
i due deliverable documentali + questa sessione di memoria. La checklist
NON è dichiarata verde: il triage è documento di sola proposta, la guida
decisiva resta all'orchestratore/umano, il gate spetterà al tester e
l'accettazione all'umano a fine ciclo L5.

---

**Footer di provenienza.**
Modello: `opencode/glm-5.3-flash-free` (classe `light`; variante: nessuna —
default del catalogo), assegnato dal Router (policy `.opencode/routing.md`
v1, ATTIVA, canale «Free-first»), 31/10/2026. HEAD verificato `1a479c6`.
Non si dichiara conformità né completamento: il prodotto è il triage, ogni
«proposta» resta da firmare.
