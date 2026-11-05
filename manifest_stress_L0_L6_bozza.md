# Manifest dello stress test L0–L6 — bozza non eseguibile

**Stato: BOZZA, non approvata come manifest definitivo. Nessuna esecuzione
autorizzata da questo documento.** Gli elementi indicati come `TBD` devono
essere completati e preregistrati prima della firma. Le scelte etichettate
«proposta» non sono legge vigente. Questo testo non modifica le SPEC o le
leggi firmate. Per le risposte ricevute e i conflitti si veda
`bozza_emendamento.md`.

## Provenienza e ambiente

| Voce | Valore / stato |
| --- | --- |
| Commit sorgente | **TBD** — commit esatto da fissare dopo l'emendamento approvato |
| `scenario_id` | **TBD** — valore, codifica e collocazione |
| Runtime | Docker; nessun fallback host |
| Immagine | `rust:1.85.1@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4` |
| Dipendenze | `Cargo.lock` del commit fissato; nessuna modifica al progetto durante la prova |
| Integrità manifest | **TBD** — formato canonico, momento del congelamento e hash SHA-256 |
| Artefatti | Manifest, oracoli, log e relativi hash; percorsi e schema **TBD** |

Il manifest definitivo e gli oracoli devono essere congelati **prima**
dell'avvio della prova. L'esecuzione avviene solo dopo l'approvazione del
testo, del manifest e degli oracoli. Non usare valori ricavati dalla run
come oracoli della stessa run.

## Log e identificativi

- Log principale L0: seed `1042`; genesi senza draw. Tick, ordine e
  contenuto esatti: **TBD**. Un ciclo L0 valido per tick, secondo il regime
  che sarà approvato per la collocazione degli eventi di livello.
- Per ciascuna coppia L6: seed separato nell'intervallo `1000..=1099`, un
  log memoria e un log controllo. I task L6 restano distinti dai cicli L0
  del log principale.
- `scenario_id`, ID di run, ID di interazione e collegamenti tra artefatti:
  **TBD**. Non introdurre chiavi nei payload L0/L2–L5 a schema chiuso senza
  autorizzazione normativa: valutare la registrazione nei soli metadati
  esterni.

## Dati L1 — proposta da approvare

Quattro nodi con `kind="elemento"`, chiavi `alfa`, `beta`, `gamma`, `delta`;
un arco `rel="conosce"` da `alfa` a `beta`, entrambi endpoint di kind
`elemento`. `rel` rispetta il dominio descritto da L1 §3; nessun identificativo
hash è assunto in anticipo. L'ordine delle dichiarazioni, il payload F1
canonico preciso, eventuali ulteriori estrazioni e gli eventi ospiti sono
**TBD**. In particolare, lo stub del ciclo L0 e la collocazione delle
estrazioni devono essere risolti prima di dichiarare il manifest coerente.

## Configurazioni e piano ordinato — incompleto

Valori comunicati: L2 `k=3`, `tau=5` in **seq**, una configurazione prima
delle selezioni; L3 `T=1`, `H=10` tick prima delle transizioni.

La tabella seguente è un **ordine logico proposto**, non la sequenza
eseguibile richiesta: non definisce né tick né `seq` effettivi. Gli eventi
decisionali e narrativi aggiungono seq e non possono essere conteggiati
retroattivamente come se fossero assenti.

| Passo | Operazione o evento da preregistrare | Tick / seq / payload esatto |
| --- | --- | --- |
| M0 | Genesi e verifica iniziale | tick 0, seq 0; payload da L0 §4 |
| M1 | Configurazioni `l2=config` e `l3=config` | **TBD**; ordinamento e collocazione tra cicli **TBD** |
| M2 | Dichiarazioni L1 e prima attivazione dell'identità bersaglio | **TBD** |
| M3 | Attivazione della stessa identità in un evento distinto; giudizio per `promote` | **TBD**; decisioni L2/L3 generate e apposte **TBD** |
| M4 | Apertura `barrier`, select in finestra con espulsione, chiusura e `summary` | **TBD**; ordine e condizioni conformi a L2.25 **TBD** |
| M5 | Giudizio successivo al decadimento richiesto per `archive` | **TBD**; tick e stato attivazioni sufficienti a score zero **TBD** |
| M6 | Nuova attivazione dopo la seq di `archive`, poi `recall` | **TBD**; `at`, `origin_seq`, id e tick **TBD** |
| M7 | Revisioni L4 dopo promote, archive e recall | **TBD**; cutoff, `at`, sources, testo e hash **TBD** |
| M8 | Osservazioni, previsioni, accordo/disaccordo e `model_update` L5 | **TBD**; vedere sequenza proposta sotto |
| M9 | Snapshot diagnostico esterno dopo aggiornamento L5 | **TBD**; non è evento o narrazione L4 |

L'ordine delle revisioni M7 rispetto a M3/M5/M6 è **TBD**: la riga M7
identifica la copertura, non autorizza a rimandare tutte le revisioni dopo
il recall. Analogamente, non è dimostrato che i quattro nodi da soli
producano il `summary`: occorre preregistrare select, espulsioni e stato
della finestra secondo la legge.

## L5 — sequenza proposta da completare

1. Osservazione `agente_alpha`, comportamento `1`.
2. Previsione per alpha, apposta secondo le regole L5.
3. Osservazione alpha `1` per verificare l'accordo.
4. Osservazione `agente_beta`, comportamento `2`.
5. Nuova previsione per alpha.
6. Osservazione alpha `2` per verificare il disaccordo e il `model_update`.

Questa è una proposta di ordine; ogni evento `AUX`, array canonico,
`at`, `sources`, tick, seq e base usata per valutare sono **TBD** fino alla
compilazione completa. La coppia di agenti e il confronto per il secondo
ordine sono **TBD**, come l'eventuale prova esplicita che il caso sia
conforme a L5.8. Non si considera il piano eseguibile sulla base di questa
lista.

## L6 — corpus e collegamento

Per la valutazione, la legge L6 descrive 100 coppie appaiate, 120 task per
run (80+40), policy memoria/controllo, e timeout di 5 secondi per coppia.
Nel piano richiesto i log dei bracci rimangono separati. Collegamenti,
configurazione del timeout realmente interruttivo, e procedura indipendente
di replay delle azioni e reward sono **TBD**. Nessuna conclusione di
efficacia statistica è attribuita allo stress test.

Per ogni coppia è richiesta una riga JSONL esterna con esiti, delta,
validità e hash artefatti; nomi esatti dei campi, canonizzazione, indicazione
esplicita di errore e organizzazione nel log unico sono **TBD**.

## Fixture del gate — proposta

- **Positiva:** manifest e fixture appaiata validi; accettazione prima della
  valutazione. Contenuto byte-esatto e output atteso: **TBD**.
- **Negativa:** copia della fixture positiva con un evento alterato senza
  aggiornare l'hash; rifiuto per integrità prima della valutazione. Evento,
  byte modificati e motivo codificato: **TBD**.

Il rifiuto previsto dà PASS al gate negativo e non conta come run invalida;
la sua accettazione dà FAIL. Una run prevista dal protocollo che viola un
prerequisito è invece invalida e rende non superato il test complessivo.

## Punto normativo da approvare prima della versione eseguibile

L0 §5 prescrive un ciclo `SENSE → ACTION → STATE`; le leggi vigenti usano
`ACTION` per decisioni L2–L5 e `SENSE` per osservazioni L5. La bozza
coordinata §§13.1–13.8 propone invece AUX tra cicli e una fixture esterna
per L5: è una proposta da ratificare, non legge vigente o funzionalità già
implementata. Restano da completare la specifica delle estrazioni e la
sequenza effettiva, senza interrompere le triplette L0.

## Fixture L5 candidata per il gate di integrità

Questo esempio concreto non sostituisce le fixture dello scenario completo
né è approvato per l'esecuzione. `fixture_id` proposto: `l5-genesis-01`;
ancora prevista: seq 0 GENESIS, tick 0; apposizione proposta dopo genesi e
prima del primo SENSE, a finestra L2 chiusa. La fixture è costituita dai
seguenti **136 byte UTF-8**, senza BOM né newline finale:

```json
{"observations":[{"agente":"agente_alpha","confidenza":1000,"ordine_tom":0,"proposizione":"1","soggetto":"agente_alpha","sources":[0]}]}
```

Il digest SHA-256 candidato dei soli byte della fixture è
`c66da3195548417fcbcaea078e8bab66b98b539a68ddde6011223800c117fbac`.
Nel manifest definitivo associare al suo ID un percorso relativo fissato,
l'ancora e questo digest; `data` dell'AUX deve coincidere con quei byte e
`provenance.external` deve riportare ID e digest. Verificare anche che la
seq 0 sia effettivamente la genesi del log considerato e che l'AUX sia nella
fase e al tick ammessi.

Per la copia negativa cambiare il byte all'offset **91**, indicizzato da zero,
da ASCII `1` a ASCII `2`, nel valore di `proposizione`, senza aggiornare il
digest preregistrato. La copia resta JSON canonico ma non corrisponde al
digest: `digest_mismatch` prima di valutare è PASS del gate, non una run
invalida. Per questo caso il digest della copia alterata, utile solo a
verificare la preparazione, è
`7d4d7aa8059739b6b4c09c15a565254a05b208e662f51b3ee31f81307688f099`.
Nessuna valutazione deve partire nel caso negativo. Questi digest non sono
hash di eventi o risultati della prova; la fixture positiva richiede anche
tutti gli altri controlli di schema, ancoraggio e corrispondenza con l'AUX.

## Prefisso L5 candidato, con seq esplicite

Questo è un **prefisso di progetto**, non l'intera sequenza eseguibile: vale
solo se l'emendamento v2 viene ratificato e se non si inserisce nessun altro
evento prima della seq 9. Tutti gli AUX sotto hanno tick 0 e `sources=[0]`;
la finestra L2 è chiusa. La seq 0 è GENESIS, senza draw. Le fixture usano
`sources=[0]` come ancora, non come fonte del contenuto; ogni osservazione
ha `members=[]`, `refs=[]` e `external` con ID e digest della sua fixture.

| seq | Evento proposto | Contenuto rilevante / `provenance.refs` |
| --- | --- | --- |
| 1 | L5/observation | fixture `l5-genesis-01`; `[]` |
| 2 | L5/predict alpha | `at=1`, `previsto=1`, `data.sources=[1]`; `[1]` |
| 3 | L5/observation | ancora fixture `l5-genesis-01`; `[]` — chiude in accordo la finestra del predict 2 |
| 4 | L5/observation | fixture `l5-beta-01`; `[]` |
| 5 | L5/predict alpha | `at=3`, `previsto=1`, `data.sources=[1,3]`; `[1,3]` |
| 6 | L5/observation | fixture `l5-alpha-02`; `[]` — chiude in disaccordo la finestra del predict 5 |
| 7 | L5/model_update alpha | `at=6`, `previsto=1`, `osservato=2`, `data.sources=[5,6]`; `[5,6]` |
| 8 | L5/observation | fixture `l5-perspective-01`; `[]`, ordine 1 |

Gli ID alpha e beta sono quelli candidati nella tabella degli oracoli. In
ogni decisione `id` è l'ID alpha; `members=[]`. `data.sources` cita AUX,
mentre `sources` esterno è l'ancora L0. L'ordine 1 è un dato dichiarato,
non un accesso alla mente di beta e non aggiorna contatori di ordine 0.
Questo prefisso non esaurisce la prova di composizione di L5.8: il relativo
contratto v2 e il confronto byte per byte rimangono da specificare.

Oltre alla fixture `l5-genesis-01` sopra, le fixture candidate sono gli
oggetti qui sotto, ciascuno codificato esattamente come riga UTF-8 senza
newline finale; le righe Markdown non fanno parte dei byte. Il manifest
definitivo deve assegnare un percorso relativo distinto a ciascun ID.

| ID | Byte | SHA-256 |
| --- | ---: | --- |
| `l5-beta-01` | 134 | `b0a0fdb92c0dfbe7bfaf9cd56bbbd78e4cfef58378aeacc4037036a975fa77c5` |
| `l5-alpha-02` | 136 | `7d4d7aa8059739b6b4c09c15a565254a05b208e662f51b3ee31f81307688f099` |
| `l5-perspective-01` | 135 | `9a5b3c1cd79054c46f205684e034fc3c96248cf8dc58843f7fd793c76386cc79` |

`l5-beta-01`:
```json
{"observations":[{"agente":"agente_beta","confidenza":1000,"ordine_tom":0,"proposizione":"2","soggetto":"agente_beta","sources":[0]}]}
```

`l5-alpha-02`:
```json
{"observations":[{"agente":"agente_alpha","confidenza":1000,"ordine_tom":0,"proposizione":"2","soggetto":"agente_alpha","sources":[0]}]}
```

`l5-perspective-01`:
```json
{"observations":[{"agente":"agente_alpha","confidenza":1000,"ordine_tom":1,"proposizione":"2","soggetto":"agente_beta","sources":[0]}]}
```

I byte di `l5-alpha-02` coincidono intenzionalmente con la copia alterata
del gate negativo precedente: sono validi **solo** se associati all'ID e
al digest propri, non quando sostituiscono `l5-genesis-01` mantenendone il
digest. Nessuno di questi hash è l'hash di un evento del log.

## Piano L6: formato e generazione candidati

Per ogni seed `1000..=1099`, il piano candidato è JSON canonico UTF-8,
senza BOM né newline finale, con chiavi esatte `rule`, `seed`, `tasks`.
`rule` è array di 12 interi per cue `cue:00`…`cue:11`; `tasks` è array
ordinato di 120 oggetti esatti `{"cue":"cue:NN","giusta":0}` con
`giusta` sostituito dalla classe effettiva `0..=2`. Il manifest definitivo
deve identificare posizione e digest di ogni piano. Lo schema è una
proposta di artefatto esterno: non modifica i payload L6.

Il generatore indipendente candidato segue `spec/L0_SPEC_v0.4.md` §8 per
seeding e draw PCG64 e la procedura del piano in `src/valuation.rs`:

1. Inizializzare `rule` a quattro 0, quattro 1 e quattro 2. Per `i` da 11
   a 1, estrarre un `u64`, porre `j=draw mod (i+1)` e scambiare `rule[i]`
   con `rule[j]`.
2. Generare 80 indici di apprendimento, ciascuno `draw mod 12`.
3. Appendere tre volte gli indici `0..=11` in ordine, quindi quattro indici
   ciascuno `draw mod 12`. Per ogni indice `i`, il task contiene il cue
   `cue:NN` a due cifre e `giusta=rule[i]`.

Sono 95 draw offline per seed dopo l'inizializzazione: 11 + 80 + 4. La
generazione non consuma draw nei log dei bracci né nel ciclo principale.
Il piano viene generato e verificato indipendentemente **prima** della run;
confrontare ogni task con entrambi i log. Il numero e l'algoritmo dei draw
descrivono questo generatore candidato, non provano da soli l'indipendenza
del verificatore o la correttezza dei file. I piani candidati sono in
`preparazione_stress/candidati_l6/piani.jsonl`; i digest individuali sono
in `oracoli.jsonl` nella stessa directory e coprono ogni riga senza LF.
Restano da revisionare indipendentemente e approvare prima della prova.

## Trasformazione sintetica L0 → L1 candidata

Si propone un interprete deterministico, non una percezione del mondo: il
suo input è l'evento `SENSE` del ciclo principale, con `tick` e payload
`{"stimolo":"<u64 decimale canonico>"}`. Il gate verifica forma, esistenza
nel log e posizione nel ciclo completo; lo stimolo viene validato ma la
tabella usa soltanto il tick. Pertanto i nomi di nodi e archi sono **scelte
sintetiche del protocollo**, non significato intrinseco del numero PRNG.

| Tick del SENSE | `nodes` in ordine | `edges` |
| ---: | --- | --- |
| 1 | `elemento/alfa`, `elemento/beta`, `elemento/gamma`, `elemento/delta` | `alfa → beta`, `rel=conosce`, endpoint kind `elemento` |
| 2 | `elemento/alfa` | `[]` |
| 253 | `elemento/beta` | `[]` |
| 254 | `elemento/alfa` | `[]` |
| altri | `[]` | `[]` |

Ogni voce nodo è esattamente `{"key":"…","kind":"elemento"}`; l'arco è
`{"dst_key":"beta","dst_kind":"elemento","rel":"conosce","src_key":"alfa","src_kind":"elemento"}`.
L'array mantiene l'ordine della tabella, non ordina i nodi per ID. Per ogni
tick elencato il chiamante propone un `L1/extract` dopo lo STATE dello stesso
ciclo, con `sources` uguale all'array singleton della seq del SENSE del ciclo;
ogni membro in `provenance.members` ha la stessa fonte. Negli altri tick
non appone estrazioni. La tabella e la relazione input/output devono essere
incluse nel manifest finale e verificate prima delle valutazioni.

Con `H=10`, al giudizio tick 253 le attivazioni di alfa a tick 1 e 2 hanno
entrambe età di almeno 250 tick e contributo L3 zero; il tick 253 fornisce
un evento di valutazione distinto. L'estrazione di tick 254 fornisce una
sequela successiva all'archive **solo se** l'archive è stato effettivamente
apposto fra i due cicli: il manifest deve fissarne seq e ordine. La tabella
non dimostra da sola che selezioni, archive, recall e narrazioni siano
validi: mancano ancora il piano di apposizione completo e la risoluzione
dei possibili conflitti tra finestra L2 e decisioni degli altri livelli.
