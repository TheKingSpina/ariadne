# L2 — Legge di selezione (default firmati dall'orchestratore, 23/10/2026)

Emenda operativa a `spec/L2_SPEC_v0.1.md`; approvata dall'umano nel messaggio
«producer L2 test-first con i default firmati». Da audit del reviewer, NON
pre-auditate nel codice. Dove questa legge tace, il producer SÍ ferma: niente
estrapolazione; chiede all'orchestratore.

**L2.1 — Tipi evento (Q1).** Nessun tipo logico L0 nuovo. Ogni scritta L2 è un
evento `ACTION` con chiave payload `l2` (stringa) discriminante:
`"config" | "select" | "summary" | "barrier"`. Payload L0 resta flat (solo
`i64`/stringa); le liste sono stringhe contenenti JSON canonico interno (come
F1 di `spec/L1_SPEC_v0.2.md`). Gli eventi senza `l2` non sono di L2.

**L2.2 — Config (Q5).** L'evento `l2="config"` porta `k` (i64 ≥ 0) e `tau`
(i64 ≥ 1). Vale dalla propria seq in poi; config successive ragpiano la
storia. Se la selection operates senza config / con config malformata →
errore, non aggiorna default impliciti.

**L2.3 — Attivazione (Q2).** Un evento L0 con estrazioni L1 attiva ogni
elemento che dichiara (ragso diretto) o menzionato (endpoint di arco) nello
stesso evento; una sola attivazione per elemento per seq, mai in replay.
L'**identità** dell'elemento è l'id L1: SHA-256 canonico json di `{kind,key}`
(stessa convenzione di `src/view.rs`). Il *contaggio* attivazioni avviene su
ogni evento del log con estrazioni, non sulla vista aggregata: la vista serve
per l'id canonico e per gli archi (L2.5), il payload grezzo per le menzioni.

**L2.4 — Salienza (Q3).** score(e, t) = Σ_att contributi interi:
contribution(a) = `2^24 >> m`, con `m = min(⌊(t − seq_a) / tau⌋, 24)`,
`seq_a` = seq dell'evento che ha generato l'attivazione, `t` = seq
dell'evento di valutazione. Puro `u64`/shift, nessun float.

**L2.5 — Peso archi (Q4).** Peso fisso `1.0` per ogni arco, indipendente da
`rel`. Contributo dell'arco = un attivazione al tick dell'evento che ha
dichiarato l'arco (seq di dichiarazione minimo, D18 L1); senza doppio
ricalcolo per riprova. Gli archi non attivano se stesso.

**L2.6 — Budget e parità (Q8).** Dopo ogni valutazione: se il set eccede `k`,
esce l'elemento con punteggio minimo; pari merito, rimane quello con seq di
origine minore. Se il set è sotto `k` dopo un evento: entra l'elemento non
attivo con punteggio massimo; pari merito, entra quello con seq di origine
minore. `k = 0` ammesso (nessun attivo).

**L2.7 — Momento di decisione (Q9).** La valutazione scatta dopo ogni evento
L0 con estrazioni; le selezioni scritte in quel momento ereditano il tick
dell'evento di valutazione. La replay segue l'ordine di seq.

**L2.8 — Barrier e offline (Q7).** `l2="barrier"` apre la finestra offline;
la chiude il primo evento L0 non-`l2` successivo. Se il payload contiene
`phase="close"` si chiude su quello; una barrier aperta al termine del log
resta valida. Barrier nidificate/duplicate durante una finestra attiva →
errore.

**L2.9 — Summary (Q11).** Evento `l2="summary"` con `expulsions` (stringa
JSON canonico, array di seq degli elementi espulsi in quella finestra) e
`text` (stringa deterministica: id degli espulsi in ordine di id, uniti da
`|`). Il consolidamento copre gli elementi **espulsi** nella finestra; le
sinstesi già registrate non si ri-emettono in replay. Nessun tema di
"espulsione pop" (rimozione dal log o dalla vista).

**L2.10 — Select (Q6).** Evento `l2="select"` con `ups` e `downs`, ciascuno
stringa JSON canonico di seq (i64) degli elementi che entrano / escono in
quella decisione. Ancoraggio = le seq degli eventi che hannogenerato la
valutazione (stessa lista su cui è calcolato il punteggio).

**L2.11 — Replay non ri-appende (Q12).** In replay il builder è a sola scrittura:
legge gli eventi `l2` già nel log e li tratta come **autoritativi**; non ri-emette
select/summary/barrier. Niente verifica del ricalcolo: le decisioni diverse dal
ralcolamento sono un mismatch dichiarato, non un errore catchable.

**L2.12 — Legge con L1 (Q10).** L2 invoca `view::build(&Log)` e verifica che
`view.head_hash` corrisponda all'hash dell'ultimo evento del log prima di
valutare; mi dash se non coincide. Zero scritture su log e vista oltre agli
eventi L2 propri (L2.1).

---

# Emendamenti L2.13–L2.20 (24/10/2026, risposte alle 8 domande del producer; firmati per delega umana, da audit del reviewer)

**L2.13 — Estrazione e menzioni (D1).** «Estrazioni» = chiavi payload top-level
`nodes`/`edges`, stringhe con array JSON canonico interno, secondo il criterio
F1 di `spec/L1_SPEC_v0.2.md`; stessi vincoli di parse (forma non canonica =
errore). Menzione = dichiarazione diretta in `nodes` o endpoint di arco in
`edges`, nello stesso evento. Identità = id L1. Se un elemento è sia
dichiarato sia menzionato nello stesso evento, una sola attivazione.

**L2.14 — `evaluate` ed eventi L2 già presenti (D2).** `build_active` è il
replay autorevole a fine log (L2.11). `evaluate(log, base)` produce
decisioni SOLO per eventi L0 successivi all'ultima decisione già registrata
nel log; gli eventi `l2` preesistenti non si ri-emettono. Append resta al
chiamante via API L0: il builder L2 non scrive.

**L2.15 — Emissione della sintesi (D3).** Consolidamento = un solo evento
`l2="summary"` emesso quando la finestra offline si chiude (via
`phase="close"` o primo evento L0 non-`l2` successivo), come primo evento L2
dopo la chiusura. Finestra chiusa senza espulsioni = nessun summary. La
barrier stessa non scrive: dichiara solo apertura/chiusura.

**L2.16 — Contenuto di summary (D4).** `expulsions` = stringa JSON canonico
di array delle `origin_seq` (i64) degli elementi espulsi nella finestra,
dedup e ordinato crescente. `text` = id L1 (hex minuscolo a 64) degli
espulsi, ordine lessicografico di id, separati da `|`. Le seq di
`expulsions` sono le origin_seq L1 degli elementi (L1 D17/D18), non le seq
delle decisioni select.

**L2.17 — Candidati all'ingresso (D5).** Solo gli elementi attivati
dall'evento corrente (L2.3) sono candidati all'ingresso; nessun richiamo di
elementi storici non attivati. Parità: entra il min seq di origine;
l'espulsione (L2.6) si valuta sull'insieme aggregato attivi+candidati.

**L2.18 — Provenienza del select (D6).** L'evento `l2="select"` porta la
chiave `at` (i64) = seq dell'evento L0 di valutazione che ha generato la
decisione: è così che la checklist «decisione ancorata a una seq di
origine» è soddisfatta. Il tick dell'evento select = tick dell'evento di
valutazione.

**L2.19 — Semantica di `phase="close"` (D7).** `l2="barrier"` con payload
`{"phase":"close"}` chiude la finestra corrente nello stesso evento; una
close senza finestra aperta = errore. Barrier senza `phase` apre; una
seconda apertura senza close = errore. Altro valore di `phase` = errore. Se
in replay si raggiunge la fine del log con finestra ancora aperta: nessun
errore, nessuna scrittura.

**L2.20 — API di `src/selection.rs` (D8).** Solo cose pubbliche: 
- `pub struct ActiveSet`; `pub fn build_active(log: &Log) -> Result<ActiveSet, L2Error>` = replay autorevole a fine log (L2.11, con check `head_hash` L2.12);
- `pub enum Decision { Select { at: u64, tick: u64, ups: Vec<u64>, downs: Vec<u64> }, Summary { expulsions: Vec<u64>, text: String } }` (tipo pubblico);
- `pub fn evaluate(log: &Log, base: &ActiveSet) -> Result<Vec<Decision>, L2Error>` = decisioni nuove per eventi L0 dopo l'ultima decisione già registrata nel log. L'output include sia `Select` sia `Summary`: in ordine di seq; **al momento della chiusura di una finestra, il `Summary` di quella finestra precede l'eventuale `Select` generato dallo stesso evento di chiusura** (L2.15: la sintesi è il primo evento L2 dopo la chiusura). La decisione select resta al massimo una per evento di valutazione (L2.7). Un Summary esiste solo se la finestra ha avuto almeno un'espulsione;
- `pub fn encode(d: &Decision) -> Vec<(String, Value)>` = encoding payload L0: per `Select` chiavi `l2="select"`, `at`, `ups`, `downs`; per `Summary` chiavi `l2="summary"`, `expulsions`, `text`; liste e numeri come stringhe JSON canoniche/i64 per la legge esistente (`Value::Str` per le liste, `Value::Int` per `at`);
- `pub enum L2Error { Config, Schema, Barrier, HeadMatch }`.
Append via la API di log resta al chiamante. Zero I/O, zero wall-clock, zero RNG.

**L2.21 — Rigore delle chiavi (24/10/2026, 10ª domanda).** Un payload con
`l2` presente ma valore fuori dal dominio di L2.1 → `L2Error::Schema`. Chiavi
extra rispetto alle prescritte in un evento `l2` (config: solo `k`,`tau`;
select: solo `at`,`ups`,`downs`; summary: solo `expulsions`,`text`; barrier:
solo `phase`) → `L2Error::Schema`. In `config`, `k`/`tau` mancanti, non-i64
o fuori dominio (`k≥0`, `tau≥1`) → `L2Error::Config`. Eventi senza `l2`
restano fuori da L2, come da L2.1.

**L2.22 — Semantica di `base` in `evaluate` (24/10/2026, 11ª domanda).**
`ActiveSet` porta i campi `log_len` (numero di eventi L0 consumati) e
`head_hash`. `evaluate(log, base)` richiede **coerenza di prefisso**: il log
deve avere ≥ `log_len` eventi e l'hash dell'evento in posizione `log_len−1`
deve coincidere con `base.head_hash`; in caso contrario → `L2Error::HeadMatch`.
La coppia (log, base) NON richiede che `base` copra l'intero log: il log può
estendersi oltre, e le decisioni riguardano la coda dopo `base.log_len`. Dentro
la coda, se dopo `base.log_len` esistono già eventi `l2` ("select"/"summary"),
essi sono autorevoli (L2.11/L2.14) e si considerano solo gli eventi successivi
all'ultimo di essi. Un `build_active` a fine log equivale a `log_len` = totale
eventi. Il check di L2.12 restaredicate: `head_hash` del log al termine
dell'ultimo evento consumato deve coincidere con `log.events()[base.log_len−1].hash`.

**L2.23 — Cutoff e eventi di controllo (24/10/2026, 12ª domanda).**
Gli eventi `l2="barrier"` e `l2="config"` sono eventi di **controllo**: non
sono decisioni e non partecipano al cutoff delle decisioni. Il cutoff usa
l'ultimo evento `l2` **decisionale** (`select` o `summary`) nel log: le
decisioni nuove riguardano solo gli eventi di valutazione con seq
successiva a quell'evento. Il summary della finestra è emesso al momento
della chiusura (L2.15), prima dell'eventuale `Select` generato dallo
stesso evento di chiusura quando l'evento di chiusura contiene estrazioni
(caso non-l2); se l'evento di chiusura è `phase="close"` (controllo),
nessun `Select` gli corrisponde. Le valutazioni continuano dentro una
finestra aperta: ogni evento non-`l2` con estrazioni produce la sua
decisione select.

**L2.24 — Parità completa (24/10/2026, 13ª domanda).**
Se punteggi uguali e origin_seq uguali (es. più elementi menzionati
nello stesso evento con k insufficiente), l'ordine è determinato
dall'id: **resta/entra** l'elemento con id L1 lessicograficamente
minore; l'altro è espulso. La regola generalizza il tie-break di L2.6 e
L2.17 quando anche la seq di origine coincide.

**L2.25 — Semantica della finestra offline (24/10/2026, 14ª domanda,
risoluzione della contraddizione L2.8/L2.23).**
La finestra offline contiene SOLO eventi `l2`. Dentro una finestra aperta:
- sono ammessi solo eventi `l2` (`select`, `summary` di windowprecedente,
  `barrier`) e **nessun** evento L0 non-l2;
- il primo evento non-`l2` dopo l'apertura chiude la finestra **ed è
  allo stesso tempo evento di valutazione**: emette il `Summary` della
  finestra (se ha expulsions) e POI il proprio `Select`, se genera
  transizioni (L2.15/L2.19);
- le `expulsions` del Summary della finestra sono le espanosioni
  **accumulate dai `Select` già apposti nella finestra** (eventi l2 che
  il chiamante ha scritto tra l'apertura e la chiusura, autorevoli per
  L2.11/M2.14); le espanoions generate dalla valutazione dell'evento di
  chiusura NON stanno in quel Summary: appaiono nel `Select` dello stesso
  evento di chiusura;
- `phase="close"` chiude la finestra nello stesso evento (L2.19), senza
  selezioni corrispondenti se è puro controllo;
- le valutazioni NON continuano dentro una finestra aperta: la clausola
  «Le valutazioni continuano dentro una finestra aperta» di L2.23 è
  **depennata qui** (abrogata da L2.25, commit `716e971`): dentro la finestra
  restano solo eventi `l2`.

**L2.26 — Identità degli espulsi quando origin_seq si ripete (24/10/2026,
15ª domanda).** `select` porta `ups`/`downs` come array di origin_seq
(L2.10, invariato). Se una medesima origin_seq in `downs`/`ups`
corrisponde a più elementi L1 con lo stesso origin_seq nello stesso
evento, l'identità esatta (id) che il Summary riporta in `text` è
dedotta dal replay applicando lo **stesso ordine di decisione della
legge** (L2.6 → L2.17 → L2.24) allo stato attivo immediatamente
precedente all'evento select: sono espulsi, in ordine, gli elementi che
il tie-break segnala finché il conteggio dei downs per quella origin_seq
è soddisfatto. `expulsions` resta array di origin_seq (L2.16); `text`
riporta gli id così dedotti, dedup, ordine lessicografico asc, separati
da `|`. La deduzione è deterministica e non muta `build_active`: il set
attivo resta autorevole dai select (L2.11).

---

**L2.27 — Riprova dell'arco (31 ottobre 2026; firmato dall'umano:
«Lettura A — L2.3/L2.13»).** Risoluzione della contraddizione fra L2.5 e
L2.3/L2.13 emersa dal fix di audit R1 (`src/selection.rs:41`, ciclo
compatto L0–L3 su `b2770bd`): **vince L2.3/L2.13**. Ogni evento L0 con
estrazioni che dichiara un arco (dichiarazione originale o riprova,
reiterata quante volte si vuole) attiva gli endpoint di **quell'evento**
(a seq dell'evento), una sola attivazione per elemento per seq (dedup
dentro lo stesso evento, già L2.13): una dichiarazione successiva dello
stesso arco è quindi anch'essa una menzione e si attiva come tale.
Chiarimenti conseguenti:
- in L2.5, «contributo dell'arco = un'attivazione al tick dell'evento
  che ha dichiarato l'arco» vale **per ogni evento che dichiara
  l'arco**; la parentesi «(seq di dichiarazione minimo, D18 L1)» è
  regola di **identità/origin della vista L1** (L1 D18: arco unico per
  `(src_id,dst_id,rel)` con origin_seq = seq minimo), NON una regola di
  attivazione;
- «senza doppio ricalcolo per riprova» significa: **nessuna**
  attivazione deduplicata o raddoppiata **dentro lo stesso evento**
  (L2.13), e nessun ricalcolo aggregato dalla vista; NON soppressione
  delle attivazioni derivanti da dichiarazioni ripetute in eventi
  successivi;
- coerente con la legge L3.3 (contaggio = «eventi L0 distinti che
  l'hanno attivata per L2.3/L2.13») e con l'implementazione L3
  accettata in `src/memory.rs` (per-evento, senza esenzioni di
  riprova), re-audit CONFORME del 31/10/2026.
Prova esecutiva: il test multi-arco (`distinct_edge_events_
activate_only_their_own_endpoints`) e il test di riprova reiterata
dello stesso arco che accompagna il fix R1 fissano questa semantica;
nessuno dei due può regredire senza violare questa legge.
