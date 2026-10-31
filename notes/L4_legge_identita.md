# L4 — Legge del sé (identità narrativa)
# (default firmati dall'orchestratore per delega umana esplicita, 31/10/2026)

Emenda operativamente `spec/L4_SPEC_v0.1.md`. Firma di delega umana:
«scegli tu» (31 ottobre 2026) in risposta al triage
`notes/L4_domande_identita.md` (Q1–Q11, 31 ottobre 2026). Il trascrittore
(orchestratore) ha firmato tutti i default proposti dal triage, più le
decisioni puntuali su Q5, Q9, Q10 e Q7. La legge entra in vigore subito;
l'accettazione umana di fine ciclo (test-first → gate → audit) è la
ratifica della delega, sul consueto percorso L2/L3. Dove questa legge
tace vale il pattern L0–L3; il producer SI ferma e chiede, niente
intuizioni.

## Terminologia firmata (chiude i termini indefiniti del triage)

- **NARRATE** → evento `ACTION` con payload `l4="narrate"` (L4.1).
- **IDENTITY_SNAPSHOT** → evento `ACTION` con payload `l4="snapshot"` (L4.1).
- **sé / documento del sé** → l'oggetto L4 di questa legge (L4.4). In
  tutta la legge «identità» e «id» continuano a significare l'id L1
  (uso vigente in L2.3 e L3.3–L3.7); la collisione di Q7 è chiusa per
  convenzione. Il codice mantiene il percorso di modulo della SPEC
  (`src/identity.rs`, tipi con prefisso `Self*`), ma NON espone il
  termine «identity» per l'oggetto L4.
- **revisione** → una coppia `narrate`+`snapshot` apposta in log
  (L4.6–L4.7); la storia delle revisioni vive SOLO nel log
  append-only.
- **affermazione narrativa / narrazione** → statement deterministico
  generato da un evento rilevante (L4.3), testo canonico su template
  vincolati, narrato in italiano, riproducibile byte per byte.
- **bibliografia** → lista `sources` di seq citate (L4.1 e L4.6).
- **delta** → le affermazioni generabili dagli eventi successivi
  all'ultimo evento `l4` del log (L4.6); non esiste un documento
  delta separato.
- **boot / ripresa sessione** → ogni ricostruzione con le funzioni
  pubbliche su un `&Log` (L4.8); niente file di identità separato,
  mai (L4.10).
- **integrità** → l'insieme delle condizioni L4.7 (ancoraggio di
  testa, coppia narrate/snapshot, verifica `self_hash`); violarne una
  → errore L4 esplicito.
- **componente stocastica** → in L4 v0.1 NON esiste: zero draw (L4.5).
- **divergenza** → proprietà scoped della checklist di uscita punto 2
  (Q9).
- **vista con revisioni** → il documento del sé più la sua storia di
  revisioni nel log; non è un costrutto nuovo oltre le viste L1–L3,
  è una vista derivata pura a sola lettura.
- **«un processo, mai un oggetto statico»** → retorica di perimetro
  della SPEC: il sé viene ricalcolato per replay a ogni boot, senza
  persistenza propria.

---

**L4.1 — Eventi l4 (Q1).** Nessun tipo logico L0 nuovo: L0 §2 resta
intatto (`SENSE | ACTION | STATE`). Ogni scritta L4 è un evento
`ACTION` con chiave payload `l4` (stringa) discriminante:
`"narrate" | "snapshot"`. Payload L0 flat; le liste rispettano F1 di
`spec/L1_SPEC_v0.3.md` (stringa contenente array JSON canonico
interno). Schema chiuso pattern L2.21/L3.1:
- `narrate` porta SOLO `at` (`i64`: seq dell'evento di valutazione
  generatore della revisione, anchoring L2.18), `text` (stringa: le
  affermazioni canoniche della revisione, separate dal carattere
  newline `\n`), `sources` (stringa JSON canonica array di `i64`,
  dedup crescente);
- `snapshot` porta SOLO `at` (`i64`, stesso valore del `narrate` a cui
  è appaiato) e `self_hash` (stringa hex minuscolo a 64).
Chiavi extra → `SelfError::Schema`; valore di `l4` fuori dominio →
`SelfError::Schema`. Un evento con `l4` e `l2` oppure `l3` assieme →
`SelfError::Schema` (domini chiusi e mutualmente esclusivi). Eventi
senza `l4` sono fuori da L4.

**L4.2 — Lettura delle fonti (Q8).** L4 è funzione del log soltanto:
- ricostruisce la vista L1 con `view::build` e verifica che
  `view.head_hash` coincida con l'hash dell'ultimo evento del log →
  altrimenti `SelfError::HeadMatch` (pattern L2.12/L3.9);
- NON ri-parsa i payload `l2`/`l3` per i loro oggetti interni: legge
  gli eventi di decisione come eventi del log per generare le
  affermazioni (L4.3);
- se la normativa usasse lo stato L2/L3, lo ottiene SOLO via API
  pubbliche vigenti (`build_active`, `build_memory`); in v0.1 NESSUNA
  chiamata a L2/L3 serve alla generazione: le affermazioni si
  ricavano dagli eventi stessi;
- un errore propagato da un'API usata si mappa: `*::HeadMatch` →
  `SelfError::HeadMatch`; ogni altro errore → `SelfError::Schema`.

**L4.3 — Rilevanza e affermazioni (Q5, firma per delega).** La
narrazione racconta solo le azioni costitutive nobili e le decisioni.
Elenco esclusivo di eventi rilevanti (in ordine di seq):
(a) evento L0 con estrazioni che dichiara per la PRIMA volta un
elemento (per l'elemento, `origin_seq` nella vista L1 = seq
dell'evento, L1 D18);
(b) evento che dichiara per la PRIMA volta un arco (per l'arco,
`origin_seq` nella vista L1 = seq dell'evento);
(c) evento con `l2` decisionale: `select` o `summary` (barrier e
config sono eventi di controllo: esclusi);
(d) evento con `l3` decisionale: `promote`, `archive` o `recall`
(config escluso).
NON sono rilevanti: eventi senza estrazioni non decisionali; eventi
con sole menzioni o riprove dell'arco (L2.27); eventi `l4` (sono il
veicolo della narrazione, non suo contenuto); gli eventi di controllo
L2/L3. Le affermazioni, a template vincolati e una per evento fonte,
in ordine di seq; per un evento con più dichiarazioni l'ordine
interno è lessicografico per nodo `(kind,key)` e per arco
`(rel,src_id,dst_id)`:
- nodo: `a seq {seq} ho dichiarato l'elemento {kind} "{key}"`
- arco: `a seq {seq} ho dichiarato l'arco {rel} da {kind_s} "{key_s}" a {kind_d} "{key_d}"`
- select: `a seq {seq} ho selezionato: entrate [{ups}], uscite [{downs}]`
- summary: `a seq {seq} ho consolidato: espulsi [{expulsions}]`
- promote: `a seq {seq} ho promosso l'identità {id}`
- archive: `a seq {seq} ho archiviato l'identità {id}`
- recall: `a seq {seq} ho richiamato l'identità {id} (origin_seq {origin})`
dove `{seq}` è la seq dell'evento che genera l'affermazione;
`{ups}`/`{downs}`/`{expulsions}` sono le origin_seq dei campi
dell'evento, ordine crescente, dedup, separati da virgola senza
spazi, vuoto reso come `[]`; `{id}` è l'id L1 completo (hex minuscolo
64); `{kind_s}/{key_s}` e `{kind_d}/{key_d}` sono sorgente e
destinazione dell'arco; `{origin}` è il campo `origin_seq` dell'evento
l3. Un evento rilevante produce una o più affermazioni (tutte le
dichiarazioni nodes/edges del suo payload); le affermazioni della
revisione si uniscono con `\n` nell'ordine sopra; un evento non
rilevante non genera nulla.
Esclusione dichiarata (da eventuali emendamenti futuri): la
narrazione NON ricalcola né ridecide la semantica L2/L3; dice che
eventi di decisione sono apposti in log e li cita come scritti; i
verdict L3 (E3 compreso) restano competenza della legge L3 e della
vista L3.

**L4.4 — Documento del sé (Q3).** Stessa regola di L3.8: documento
canonico, membri in ordine lessicografico, nessuno spazio,
`schema_version:"1"`:

```
{"head_hash":"…","schema_version":"1","self_hash":"…","statements":[…]}
```

- `statements`: array in ordine di `at` crescente; membro =
  `{"at":<i64>,"sources":[<i64>,…],"text":"<affermazioni \\n-congiunte>"}`,
  con chiavi figlie in ordine lessicografico;
- `head_hash` = hash dell'ultimo evento del log nel momento della
  ricostruzione (L4.8: a boot l'ultimo del log; nel controllo L4.7
  l'hash del narrate appaiato);
- `self_hash` = SHA-256 minuscolo del PREIMAGE: dello stesso documento
  canonico privo del campo `self_hash` (pattern D16 L1 / L3.8);
- stesso log → stesso documento byte per byte.

**L4.5 — Zero stocastica (Q6).** L4 non fa alcun draw dal PRNG, nessun
avanzamento di stato, nessuna fonte di casualità esterna; zero I/O,
zero wall-clock, zero RNG in `src/identity.rs`. Ogni stocastica futura
richiede un emendamento firmato di L0 §4 (budget dei draw) e di questa
legge: la parola «eventuali» della SPEC non è un'autorizzazione.

**L4.6 — Generazione (decisioni L4).** `evaluate_self` produce le
revisioni solo DOPO il cutoff. Cutoff: la seq dell'ultimo evento `l4`
del log (narrate o snapshot: entrambi decisionali; pattern L3.9). Per
ogni evento rilevante (L4.3) con seq > cutoff, in ordine di seq,
UNA coppia di decisioni:
1. `Decision4::Narrate { at: u64, tick: u64, text: String, sources: Vec<u64> }`
   — `at` = seq dell'evento rilevante; `tick` = tick dell'evento
   APPOSTO (pattern L2.18); `text` e `sources` da L4.3;
2. `Decision4::Snapshot { at: u64, tick: u64, self_hash: String }` —
   `at` e `tick` identici al narrate; `self_hash` = SHA-256 del
   preimage del documento cumulativo nell'istante: tutte le
   affermazioni generate fino a e inclusa questa revisione, con
   `head_hash` = hash del narrate di questa revisione.
Ordine in output: per ogni revisione prima Narrate, poi Snapshot;
revisioni in ordine di `at` crescente. Encode L0 (utility):

```
l4: "narrate";  at: i64; text: String; sources: "[i64,…]" (stringa JSON canonica)
l4: "snapshot"; at: i64; self_hash: String
```

**Precisazione su `sources` (firmata dall'orchestratore, 31/10/2026,
sotto delega):** `sources` della revisione = l'unione, dedup, in ordine
crescente delle seq CITATE nei template delle affermazioni della
revisione: la seq dell'evento che genera ciascuna affermazione più le
seq originarie che compaiono nei rimpiazzi `{ups}`, `{downs}`,
`{expulsions}`, `{origin}`. Il `{seq}` del template dà l'ancora;
la bibliografia raccoglie tutte le citazioni di seq della revisione.

Nessuna append diretta: l'append resta al chiamante via API L0
(L2.20/L3.10). Eventi non rilevanti e di controllo: nessuna decisione.

**L4.7 — Replay autorevole e verifiche.** `build_self(log)`:
- verifica L4.2 (testa); poi iterazione a seq crescente su TUTTI gli
  eventi del log;
- evento `narrate` in log: AUTOREVOLE (pattern L2.11: il testo apposto
  non si ri-narra e non si ri-emette); le sue affermazioni alimentano
  lo stato interno così com'è scritto; vincoli validati: `at` deve
  essere la seq di un evento esistente NON di un evento `l4`
  (autoreferenzialità vietata) e deve riferirsi a un evento storicamente
  ANTERIORE al narrate stesso (`at` < seq del narrate) → violazioni →
  `SelfError::Schema`;
- evento `snapshot` in log: deve essere APPAIATO — stesso `at` del
  narrate IMMEDIATAMENTE precedente nel log; un snapshot non preceduto
  dal proprio narrate (coppia rotta in mezzo al log; due snapshot di
  fila; narrate solitario CHIUSA da un evento successivo non appaiato)
  → `SelfError::Schema`; deve inoltre valere
  `snapshot.self_hash == SHA-256 del preimage del documento cumulativo
  in quell'istante` (tutte le affermazioni narrate fino a e inclusa la
  coppia, con `head_hash` = hash del narrate) → mismatch →
  `SelfError::Schema` (l'ancora d'integrità è dura);
- un narrate solitario come ULTIMO evento del log è tollerato:
  nessun errore, nessuna scrittura (append incompleta, L4.9);
- La nota L2.11 si applica: un narrate il cui testo NON coincide col
  ricalcolo di L4.3 non è errore a runtime (il testo apposto è
  autorevole); il ricalcolo divergente è un MISMATCH DICHIARATO; la
  determinismo della STESURA è verificata dai test di generazione
  (checklist 7), e l'ancora `self_hash` degli snapshot resta
  il presidio di integrità (verifica dura di L4.7);
- errore dentro un evento (chiavi extra, dominio invalido, lista non
  canonica, campo mancante) → `SelfError::Schema` (pattern L2.21).

**L4.8 — Boot, API pubblica (Q10).** Boot = ogni chiamata pubblica L4
su `&Log`; «ripresa sessione» = ricalcolo puro dal log, nessun file
«chi sono io»; log non integro → errore esplicito, mai fallback.
Solo cose pubbliche:
- `pub struct SelfDocument { head_hash: String, statements: Vec<Statement>, self_hash: String }`,
  `pub struct Statement { at: u64, sources: Vec<u64>, text: String }`;
- `pub enum SelfError { Schema, HeadMatch }`;
- `pub struct SelfBase { head_hash: String, log_len: usize }` (base
  con coerenza di prefisso stile L2.22);
- `pub fn build_self(log: &Log) -> Result<SelfDocument, SelfError>` —
  replay autorevole a fine log (L4.7);
- `pub fn evaluate_self(log: &Log, base: &SelfBase) -> Result<Vec<Decision4>, SelfError>`
  — coerenza di prefisso (log ≥ log_len; l'hash dell'evento in
  posizione `log_len−1` deve coincidere con `base.head_hash`, altrimenti
  `SelfError::HeadMatch`), decisioni per eventi con seq > cutoff della
  coda (L4.6), ordine L4.6; nessuna decisione per eventi nella base;
- `pub enum Decision4 { Narrate { at: u64, tick: u64, text: String, sources: Vec<u64> }, Snapshot { at: u64, tick: u64, self_hash: String } }`;
- `pub fn encode4(d: &Decision4) -> Vec<(String, Value)>` — payload L0
  secondo L4.6 (liste come `Value::Str` con JSON canonico, `at` come
  `Value::Int`);
Append via API L0 al chiamante. Zero I/O, zero wall-clock, zero RNG.

**L4.9 — Interazione con L2/L3 (Q11).** Scritte esplicitamente, come
L3.1 secondo capoverso:
- gli eventi `l4` sono eventi L0 non-`l2`: chiudono IMPLICITAMENTE una
  finestra L2 aperta (L2.25); sono privi di estrazioni, quindi non
  sono eventi di valutazione L2/L3 (L2.7); non entrano nei cutoff L2
  (L2.23) né L3 (L3.9); L2 e L3 non leggono `l4` (domini chiusi);
- il cutoff L4 è definito in L4.6 ed è a suo dominio; l'ordine col
  `Select` di chiusura di una finestra segue l'ordine di seq del log
  senza regole speciali: gli eventi L4 vengono apposti dopo le
  decisioni che raccontano;
- append corretto: SEMPRE la coppia narrate+snapshot in ordine, per
  ogni revisione, senza intermissioni (l'eventuale narrate solitario
  a fine log è l'unico caso tollerato, L4.7).

**L4.10 — Nulla di persistente.** Nessun file, nessun I/O, nessun
elemento persistente: il sé vive solo nel log e nel documento
ricostruito al volo («Nessun file separato "chi sono io"» della SPEC
rispettata).

---

## Checklist di uscita (gate tester)

1. `build_self` su log di sola genesi: `statements` vuoto,
   `head_hash` = hash genesi, `self_hash` coerente col preimage.
2. **Divergenza scoped (Q9 firmata):** (i) log che differiscono solo
   per eventi irrilevanti (riprove dell'arco, eventi senza estrazioni
   non rilevanti) → documento IDENTICO byte per byte; (ii) log che
   differiscono su eventi rilevanti (prima dichiarazione, decisione
   l2/l3) → documento DIVERSO. NESSUN test che assume iniettività
   universale «due log diversi → due identità diverse» (falsa in
   generale, per riformulazione firmata).
3. **Tutti i template L4.3** unit-testati: nodo, arco, select,
   summary, promote, archive, recall, più l'ordine interno, la
   congiunzione `\n` e la lista `sources`.
4. Replay completo: documento ricostruito nella stessa forma del
   documento già emesso; ogni snapshot verificato; log corrotto (mismatch
   hash, coppia rotta, `at` impossibile) → `SelfError::…` esplicito,
   nessun panic, nessun UB.
5. Cutoff: append dopo revisioni → decisioni SOLO per eventi nuovi;
   il prefisso narrato non produce nulla.
6. Interazione L2/L3 (L4.9): evento `l4` chiude la finestra L2;
   nessun side-effect su L2/L3 né sui loro cutoff.
7. Determinismo: due generazioni dal medesimo log → decisioni
   identiche byte per byte.
8. Zero draw: le API L4 non mutano lo stato del PRNG; tutta la suite
   L0–L3 resta VERDE (122 test pre-esistenti).

## Rischi aperti da segnalare, non decisioni nuove

- la tensione «stima di costi del testo» e la crescita del `text`
  delle revisioni non è un vincolo giuridico in v0.1;
- L5 (futura) legge «viste L1–L4»: la forma canonica L4.4 è quindi già
  contratto per L5 — eventuali sue revisioni vanno firmate prima di L5.

## Firma

- [x] **orchestratore, per delega umana esplicita** — 31 ottobre 2026:
  firma («scegli tu») sul pacchetto Q1–Q11 del triage
  (`notes/L4_domande_identita.md`); tutti i default del triage;
  scelte puntuali firmate: Q5 rilevanza = prime dichiarazioni +
  decisioni L2/L3; Q9 divergenza scoped; Q10 boot = chiamata pubblica
  con `SelfError { Schema, HeadMatch }`; Q7 nomenclatura («sé» per
  l'oggetto L4, «identità/id» resta dell'id L1). La ratifica umana
  arriva con l'accettazione di fine ciclo.
- [ ] **umano** — accettazione fine ciclo nel consueto percorso
  (test-first → gate tester → audit reviewer → accettazione, come
  L2/L3).
