# Bozza coordinata L0–L6: eventi AUX e alternativa esterna

**Stato: BOZZA NON FIRMATA.** Questo documento raccomanda una strada e
formula scelte proposte, non le dichiara vigenti. Non modifica le leggi, le
SPEC, il codice, i test o gli artefatti storici. Nessun test, Docker o
implementazione è autorizzato dalla presente bozza. Il manifest e gli
oracoli restano da completare e approvare separatamente.

**Base di lettura:** revisione Git `464a32a` e bozze locali
`bozza_emendamento.md`, `manifest_stress_L0_L6_bozza.md`,
`oracoli_stress_L0_L6_bozza.md`; le bozze locali non sono fonti normative.
Prima di un'eventuale implementazione va fissato il commit esatto successivo
all'approvazione dell'emendamento.

## 1. Confronto delle strade

| Aspetto | A. `AUX` nella catena L0 | B. Record esterni al log L0 |
| --- | --- | --- |
| Identità e ordine | Una seq globale e un hash concatenato per ogni AUX | Serve un proprio ordine, collegamento al log L0 e regola di integrità |
| Uso da L1–L5 | Richiede emendamenti e modifiche ai lettori perché riconoscano AUX | I lettori attuali non leggono automaticamente record esterni: serve un adattatore o API nuove |
| Stato L0 e ciclo | Richiede quarto tipo e nuove regole di fold/validazione | Non richiede un quarto tipo L0 se i record non vengono apposti nel log |
| Compatibilità storica | Nuovi lettori devono continuare a leggere e verificare i vecchi log; vecchi lettori non riconoscono AUX | Il formato L0 vecchio resta invariato; la nuova evidenza dipende anche dall'integrità del registro esterno |
| Provenienza | La seq AUX e le seq sorgenti possono coesistere nella stessa catena | Il legame fra registri deve essere autenticato e verificato separatamente; non basta uno `scenario_id` |
| Modifiche e rischio | Ampie: L0, L1–L5, parser, replay e API | L0 invariato, ma servono specifica del registro, driver/adattatore e verifica di ciò che effettivamente entra nelle viste |
| Che cosa prova | Integrazione nel medesimo log esteso, se implementata e verificata | Integrazione mediante una frontiera esterna esplicita; non prova da sola che le viste attuali leggano quei record |

### Scelta A — proposta da incorporare

Si raccomanda **A: AUX nella catena L0**, perché l'obiettivo espresso è
assegnare una seq globale agli eventi ausiliari e collegare decisioni e
provenienza al medesimo log principale. La strada B è meno invasiva per L0
ed è appropriata se si preferisce una prova mediata da registri esterni, ma
richiederebbe comunque un ponte per L1–L5 e non soddisfa automaticamente il
requisito di una catena principale comprendente gli ausiliari.

kingspina ha indicato di scegliere **A** e di integrare tale scelta come
proposta. La scelta è registrata nel testo di lavoro, **non come firma
dell'emendamento**: l'approvazione formale resta necessaria. Gli articoli
seguenti descrivono A e non costituiscono l'emendamento della strada B.

## 2. L0 — formato, ciclo e stato: proposta per A

### 2.1 Versione estesa

Propongo un formato di protocollo **v2**, con tipi top-level
`SENSE | ACTION | STATE | AUX`. I campi top-level dell'evento restano
`seq,tick,type,payload,prev_hash,hash`, nello stesso ordine; la preimage
resta quella di L0 §2, con stringa di tipo `AUX`. `seq` è globale e
contigua; `Log::append` la assegna come per gli eventi precedenti.

`AUX` è ammesso soltanto fra uno `STATE` di **ciclo completato** e il
`SENSE` del ciclo successivo; più AUX consecutivi nello stesso intervallo
sono ammessi nell'ordine del manifest. Il tick AUX è quello dell'ultimo
STATE di ciclo completato. Il successivo ciclo mantiene la tripletta
contigua `SENSE → ACTION → STATE`; genesi non consuma draw e ogni ciclo
consuma un draw al SENSE. La validazione dello stato di fase deve rifiutare
un AUX dentro la tripletta; la sola verifica degli hash non basta.

**Caso iniziale:** la genesi è `STATE` ma non è uno `STATE` di ciclo
completato. Se le configurazioni debbano poter essere apposte subito dopo
genesi o soltanto dopo il primo ciclo resta **FIRMA S2**, non è deciso qui.
La definizione deve includere il trattamento degli STATE compensativi.

Un AUX non incrementa i contatori L0 `sense`/`action`, non avanza tick o
PRNG e non muta lo snapshot PRNG. **Modifica sempre la testa hash** e la
lunghezza del log. Lo `STATE` successivo conserva contatori coerenti con
queste regole. La fold aggiorna `head` anche sugli AUX, ma non deve tentare
di leggere uno snapshot `state` da essi.

Queste sono nuove regole: richiedono modifiche a enum, parser, serializzatore,
verifica di fase, `run_cycle`, fold e API; `Log::append` attuale non le
garantisce solo per il fatto di assegnare una seq.

### 2.2 Schema AUX proposto

I campi `seq`, `tick`, `type`, `prev_hash`, `hash` restano **top-level L0**;
non sono duplicati nel payload. Il payload L0 rimane piatto, con esattamente
queste chiavi, in ordine canonico lessicografico:

```text
data        : stringa contenente JSON canonico
kind        : stringa discriminante
level       : stringa discriminante
provenance  : stringa contenente JSON canonico
sources     : stringa contenente array JSON canonico di seq
```

Proposta: `sources` è un array di interi non negativi, deduplicati e in ordine
crescente, ciascuno riferito a un evento **già presente** (`seq < seq AUX`);
`[]` è ammesso soltanto dove lo schema del tipo lo consente. `provenance`
ha uno schema chiuso per coppia `level/kind`, senza fonti temporali future;
il suo contenuto, inclusa l'eventuale ridondanza con `sources`, deve essere
definito nel catalogo normativo, non lasciato come JSON arbitrario. `data` ha
anch'esso schema chiuso per coppia `level/kind`. `tick` e `seq` non vengono
copiati nel payload salvo campi semantici specifici come `at` e
`origin_seq`. Range, UTF-8, escaping e canonicità seguono le regole
esplicitamente adottate dal catalogo.

Il catalogo sottostante è una **proposta di domini**, non sostituisce le
definizioni complete dei singoli schemi. Eventi con combinazione sconosciuta,
campi extra, stringhe JSON non canoniche, riferimenti assenti/futuri o tipo
errato sono rifiutati con errore esplicito; mapping degli errori per API:
**FIRMA S3**. Lo schema dettagliato di `provenance`, la canonizzazione
formale di `data` e i casi in cui le fonti vuote siano valide sono
**FIRMA S4** prima dell'implementazione.

| `level` | `kind` proposti | `data` da definire con schema chiuso |
| --- | --- | --- |
| `L1` | `extract` | `nodes`, `edges` con membri F1 e ordinamento L1 |
| `L2` | `config`, `select`, `summary`, `barrier` | campi corrispondenti a L2.1/L2.21, escluso il vecchio discriminante top-level `l2`; la corrispondenza con `L2_SUMMARY` è da fissare in S4/S6 |
| **TBD** | `L2_WINDOW_CLOSE` | chiusura esplicita proposta, classificata non-L2 per la semantica della finestra; `level`, `kind` wire e campi esatti da firmare in S4/S6 |
| `L3` | `config`, `promote`, `archive`, `recall` | campi corrispondenti a L3.1–L3.7, escluso `l3` |
| `L4` | `narrate`, `snapshot` | campi corrispondenti a L4.1, escluso `l4` |
| `L5` | `observation`, `predict`, `model_update` | osservazioni o campi decisionali L5.2/L5.10, escluso `l5` |

Nessun AUX L6 è proposto. Gli esempi non sono payload definitivi e non
stabiliscono nuovi nomi di campo interni oltre quelli da approvare in S4.

## 3. L1–L3 — estrazioni, identità e attivazioni

### 3.1 `origin_seq` e provenienza: proposta da firmare

Per il formato v2 propongo:

- **`event_seq`** è la seq dell'AUX `L1/extract`: identifica quando la
  dichiarazione viene registrata nel log principale.
- **`origin_seq` L1** rimane la minima **seq dell'evento che dichiara** il
  nodo o arco, quindi la prima seq AUX `L1/extract` che lo contiene. Questo
  preserva il significato di dichiarazione in L1 D18, ma estende i tipi di
  evento che possono contenerla.
- **`source_seq`/`sources`** identifica separatamente le seq degli eventi
  L0 precedenti da cui il driver dichiara di aver derivato l'estrazione.
  Si registra e valida l'esistenza della fonte, senza affermare che la
  catena hash provi la correttezza semantica dell'estrazione.
- Riprova e attivazione L2/L3 si contano alla seq del **singolo AUX
  `L1/extract`**, una per identità per seq, come estensione di L2.27; più
  estrazioni riferite alla stessa fonte L0 restano eventi distinti se
  apposte in AUX distinti. Il conteggio L3 segue la stessa convenzione.
- I campi `at` di decisioni e l'ordine dei cutoff usano seq globali reali;
  dove la decisione deriva da un'estrazione, `at` punta all'AUX
  `L1/extract` che ha attivato la valutazione. Le fonti L0 restano
  consultabili tramite la provenienza dell'estrazione.

**FIRMA S5:** confermare questa scelta oppure specificare che l'origine o le
attivazioni usino le seq sorgenti L0, incluse conseguenze per L2.27, L3 e
la deduplicazione. La scelta alternativa non è equivalente e non va
applicata automaticamente. Va inoltre specificato se `sources` vuoto sia
valido per `L1/extract` e quali tipi di fonte siano ammessi (**S4**).

### 3.2 L2 — finestra offline: proposta da firmare

kingspina propone la seguente regola precisa, che sostituisce nella bozza
la precedente proposta di chiusura con `L2/barrier phase="close"`:

1. La finestra si apre e i `select` ammessi registrano le espulsioni;
   l'ordine concreto è fissato nel manifest.
2. Tra due cicli L0 completi il driver appone un AUX denominato
   **`L2_WINDOW_CLOSE`**, classificato **non-L2 ai fini della finestra**.
   È l'eccezione esplicita al rifiuto degli altri eventi non-L2 prima della
   chiusura: chiude la finestra e congela le espulsioni già registrate.
3. **Subito dopo, senza eventi intermedi**, il driver appone un AUX
   **`L2_SUMMARY`** con le espulsioni congelate e le rispettive seq sorgenti.
   I due AUX hanno il tick dell'ultimo STATE completato e seq globali
   consecutive; non interrompono la tripletta L0 e non avanzano il tick.
4. Un evento non-L2 **diverso dal `L2_WINDOW_CLOSE` autorizzato** prima
   della chiusura è rifiutato. Il record di chiusura è quindi non-L2 ma
   esplicitamente ammesso per compiere la chiusura, non un evento da
   respingere in base alla regola precedente.

Questa è una **proposta di modifica di L2.25 per v2**, non la regola vigente
per i log storici. Non implica una modifica retroattiva ai report o ai log
v1. Restano da definire e approvare: codifica esatta `level`/`kind` dei due
AUX e loro schema chiuso; significato di «seq sorgenti» delle espulsioni;
eventi L2 ammessi nella finestra; trattamento di log interrotto o terminato
tra chiusura e summary; errori specifici. La proposta dice di apporre sempre
un summary subito dopo la chiusura: va chiarito come si coordini con L2.15,
che non prevede summary per una finestra senza espulsioni. Nessuna delle due
interpretazioni è scelta in questa bozza (S3–S4 e dettaglio S6).

**S6 — proposta ricevuta, non firma formale:** la coppia `L2_WINDOW_CLOSE`
→ `L2_SUMMARY` è riportata sopra; il testo normativo completo e i casi limite
richiedono la firma di kingspina. L'agente non firma. Manifest e oracoli
restano incompleti; nessun ordine eseguibile è dichiarato pronto.

### 3.3 L3

Gli AUX L3 trasportano config, promote, archive e recall con i campi e le
transizioni L3 già previsti; `T=1`, `H=10` tick sono valori richiesti per lo
scenario, non nuovi default generali. S5 determina quali eventi contano
come attivazioni. E1/E2/E3 restano esplicitamente applicabili, salvo diversa
firma umana. Un recall richiede una nuova attivazione successiva alla seq
dell'archive; il suo `at` deve identificare l'evento attivante ammesso.

## 4. L4–L5 — narrazione e osservazione

### 4.1 L4

Gli AUX L4 `narrate` e `snapshot` restano consecutivi e appaiati; il
`self_hash` è calcolato sul preimage del documento cumulativo ancorato
all'hash reale/simulato del narrate AUX, come ridefinito per v2. Le fonti
devono distinguere seq AUX citate e fonti L0 originarie secondo S5; regola
esatta di bibliografia, template e compatibilità dei documenti **FIRMA S7**.

Per la divergenza L4 propongo la modifica chiesta da kingspina: confronto
dei campi semantici escludendo `head_hash` e `self_hash`, verifica separata
di ciascun hash. **FIRMA S8:** definire precisamente i campi semantici e
l'ambito delle differenze irrilevanti, senza assumere equivalenza byte per
byte dei documenti completi.

L'istantanea diagnostica dopo `model_update` L5 è un **artefatto esterno**:
non è `narrate`, non è `snapshot` L4 e non entra nella catena del log. Schema,
canonizzazione e hash sono nel manifest/oracoli da firmare; non si afferma
che L4 narri automaticamente gli eventi L5.

### 4.2 L5

Per v2 propongo di emendare esplicitamente L5.2: l'osservazione esterna è
registrata dal driver in un AUX `L5/observation`, non incorporata nel
SENSE del ciclo. Il `data` contiene l'equivalente canonico delle osservazioni
`altro`/`comportamento`; l'altro agente non scrive direttamente nel log.
Le fonti e la modalità di acquisizione devono essere dichiarate, senza
dedurre che l'hash dimostri verità o autenticità dell'osservazione.

Predict/model_update, argmax e finestre mantengono i contratti L5, adattati
a tipo e cutoff v2; i vecchi eventi `SENSE` con `osservazioni` restano
leggibili nel protocollo storico. **FIRMA S9:** schema dell'osservazione AUX,
fonti ammesse e interpretazione della frontiera. Il secondo ordine e il suo
oracolo concreto sono ancora da definire nel manifest (**S14**).

## 5. L6, gate e artefatti esterni

L6 mantiene per questa prova 100 coppie di seed `1000..=1099`, bracci
separati e policy della legge L6. Il log principale v2 e i due log L6 di
ogni coppia sono collegati tramite metadati esterni preregistrati; nessuna
append L6 al principale è proposta. Non si afferma che la policy L6 usi le
viste L1–L5 o che queste causino un miglioramento.

Manifest, oracoli, `scenario_id`, ID run/interazione, log di verifica e una
riga JSONL di riepilogo per coppia devono avere schema canonico, byte e
SHA-256 preregistrabili. **FIRMA S10:** campi esatti, riferimenti agli hash,
percorsi, conservazione e strategia per evitare riferimenti circolari fra
manifest e hash dei risultati. Gli hash dei risultati sono prodotti dopo la
run, non possono essere conosciuti nel manifest iniziale; il manifest può
fissare il metodo, non inventare quei digest.

Timeout: 5 secondi per coppia, con meccanismo interrompente esterno e
confini di misura nel manifest. Replay: rigenerare indipendentemente ogni
azione e reward per entrambi i bracci e confrontare output canonici e hash.
L'implementazione odierna di `run_pair`/`fold_run` non dimostra da sola tale
rigenerazione. **FIRMA S11:** meccanismo concreto di timeout, replay,
validazione dell'appaiamento e trattamento di errori/timeout. La scelta
`333` milli come benchmark teorico per tre classi, limitata alla prova, e la
relazione con `spec/L6_SPEC_v0.2.md` §4 richiedono **FIRMA S12**.

Gate positivo: fixture valida e appaiata accettata. Gate negativo: copia
alterata, rifiutata prima della valutazione per motivo preregistrato. Il
rifiuto previsto dà PASS al gate negativo e non costituisce una run
invalida; l'accettazione della fixture alterata dà FAIL. Una run prevista
invalida rende non superato il complessivo. Fixture, punto del gate e
oracoli esatti sono **FIRMA S13/S14**. Nessuna conclusione statistica di
efficacia viene tratta dallo stress test.

## 6. Versionamento, compatibilità e artefatti storici

### Proposta di identificazione

- Il **manifest esterno** identifica esplicitamente `protocol_version`
  (nomi letterali e schema: **S15**), commit, immagine e strumenti. Il
  verificatore seleziona il parser previsto e rifiuta versioni sconosciute:
  non indovina la versione dalla presenza o assenza di AUX.
- In assenza del manifest/versione, il trattamento dell'input è **S15**:
  non assumere automaticamente v2. Se un log JSONL viene distribuito senza
  manifest, non contiene da solo un'indicazione di versione nel formato
  proposto: questa limitazione va dichiarata e risolta nella scelta S15.
- La preimage degli eventi storici non cambia. Nessuna conversione,
  risigillatura o aggiunta di AUX a log storici; un nuovo log esteso è un
  nuovo artefatto, non una riscrittura della storia.

### Impegni di compatibilità proposti

1. **Reader v2 su v1:** i nuovi lettori devono continuare a parse/verificare
   i log storici v1 e ricostruire le viste storiche secondo il contratto v1;
   test di regressione con byte e hash storici fissati.
2. **Reader v1 su v2:** i vecchi parser non riconoscono `AUX`; non si
   promette compatibilità in avanti. Il rifiuto deve essere dichiarato,
   non presentato come corruzione di un log v2 valido.
3. **Eventi storici e nuovi nello stesso log v2:** scegliere se permettere
   vecchi payload `l2/l3/l4/l5` e osservazioni SENSE insieme agli AUX oppure
   vietarli nel profilo v2. Modalità mista, cutoff, dedup, ordine e rischio di
   doppio conteggio richiedono **FIRMA S16**. Nessun reader deve applicare
   entrambe le rappresentazioni alla stessa decisione senza una regola
   esplicita. L'eventuale sola migrazione di log v1 come prefisso v2 va
   disciplinata anch'essa da S16.
4. **API:** mantenere API esistenti per la semantica v1 e introdurre API
   versionate o una selezione esplicita del protocollo; la forma concreta
   è **FIRMA S17**. Nessuna compatibilità è dichiarata verificata prima dei
   test.
5. **Report e risultati storici:** SPEC, leggi, report, firme, commit, hash
   e artefatti precedenti restano reperti della rispettiva versione. I
   verdetti storici non diventano automaticamente un gate del protocollo
   v2; i risultati futuri non sovrascrivono i report precedenti. File
   generati e percorsi concreti richiedono S10.

## 7. Impatto implementativo e percorso, subordinati a firma

Se A e gli articoli saranno approvati: modifiche coordinate a L0 (evento,
parser/canone, hash, append, verifica di fase, ciclo, fold), L1–L5
(decodifica AUX, provenienza, cutoff, finestre, decisioni, replay e API), e
verificatore L6 esterno per replay/gate/artefatti. L'invariante «nessuna
modifica al progetto durante lo stress test» riguarda la **run di prova**:
non elimina la necessità di un'implementazione separata, testata e
committata prima di fissare il commit da verificare. La progettazione non
autorizza ora tali modifiche.

Percorso proposto: approvazione normativa → implementazione e regressioni
test-first su un commit identificato → manifest e oracoli completi e
approvati → commit esatto post-emendamento/implementazione → esecuzione
Docker autorizzata → report senza accettazione automatica. Test di log
storici, rifiuto AUX in fase illegale, sequenze multiple, provenienze,
chiusura L2, E1/E2/E3, hash L4, osservazioni L5, replay indipendente L6 e
gate negativo/positivo sono da includere nel piano di test dopo la firma.
Non si afferma che tali test esistano già o che l'architettura passi.

## 8. Riepilogo delle firme umane richieste

- **S1:** A indicata da kingspina come proposta; approvazione formale
  dell'emendamento ancora necessaria.
- **S2:** AUX dopo genesi e dopo STATE compensativo; definizione della fase.
- **S3:** errori e mapping per parser, fold e API L1–L5.
- **S4:** schemi canonici completi `data`/`provenance`/`sources`, fonti vuote
  e tipi di fonte ammessi per ogni `level/kind`.
- **S5:** `origin_seq`, seq di attivazione, sorgenti e dedup L1–L3.
- **S6:** chiusura esplicita indicata da kingspina come proposta; testo
  completo, casi limite e approvazione formale ancora necessari.
- **S7:** regole L4 v2 per template, bibliografia, hash e compatibilità.
- **S8:** perimetro esatto della divergenza semantica L4.
- **S9:** schema e provenienza delle osservazioni L5 e frontiera.
- **S10:** schema, collocazione e hash degli artefatti esterni.
- **S11:** timeout interrompente, replay indipendente e appaiamento L6.
- **S12:** benchmark casuale e rapporto con la SPEC L6.
- **S13:** definizione operativa di gate e di FAIL/run invalida.
- **S14:** manifest completo, fixture, oracoli e secondo ordine L5.
- **S15:** identificazione protocollo, compresi log senza manifest.
- **S16:** regole per log misti e/o estensione di prefissi storici.
- **S17:** versionamento e compatibilità delle API.

**Firma di kingspina sull'emendamento:** non apposta. S1 e la direzione S6
sono proposte comunicate da kingspina, non una firma dell'emendamento; le
altre decisioni restano non firmate. Nessuna implementazione o esecuzione è
autorizzata da questo aggiornamento.
