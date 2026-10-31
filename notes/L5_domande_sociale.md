# Triage L5 — domande da risolvere prima dei test

Documento di triage in sola lettura: non modifica né interpreta in via
normativa la SPEC vigente. Le proposte indicate come default sono scelte da
sottoporre a firma; non autorizzano da sole implementazione o deviazione
dalle leggi L0–L4. Segue il pattern `notes/L4_domande_identita.md`. HEAD al
momento del triage: `1a479c6` (`1a479c6d4e093e36b1eb1bc4dfde5f6ae8f9147b`,
verificato; ciclo L4 chiuso e accettato, 146/146 debug+release).

Contratto di perimetro: `spec/L5_SPEC_v0.1.md` §1–§6 (28 righe: §1 perimetro,
§2 input, §3 costruzione, §4 predizione e correzione, §5 frontiera
intersoggettiva, §6 checklist). Ogni citazione «§N» si riferisce a quel file;
le citazioni delle leggi riportano le intestazioni come in
`notes/L{2,3,4}_legge_*.md`.

## Q1 — Tipi logici degli eventi L5 o discriminante `l5`?

- **L5:** §2, «Scritture solo eventi propri: MODEL_UPDATE, PREDICT»; §1,
  «Le sue decisioni sono eventi in coda a L0: replayabili come tutto il
  resto».
- **Leggi:** L0 §2 enumera `type` come `SENSE | ACTION | STATE` —
  enumerazione chiusa; il pattern vigente è fissato da L2.1, L3.1 e L4.1:
  nessun tipo logico L0 nuovo, ogni scritta di livello è un `ACTION` con
  discriminante a dominio chiuso (`l2`/`l3`/`l4`). L4.1 afferma che L0 §2
  resta intatto.
- **Indeterminato:** la SPEC L5 non dice se MODEL_UPDATE e PREDICT siano
  (a) due nuovi tipi logici L0 — richiederebbe emendamento firmato di L0 §2
  e riverifica di parser/preimage — oppure (b) eventi `ACTION` con
  discriminante `l5`.
- **Default da deliberare — proposta:** (b) `ACTION` con discriminante
  `l5`, dominio chiuso `model_update | predict` (pattern L2.1/L3.1/L4.1);
  l'alternativa (a) richiede firma umana esplicita perché modifica la
  legge L0, non solo la SPEC L5.
- **Tocca:** nuovo `src/social.rs`; `event.rs`/`log.rs` solo in caso di
  emendamento L0, cosa che la presente nota non autorizza.

## Q2 — Schema chiuso dei payload `l5` e bibliografia delle seq

- **L5:** §2; §4, «PREDICT emette una previsione … citando le seq delle
  osservazioni usate»; §3, «id canonico deterministico».
- **Leggi:** L0 §3 payload flat solo `i64`/stringhe; L1 §3 (F1) e L2.1
  risolvono le liste come stringhe con JSON canonico interno; schema chiuso
  a tag: L2.21, L3.1, L4.1 (chiavi extra → errore).
- **Indeterminato:** la SPEC non specifica nomi né chiavi né tipi dei
  payload `l5`: che campi porta `model_update` (identità dell'altro? quale
  statistica corretta? provenienza?), che campi `predict` (identità?
  previsione? bibliografia delle seq osservate?).
- **Default da deliberare — proposta:** schema chiuso pattern
  L2.21/L3.1/L4.1, allineato alla costruzione L4.6: `l5="predict"` con
  `at` (anchoring L2.18 sull'osservazione di riferimento), `id` dell'altro,
  previsione (forma intera/stringa da firmare), `sources` (stringa JSON
  canonica F1 di seq osservate, dedup crescente — stesso formato delle
  `sources` L4.6); `l5="model_update"` con `at`, `id`, statistica corretta
  e `sources`. L'elenco ESATTO delle chiavi va firmato in legge L5.
- **Tocca:** `src/social.rs`; `payload.rs` solo se servisse un helper nuovo.

## Q3 — Id canonico degli «altri»: deriva, collisioni, relazione con id L1

- **L5:** §3, «Ogni altro agente ha un id canonico deterministico, come i
  nodi L1».
- **Leggi:** L1 D18 — id nodo = SHA-256 del canone `(kind,key)`; L4
  (checklist) usa gli id L1 di nodi/archi; il confronto col significato di
  «identità» è stato chiuso per convenzione a L4 (collisione firmata),
  non replicato qui.
- **Indeterminato:** la SPEC non definisce la tuple canonica che genera
  l'id dell'altro («come i nodi L1» è analogia, non formula), né lo scope
  del namespace, né la gestione delle collisioni, né cosa sia «altro
  agente» nella forma dei dati.
- **Default da deliberare — proposta:** id dell'altro = SHA-256 minuscolo
  del canone di una tuple dedicata (per es. `{"altro":<etichetta>}` —
  formula esatta da firmare), scope SEPARATO dallo spazio dei nodi L1/sé
  per escludere ogni collisione; determinismo garantito dalla stessa
  SHA-256.
- **Tocca:** `src/social.rs`.

## Q4 — Fonte delle osservazioni: come «l'altro» entra nel mio log via SENSE

- **L5:** §1, «l'altro è osservabile solo attraverso i SENSE che entra nel
  mio log»; §5, «niente canali nascosti».
- **Codice vigente:** `src/cycle.rs` produce l'UNICO payload SENSE come
  `{"stimolo": "<draw PCG64 in decimale>"}` — stub del mondo a stimolo
  singolo, senza alcun concetto di istanza/osservazione «di qualcuno».
  Nessuna SENSE vigente contiene l'informazione «chi ha fatto cosa».
- **Leggi:** L0 §4 budget draw (1 draw per tick al SENSE, 0 altrove);
  modificare la FORMA del payload SENSE non è previsto dalle leggi L0–L4:
  può richiedere emendamento L0 o forma firmata dalla legge L5 (come L4.1
  ha fatto per `l4`).
- **Indeterminato / opzioni:** (a) la legge L5 definisce la forma del
  payload SENSE «osservato» (campo con tuple canonica di Q3), toccando lo
  stub del ciclo — firma umana; (b) v0.1 osserva SOLO gli stimoli dello
  stub (ogni tick = comportamento «dell'ambiente», nessun «altro»
  identificato — semantica debole); (c) multistanza esplicita fuori scope
  v0.1.
- **Default da deliberare — proposta:** nessuna: l'ingresso stesso dei
  dati dell'altro nel log ha bisogno di una firma; vedi NON decido io
  (punto di rischio maggiore del ciclo).
- **Tocca:** potenzialmente `src/cycle.rs` (stub del mondo) — NON
  autorizzato qui.

## Q5 — Parametri statistici del modello: QUALI statistiche in v0.1

- **L5:** §3, «insieme di parametri statistici aggiornati solo dagli eventi
  SENSE che lo riguardano».
- **Leggi:** L0 §3: solo `i64`/stringhe, niente float; L2.4 usa aritmetica
  intera con saturazione; L3.3 conta riattivazioni intero; L4.4 pattern
  documento canonico e preimage.
- **Indeterminato:** quali statistiche? frequenze, «ultimo comportamento»,
  contatori di accordi/disaccordi? forma della rappresentazione byte per
  byte (documento del modello con hash?) non è scritta.
- **Default da deliberare — proposta:** modello minimo intero: `n`
  osservazioni totali, ultimo comportamento osservato, contatori per classe
  di comportamento (enumerata in firma), aggiornati da ogni SENSE
  riguardante; nessuna media/float: solo interi con saturazione stile L2;
  documento canonico del modello con eventuale hash a preimage (chiavi
  lessicografiche,
  `schema_version`, eventuale hash pattern exclude-self come L4.4). La
  correzione dello stato del modello passa SOLO via MODEL_UPDATE (§4).
- **Tocca:** `src/social.rs`.

## Q6 — PREDICT: forma della previsione, bibliografia, errore se manca il modello

- **L5:** §4, «PREDICT emette una previsione sul comportamento atteso,
  citando le seq delle osservazioni usate».
- **Leggi:** L2.20/L3.10/L4.6: l'append resta al chiamante via API L0; L4.7:
  il testo apposto è autorevole.
- **Indeterminato:** la SPEC non dice (a) dove vive l'output di PREDICT
  (evento `l5="predict"` apposto in log? sola vista?), (b) la forma della
  «previsione» (intero? stringa? numero/categoria?), (c) l'errore quando
  il modello dell'altro NON ESISTE (zero osservazioni).
- **Default da deliberare — proposta:** (a) PREDICT = evento
  `l5="predict"` apponibile via encode (API pubblica), con la previsione
  RI-LEGGIBILE in replay; (b) previsione = valore intero o classe stringa
  (firma); (c) errore dedicato `SocialError::NoModel` o rientrante in
  `Schema` — da firmare (vedi Q13). La previsione è funzione pura del
  modello corrente: stesso log → stessa previsione, byte per byte
  (checklist 1).
- **Tocca:** `src/social.rs`.

## Q7 — Trigger di MODEL_UPDATE: quando e con quale materiale

- **L5:** §4, «Ogni disaccordo con l'osservato genera un MODEL_UPDATE di
  correzione: il modello sbaglia, il log non sbaglia mai».
- **Leggi:** L4.7 — il testo apposto è autorevole e il ricalcolo divergente
  non è errore a runtime; L2.11 replay a sola scrittura. Il «modelo
  sbaglia» in §4 DEVE perciò vivere nel materiale apposto (update autorevoli),
  non in un ricalcolo emendante.
- **Indeterminato:** rispetto a cosa si misura il disaccordo (la previsione
  precedente apposta nel log? la storia successiva all'ancora?); quando
  scatta la correzione; con quale materiale.
- **Default da deliberare — proposta:** a ogni SENSE successiva a un
  `l5="predict"` dell'altro (finestra: predict a seq p, osservazione a seq
  o > p, stesso `id`), se il comportamento osservato differisce dalla
  previsione apposta → UN solo `model_update` con `at` = seq
  dell'osservazione; deterministico in replay (stesso log → stessi update,
  byte per byte). La definizione analitica di «disaccordo» è semantica di
  business: vedi NON decido io.
- **Tocca:** `src/social.rs`.

## Q8 — Cutoff L5 e interazione con finestre/cutoff L2/L3/l4

- **L5:** nessun accenno a cutoff o finestre.
- **Leggi:** L2.23/L2.25 (cutoff e finestra L2; un `l4` chiude la finestra
  L2), L3.9 (cutoff L3 sulle decisioni L3 in log), L4.6/L4.8 (cutoff L4
  sull'ultimo evento `l4`; prefix-base L2.22), L4.9 (nessun side-effect
  sulle cutoff altrui).
- **Indeterminato:** esiste un cutoff L5? come interagisce con la finestra
  L2 e con l'ultimo evento `l4`? un `l5` è comunque un evento non-l2 e
  CHIUDRA la finestra L2 per la legge vigente (L2.25: già accadono con
  `l3` e `l4`): va firmato esplicitamente per evitare sorprese.
- **Default da deliberare — proposta:** cutoff L5 = seq dell'ultimo evento
  `l5` nel log (pattern L3.9/L4.6); valutazione con prefix-base L2.22;
  `l5` NON altera cutoff L2/L3/L4 (nessun side-effect, analogo L4.9); la
  chiusura implicita della finestra L2 per un `l5` segue la legge vigente
  e va verificata con test (come già fatto per `l4` in check6 L4).
- **Tocca:** `src/social.rs`; letture su `selection.rs`/`memory.rs`/
  `identity.rs` senza mutarli.

## Q9 — Teoria della mente di secondo ordine (checklist 5): definizione operativa

- **L5:** §5, «I modelli degli altri possono riferirsi a terzi: la mente
  dell'altro che modella una terza mente, sempre come vista, mai come
  accesso»; §6, «Test di teoria della mente di secondo ordine superato».
- **Leggi:** L1 D18; la SPEC §3 impone che ogni modello sia vista derivata
  pura del log — un modello di «terzo» deve restare funzione pura del
  medesimo log, senza alcun accesso diretto all'altro.
- **Indeterminato:** rappresentazione dei «terzi»; criterio di superamento
  del test (cosa si paragona, su quale sotto-logista, in che forma).
- **Default da deliberare — proposta:** operazionalizzazione v0.1 minima:
  il modello dell'altro A contiene, per ogni terzo T osservato nel log, un
  sotto-modello statistico dello stesso tipo di Q5; il test di secondo
  ordine: la previsione che A attribuisce a T su un ulteriore oggetto U
  deve RIPRODURRE (byte per byte) la forma della previsione che il
  medesimo copia T-on-U produce quando costruita direttamente dal log —
  cioè composizione di viste pura, zero accessi. Criterio e operatori da
  firmare.
- **Tocca:** `src/social.rs`.

## Q10 — Divergenza scoped (checklist 4)

- **L5:** §6, «osservazioni diverse → modelli diversi».
- **Leggi:** L4 (firmata): divergenza SCOPED — nessuna iniettività
  universale; head_hash propaga ogni differenza di catena L0.
- **Indeterminato:** che asserzione; a quanto si estende l'iniettività
  (modello intero? sotto-modelli?).
- **Default da deliberare — proposta:** test alla maniera L4: due log che
  differiscono SOLO per un'osservazione rilevante (per Q5) → i
  sotto-modelli parametrici differiscono; il modello intero differisce per
  il head_hash/ancora L0. Formula scoped firmata.
- **Tocca:** `src/social.rs`, test.

## Q11 — Determinismo byte per byte e zero wall-clock/RNG/I-O

- **L5:** §3, «stesso log → stesso modello dell'altro, byte per byte»;
  §1, «rappresentazioni predittive, non credenze magiche».
- **Leggi:** L0 §4 budget draw; L4.5 zero stocastica esplicita; L4.10
  niente I/O/file.
- **Indeterminato:** la SPEC parla di «rappresentazioni predittive» ma NON
  chiarisce se le statistiche v0.1 siano deterministiche (contatori,
  argmax, soglie) o coinvolgenti componenti stochastiche; qualunque draw
  L5 oltre il budget L0 richiederebbe emendamento firmato (precedente:
  L4.5).
- **Default da deliberare — proposta:** sì: zero draw, zero stocastica —
  «predittivo» = funzione deterministica del solo modello (es. argmax dei
  contatori con tie-break deterministico da firmare); ogni componente
  stocastica futura richiede emendamento firmato di L0 §4 + legge L5.
  Il tie-break della semantica è parte del modulo: NON decido io solo per
  la firma.
- **Tocca:** `src/social.rs`; nessun tocco a `prng.rs`.

## Q12 — Interazione con L4: cosa un modello dell'altro può fare coi dati del sé

- **L5:** §2, «Log L0 e viste L1–L4, in sola lettura».
- **Leggi:** L4.8 — boot = replay pubblico puro; il documento del sé che
  L5 leggerebbe è `SelfDocument` byte-per-byte con head_hash anch'esso alla
  catena L0; L4.2 mappa gli errori propagati.
- **Indeterminato:** se il modello dell'altro può citare/ancorare a
  elementi del sé (self_hash? affermazioni? nessuna procedita? senza
  registro).
- **Default da deliberare — proposta:** i dati del sé sono IN LETTURA
  accessibili a L5 via `build_self` ma NON entrano nel modello dell'altro
  se non come osservazioni (nessuna ri-emissione); nessuna scritura L5
  che alteri cutoff o checkpoint L4 (analogia L4.6/L4.9). «Il sé visto
  dall'altro» è semantica di business — NON decido io.
- **Tocca:** `src/social.rs`; letture su `identity.rs` senza mutarlo.

## Q13 — Errori L5 e frontiera intersoggettiva (§5): test dimostrativi

- **L5:** §5, «L'altro non scrive mai direttamente nel mio log: tutto ciò
  che fa esiste per me solo come evento percepito — niente canali
  nascosti».
- **Leggi:** L4.8 `SelfError { Schema, HeadMatch }` — pattern vivente;
  L0 `log.verify()` come invariante di catena.
- **Indeterminato:** errori L5 non definiti (naming, casi); «zero
  scritture dirette» come si DIMOSTRA (checklist 2).
- **Default da deliberare — proposta:** `SocialError { Schema, HeadMatch
  }` (pattern L4.8), eventualmente `NoModel` (Q6(c)) — da firmare;
  frontiera testata per costruzione (replay singolo su un solo log) più
  log.verify() L0, dato che in L0 l'append avviene solo tramite le API
  pubbliche — non decidere la «multi-log» qui.
- **Tocca:** `src/social.rs`.

## Q14 — Naming di modulo, tipi e API pubbliche

- **L5:** il mandato del triage fissa solo nomenclatura del modulo
  `src/social.rs`; la SPEC non dà nomi di tipi né di API.
- **Leggi/pattern:** L2.20, L3.10, L4.8 — solo cose pubbliche; naming
  vigente: `MemoryView`/`SelfDocument`/`SelfError`.
- **Indeterminato:** naming dei tipi (`OtherModel`? `SocialModel`?),
  funzioni (`build_model`? `evaluate_social`?), viste, decisioni
  (`Decision5`?: niente è fissato dalla SPEC.
- **Default da deliberare — proposta:** modulo `src/social.rs`; tipi
  `OtherModel`/`SocialView`/`SocialError`/`Decision5`; API `build_model(s)`
  + `evaluate_social(log, base)` + encode (il coefficiente evidenza
  RESTA «solo cose pubbliche» e nessuna collisione semantica con i termini
  L1/L4).
- **Tocca:** `src/social.rs`; `src/lib.rs` (una riga `pub mod social;`).

## Q15 — Rischio SPEC v0.1: eccezioni e ambiguità rilevate (senza risolverle)

- **Rispetto alla SPEC (28 righe, sintetica):**
  (a) NON definisce né «comportamento» osservabile né lo spazio dei valori
  osservati;
  (b) NON definisce la forma delle osservazioni nel payload SENSE (Q4):
  oggi lo stub `cycle.rs` non ha alcun concetto di «altro»;
  (c) NON definisce statistiche (Q5), forma della previsione (Q6), trigger
  di correzione (Q7), cutoff L5 (Q8), criterio del test di secondo ordine
  (Q9), errori (Q13), API pubbliche (Q14);
  (d) «parametri statistici» non chiarisci se deterministici in v0.1 (Q11);
  (e) «terzo»/«terza mente» (§5) non è definito;
  (f) «viste L1–L4» come input: il contratto L4 (SelfDocument ancorato
  alla catena) dev'essere la sola fonte di verità del sé per L5 —
  ribadito, non ambiguito.
- **Default da deliberare — proposta:** nessuna modifica alla SPEC: le
  voci sopra sono osservazioni per la legge L5.
- **Tocca:** legge L5 futura.

## Termini nuovi non definiti in `spec/L5_SPEC_v0.1.md`

Elenco dei nomi usati dalla SPEC senza definizione normativa; nessuna
definizione è qui proposta:

- modello dell'altro / modelli degli altri (oggetto, statistiche, forma)
- id canonico («come i nodi L1» — analogia senza formula)
- altro agente / terza mente / terzo (cos'è nella forma dei dati)
- osservabile / osservazione via SENSE (forma del payload SENSE)
- comportamento (atteso/osservato — spazio dei valori)
- parametri statistici (quali? deterministici?)
- previsione (forma, ancoraggio, output)
- disaccordo con l'osservato (criterio)
- MODEL_UPDATE / PREDICT (campi, schema, tipi L0)
- teoria della mente di secondo ordine (criterio del test)
- vista derivata (L5 vs viste L1–L4: rapporti)
- frontiera intersoggettiva (operazionalizzazione del §5)
- costruzione del modello (la funzione esatta)

## Non procedere prima delle risposte

- Non dichiarare MODEL_UPDATE/PREDICT tipi logici L0 nuovi né dominio
  `l5` senza firma (Q1).
- Non codificare schema payload, id canonico, statistiche, previsione,
  trigger, cutoff, errori, API senza le firme (Q2–Q3, Q5–Q8, Q13–Q14).
- Non toccare lo stub SENSE di `cycle.rs` né la forma del payload SENSE
  senza firma esplicita (Q4 — punto di rischio massimo).
- Non introdurre componenti stocastiche né consumare draw oltre al budget
  L0 (Q11 — richiederebbe comunque un emendamento).
- Non scrivere test che assumano iniettività universale dei log (Q10 —
  divergenza scoped).
- Non trattare il modello dell'altro come accesso diretto all'altro (§5)
  né definire canali nascosti/multi-log senza firma.
- Non introdurre float né I/O né wall-clock (L0 §3/§7; L4.10 per analogia
  normativa).

---

**Footer di provenienza.**

- Modello: `opencode/glm-5.3-flash-free` (classe `light`; variante: nessuna
  — default del catalogo) assegnato dal Router (policy
  `.opencode/routing.md` v1, ATTIVA, canale «Free-first»).
- Data: 31/10/2026. HEAD: `1a479c6`
  (`1a479c6d4e093e36b1eb1bc4dfde5f6ae8f9147b`, verificato).
- **Documento di TRIAGE, non di decisione:** ogni «proposta» indica un
  default da sottoporre a firma; nessun deliverable qui autorizza
  implementazione, legge o deviazione da L0–L4 (vedi NON decido io).
  NESSUN COMMIT: questo file e `reports/l5_triage.md` restano non
  committati, per il mandato (l'orchestratore committerà dopo la firma
  umana).
