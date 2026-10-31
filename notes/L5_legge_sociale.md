# L5 — Legge sociale (modelli degli altri e teoria della mente)
# (default firmati dall'orchestratore per delega umana esplicita, 31/10/2026)

Emenda operativamente `spec/L5_SPEC_v0.1.md`. Firma di delega umana:
firma di tutti i 15 default del triage `notes/L5_domande_sociale.md`
(31 ottobre 2026) + decisione puntuale su Q4 = (a). Il trascrittore
(orchestratore) firma i default e le precisazioni puntuali necessarie a
rendere la legge deterministica. La legge entra in vigore subito;
l'accettazione umana di fine ciclo (test-first → gate → audit) è la
ratifica della delega, sul consueto percorso L2/L3/L4. Dove questa legge
tace vale il pattern L0–L4; il producer SI ferma e chiede, niente
intuizioni.

## Terminologia firmata (chiude i termini indefiniti del triage)

- **PREDICT** → evento `ACTION` con payload `l5="predict"` (L5.1).
- **MODEL_UPDATE** → evento `ACTION` con payload `l5="model_update"`
  (L5.1).
- **altro / l'altro** → un agente NON-io osservabile nel mio log; forma
  dati: una label (stringa non vuota); id canonico a L5.3.
- **terzo / terza mente** → un altro distinto da quello modellato, come
  apparirebbe prospetticamente; in v0.1 è composizione pura secondo
  L5.8 e MAI come accesso.
- **osservazione** → evento `SENSE` con chiave payload `osservazioni`
  (L5.2): la forma unica con cui l'altro entra nel mio log (Q4(a)).
- **comportamento (atteso/osservato)** → classe `i64` ≥ 0; lo spazio dei
  valori è l'insieme delle classi osservate, che i contatori del modello
  enumerano (L5.5).
- **id canonico dell'altro** → SHA-256 minuscolo del canone della tuple
  dedicata di L5.3; scope SEPARATO dai nodi L1 e dal sé.
- **modello dell'altro** → l'oggetto L5: contatori interi su tutte
  osservazioni dell'altro nel log (L5.5); vista derivata pura a sola
  lettura.
- **previsione** → la classe argmax del modello (deterministica,
  L5.6), citante bibliografia delle seq osservate.
- **disaccordo con l'osservato** → previsione apposta ≠ classe dell'
  osservazione successiva che chiude la finestra (L5.7): UNA definizione
  analitica, firmata; niente criteri libreschi.
- **cutoff L5** → seq dell'ultimo evento `l5` del log (L5.4).
- **frontiera intersoggettiva** → proprietà strutturale: l'altro non ha
  alcun canale di scrittura nel log; ogni suo traccia è un evento SENSE
  percepito (L5.9), testata per costruzione e con `log.verify()` L0.
- **errori** → `SocialError { Schema, HeadMatch, NoModel }` (L5.10).
- **multistanza / multi-log / canali nascosti** → FUORI SCOPE v0.1:
  ogni estensione richiede emendamento firmato.
- **componente stocastica** → in L5 v0.1 NON ESISTE: zero draw (L5.11),
  precedente L4.5.

---

**L5.1 — Eventi l5 (Q1).** Nessun tipo logico L0 nuovo: `model_update` e
`predict` SONO eventi `ACTION` con chiave payload `l5` (stringa)
discriminante a dominio chiuso: `"predict" | "model_update"` (pattern
L2.1/L3.1/L4.1). `L0 §2` resta intatto; tipi L0 nuovi solo con
emendamento firmato. Un evento con `l5` e `l2`/`l3`/`l4` assieme →
`SocialError::Schema` (domini chiusi e mutualmente esclusivi). Eventi
senza `l5` sono fuori da L5.

**L5.2 — Osservazioni (Q4(a), firma puntuale).** L'altro entra nel log
SOLO come evento `SENSE` con chiave payload `osservazioni` (stringa F1:
JSON canonico di array interno). Membro = oggetto con chiavi in ordine
lessicografico: `{"altro":"<label>","comportamento":<i64>}` con label
non vuota e comportamento `i64` ≥ 0; doppi membri uguali sono AMMESSI e
contano ognuno; l'array vuoto è ammesso (evento senza effetto
statistico, MAI errore). Forma malformata (JSON non canonico, chiavi
extra, membri con shape errata, label vuota, comportamento negativo) →
`SocialError::Schema`. La chiave `osservazioni` su un evento
NON-`SENSE` → `SocialError::Schema` (specchio della regola di patch L4: solo un
  `SENSE` può portare osservazioni; la chiave vive solo nel proprio
  canale percettivo, mai sugli altri eventi).
**Perimetro v0.1 (firma puntuale):** NESSUN tocco a `src/cycle.rs` né a
qualsiasi generazione di contenuto del mondo: la legge definisce la
FORMA; le osservazioni sono apposte nel log dal chiamante via API L0
(NO append dentro L5); l'estensione generativa dello stub del mondo è
emendamento futuro firmato. Il budget dei draw di L0 §4 resta 1 draw per
tick al SENSE, 0 altrove.

**L5.3 — Id canonico dell'altro (Q3).** `id_altro(label)` = SHA-256
minuscolo (64 hex) del canone `{"altro":"<label>"}` — membri in ordine
lessicografico, nessuno spazio (pattern L1 D18, tuple dedicata). Scope
SEPARATO: non c'è relazione normativa con gli id L1; una label non crea
mai nodi L1.

**L5.4 — Cutoff L5 e interazioni (Q8).** Cutoff L5 = seq dell'ultimo evento
`l5` del log (`predict` o `model_update`, entrambi decisionali; pattern
L3.9/L4.6). Le viste L1–L4 sono input in sola lettura.
Interazioni: un `l5` è un evento `ACTION` privo di estrazioni → NON è
evento di valutazione L2/L3, NON entra nei cutoff L2 (L2.23)/L3
(L3.9)/L4 (L4.6); CHIUDE IMPLICITAMENTE una finestra L2 aperta (L2.25,
come già `l3` e `l4`): ciò va verificato in test (append a finestra
aperta + `l5`, replay intatto). NESSUN side-effect di L5 su L2/L3/L4;
L2/L3/L4 NON leggono `l5`.

**L5.5 — Modello (Q5).** Vista derivata pura, funzione del SOLO log:
per ogni altro che compaia in almeno una osservazione:
- `id` = id canonico della label (L5.3); le osservazioni si scorrono in
  ordine di seq;
- `n` = numero di osservazioni (u64, crescente, saturante);
- `ultimo` = `Option<i64>`: il comportamento dell'ultima osservazione per
  seq (nessuna → `None`; impossibile in un modello esistente, ma il tipo
  copre lo stato intermedio);
- `contatori` = mappa `i64 → u64` contatori per classe, incrementi
  SATURATI (stile L2), nel documento in ordine di classe crescente;
  SOLO interi: zero float, zero medie, zero wall-clock.
`build_social(log)` (L5.10) restituisce la `SocialView` con `head_hash`
= hash dell'ultimo evento del log, `models` in ordine di id
lessicografico. Stesso log → stessa vista, byte per byte (checklist 1).
Il codice deriva i contatori SOLO dalle osservazioni; la correzione
dello stato passa SOLTANTO via MODEL_UPDATE (L5.7, apposta dal
chiamante), MAI come scrittura derivata nel log.

**L5.6 — Previsione (Q6).** `predict_social(log, base, id)` = funzione
pura: modello del prefisso (coerenza L2.22 su `SocialBase`), poi:
- modello inesistente (zero osservazioni nel prefisso per quell'id) →
  `SocialError::NoModel`;
- previsione = classe **argmax** dei contatori; parità → la classe
  MINORE vince (tie-break firmato, deterministico);
- output: `Prediction { id: String, previsto: i64, sources: Vec<u64> }`
  con `sources` = dedup crescente delle seq di TUTTE le osservazioni di
  quell'id nel prefisso (la bibliografia della previsione, §4).
La previsione RESIDE in un evento: il chiamante che la vuole in log
appone `l5="predict"` con `encode5_predict` (L5.10) con `at` = seq
dell'ANCORA (ultima osservazione usata, anchoring L2.18), `id`,
`previsto` (= il campo previsto della `Prediction`), `sources`;
l'append resta al chiamante (L2.20/L3.10/L4.6). Il testo della
previsione apposto è autorevole in replay (pattern L2.11); le
discrepanze di forma sono Schema, quelle di contenuto NON sono errori
runtime.
Zero osservazioni nel prefisso e id NON registrato → `NoModel`;
payload `predict` malformato (chiavi extra, `id` non hex64, `at` che non
riferisce seq esistente, `sources` non canonico) → `Schema`.

**L5.7 — MODEL_UPDATE: trigger, materiale e finestra (Q7).** Nella
finestra (predict a seq `p` per id X, osservazione di X a seq `o > p`
prima successiva) vale: se il comportamento osservato a `o` differisce da
`previsto` del predict → UN SOLO evento `model_update` da generare,
con: `at` = `o`; `id` = X; `previsto` = campo del predict; `osservato`
= classe osservata; `sources` = dedup crescente della coppia di seq
CITATE nel trigger (la seq del predict `p` e la seq dell'osservazione
`o`; un altro materiale NON è citato). `tick` della decisione = tick
dell'evento osservazione `o` (L2.18). UN update PER FINESTRA: dopo l'osservazione che conclude la finestra, il
predict successivo riapre. La finestra è definita dal predict più
recente per quell'id: un secondo predict senza osservazioni tra i due
SOSTITUISCE il primo come anchor della finestra. In replay: gli eventi
`model_update` apposti sono AUTOREVOLI (pattern L2.11): il ricalcolo
divergente è MISMATCH DICHIARATO, non errore a runtime; gli errori di
FORMA sono Schema.

**L5.8 — Teoria della mente di secondo ordine (Q9).** In v0.1 la
composizione è pura e IDENTITÀ: il modello di A proiettato sul terzo T
è, per definizione, la STESSA vista statistica di T costruita dal log
(nessun accesso, nessun sottostato alimentato da canali); il test di
secondo ordine (checklist 5) verifica che la previsione composta
(A-su-T su un'ulteriore classe U) è la stessa funzione di previsione
diretta su T — byte per byte. Semantica più ricca (prospettiche
distinte) = emendamento futuro firmato. La composizione NON modifica
nessun modello e NON altera nulla.

**L5.9 — Frontiera intersoggettiva (Q13, §5).** Proprietà strutturale:
NESSUNA API di L5 scrive nel log; il log è IO (mio); l'append di eventi
l5 è apposto dal chiamante (la mia voce). Test dimostrativi: ogni
modello è costruito da UN solo log; l'ancora di catena (`head_hash`) e
`log.verify()` L0 garantiscono che nessun modello provenga da canali
esterni. Multi-log e canali nascosti: FUORI SCOPE (terminologia); non
implementati, non testati con semantica fuorviante.

**L5.10 — API pubblica (Q14).** Solo cose pubbliche, zero I/O, zero
wall-clock, zero RNG:
- `pub struct OtherModel { id: String, n: u64, ultimo: Option<i64>, contatori: BTreeMap<i64, u64> }`;
- `pub struct SocialView { head_hash: String, models: BTreeMap<String, OtherModel> }`;
- `pub struct Prediction { id: String, previsto: i64, sources: Vec<u64> }`;
- `pub enum SocialError { Schema, HeadMatch, NoModel }`;
- `pub struct SocialBase { head_hash: String, log_len: usize }` (coerenza
  di prefisso stile L2.22);
- `pub fn build_social(log: &Log) -> Result<SocialView, SocialError>` —
  replay a fine log (L5.5);
- `pub fn predict_social(log: &Log, base: &SocialBase, id: &str) ->
  Result<Prediction, SocialError>` — L5.6;
- `pub fn evaluate_social(log: &Log, base: &SocialBase) ->
  Result<Vec<Decision5>, SocialError>` — coerenza di prefisso; decisioni
  SOLO per finestre con osservazione `o > cutoff` (L5.4); una decisione
  per finestra (L5.7), in ordine di seq; nessuna decisione per eventi
  nella base;
- `pub enum Decision5 { ModelUpdate { at: u64, tick: u64, id: String,
  previsto: i64, osservato: i64, sources: Vec<u64> } }`;
- `pub fn encode5(d: &Decision5) -> Vec<(String, Value)>` — payload L0
  di `model_update` secondo L5.7;
- `pub fn encode5_predict(id: &str, at: u64, previsto: i64, sources:
  &[u64]) -> Vec<(String, Value)>` — payload L0 di `predict`
  (L5.6: `l5="predict"`, `at` come `Value::Int`, `id` stringa,
  `previsto` `Value::Int`, `sources` `Value::Str` F1).
Chiavi payload ESATTE:
- `predict`: `l5, at, id, previsto, sources` (chiuso);
- `model_update`: `l5, at, id, previsto, osservato, sources` (chiuso);
- qualsiasi chiave extra/assente/contaminata → `Schema`.
Modulo: `src/social.rs`; una riga `pub mod social;` in `src/lib.rs`.
Nessun tocco ad altri moduli di produzione.

**L5.11 — Zero stocastica (Q11).** L5 fa ZERO draw dal PRNG, nessuna
fonte esterna, nessuna attività stocastica: «predittivo» significa
esclusivamente la funzione deterministica L5.6. Ogni componente
stocastica futura richiede emendamento firmato di L0 §4 e della
presente legge. La suite L0–L4 resta VERDE, intatta (146 test).

---

## Checklist di uscita (gate tester)

1. **Replay byte per byte:** due `build_social` sul medesimo log →
   viste identiche; log di sola genesi → models vuoto, `head_hash` =
   hash genesi.
2. **Frontiera (checklist 2):** nessuna scrittura L5 nel log; append
   SOLO via API L0 dal chiamante; ogni valore riscontra `head_hash` e
   `log.verify()`.
3. **Ogni previsione è ancorata:** `sources` = seq osservate, dedup
   crescente; `at` del predict riferisce seq esistente.
4. **Divergenza scoped (checklist 4, Q10):** (i) log che differiscono
   solo per eventi irrilevanti (osservazioni di un ALTRO id, eventi
   non-rilevanti) → il modello di UN id dato resta
   IDENTICI byte per byte a livello di `models` di quell'id; (ii) log che
   differiscono per un'osservazione rilevante di X → il modello e i
   contatori di X cambiano; il modello INTERO differisce comunque per
   `head_hash`. NESSUN test che assuma iniettività universale.
5. **Test di secondo ordine (checklist 5, L5.8):** composizione pura =
   previsione diretta; byte per byte; nessuna via alternativa.
6. **Errori precisi e non-panic:** payload malformati (chiavi extra,
   JSON non canonico, negative, label vuote, l5 con l2/l3/l4,
   `osservazioni` su evento non-SENSE, id non-hex, `at` impossibile) →
   `Schema`; base incoerente → `HeadMatch`; predict su modello assente →
   `NoModel`. MAI panic né UB.
7. **Cutoff (L5.4):** eventi `l5` nel prefisso della base → ZERO
   decisioni; solo finestre post-cutoff; chiusura finestra L2 da `l5`
   verificata come già per `l4` (check6 L4 pattern); NESSUN side-effect
   su L2/L3/L4 (suite intera verde).
8. **Zero draw:** le API L5 non mutano lo stato PRNG; determinismo: due
   `evaluate_social` → decisioni identiche byte per byte.

## Rischi aperti da segnalare, non da decidere (per completezza)

- Lo stub del mondo non genera osservazioni: in v0.1 L5 è verificato su
  log fabbricati dai test (normale, ma la SPEC presume un mondo con
  altri: l'estensione generativa del mondo è emendamento firmato
  futuro).
- Semantica L5 ricca (prospettiche differenziate, second-order non
  identità) riservata a emendamenti futuri firmati.
- Crescita di contatori/labels non è vincolata da budget in v0.1.

## Firma

- [x] **orchestratore, per delega umana esplicita** — 31 ottobre 2026:
  firma di tutti i 15 default del triage (Q1 l5 dominio chiuso; Q2
  schema chiuso; Q3 id SHA-256 tuple dedicata; Q5 contatori interi
  saturanti; Q6 predict evento + NoModel; Q7 una finestra = un update; Q8
  cutoff L5 zero side-effects; Q9 composizione identità; Q10 divergenza
  scoped; Q11 zero stocastica; Q12 sé in sola lettura; Q13
  SocialError{Schema,HeadMatch,NoModel}; Q14 naming social.rs; Q15
  nessuna modifica SPEC) + Q4(a) con perimetro puntuale (stub del
  mondo NON toccato in v0.1: la legge definisce la forma, il chiamante
  appone le osservazioni) + precisazioni puntuali firmate: nomenclatura
  esatta delle chiavi payload, argmax tie-break = classe minore, `at`
  e `tick` delle decisioni, `sources` = `[min(o,p), max(o,p)]` (le seq
  citate dal trigger), comportamento = classe i64 ≥ 0. RATIFICA umana a
  fine ciclo.
- [ ] **umano** — accettazione fine ciclo nel consueto percorso
  (test-first → gate tester → audit reviewer → accettazione).
