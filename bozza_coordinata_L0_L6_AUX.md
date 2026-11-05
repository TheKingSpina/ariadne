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
| `L2` | `config`, `select`, `summary`, `barrier` | campi corrispondenti a L2.1/L2.21, escluso il vecchio discriminante top-level `l2`; chiusura esplicita con `phase="close"` proposta in S6 |
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

kingspina ha rettificato la proposta precedente, che chiamava
imprecisamente «non-L2» l'evento di chiusura. La nuova formulazione
proposta per v2 è:

1. La finestra si apre e i `L2/select` ammessi registrano le espulsioni;
   l'ordine concreto è fissato nel manifest.
2. Tra uno `STATE` di ciclo completato e il `SENSE` del ciclo successivo,
   un AUX **`L2/barrier` con `phase="close"`** chiude esplicitamente la
   finestra. È classificato L2, non non-L2.
3. Dopo la chiusura, un AUX **`L2/summary`** riporta soltanto le espulsioni
   già registrate nella finestra. Il nuovo `SENSE` resta fuori finestra e la
   tripletta L0 `SENSE → ACTION → STATE` rimane contigua e indivisibile.
4. Rimane nella bozza la proposta precedente di rifiutare un evento non-L2
   prima della chiusura esplicita: non chiude implicitamente la finestra.
   La compatibilità di questa proposta con tutti i casi limite di v2 resta
   da approvare; non è una regola vigente per i log storici.

Questa è una **proposta di modifica di L2.25 per v2**, non la regola vigente
per i log storici. Non implica una modifica retroattiva ai report o ai log
v1. Restano da definire e approvare: schema chiuso dei payload; significato
delle seq associate alle espulsioni; eventi L2 ammessi nella finestra;
trattamento di log interrotto o terminato tra chiusura e summary; errori
specifici. Va inoltre chiarito se il summary debba essere immediatamente
consecutivo alla chiusura e se sia richiesto anche senza espulsioni: L2.15
vigente non lo prevede per una finestra senza espulsioni. Nessuna di queste
alternative è scelta in questa bozza (S3–S4 e dettaglio S6).

**S6 — proposta ricevuta, non firma formale:** la chiusura `L2/barrier` con
`phase="close"`, seguita dal `L2/summary`, è riportata sopra; il testo
normativo completo e i casi limite richiedono la firma di kingspina. L'agente
non firma. Manifest e oracoli restano incompleti; nessun ordine eseguibile è
dichiarato pronto.

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

**Firma di kingspina sull'emendamento:** non apposta. Le indicazioni più
recenti di kingspina sono registrate nell'appendice seguente: prevalgono
come input di redazione sulle proposte precedenti incompatibili, senza
costituire da sole un testo normativo completo. Nessuna implementazione o
esecuzione è autorizzata da questo aggiornamento.

## 9. Appendice — indicazioni successive di kingspina

Le indicazioni seguenti aggiornano la direzione della bozza. **Non sono una
firma dell'emendamento completo** e non modificano le leggi vigenti. Le
sezioni precedenti contengono proposte storiche: dove differiscono da questa
appendice devono essere riformulate prima della presentazione alla firma.

### Scelte espresse

- **S1 — architettura:** strada **A**, AUX nel log.
- **S2 — collocazione e atomicità:** AUX ammessi solo dopo GENESIS a tick
  `0` o tra `STATE` e `SENSE`; append atomica.
- **S4 — campi dello schema chiuso:** `protocol_version`, `level`, `kind`,
  `tick`, `sources`, `provenance`, `data`.
- **S5 — origine:** `origin_seq` indica sempre la **seq L0 sorgente**,
  distinta dalla seq dell'AUX di estrazione. Questa indicazione sostituisce
  la proposta della §3.1 che faceva coincidere `origin_seq` con la seq AUX.
- **S6 — chiusura e summary:** `L2/barrier` con `phase="close"` deve essere
  seguito immediatamente da `L2/summary` alla seq globale successiva; un
  summary vuoto è valido. Nessun evento intermedio è ammesso; log interrotto
  tra i due o sequenza diversa non è valido. Questa indicazione supera la
  domanda della §3.2 sulla consecutività e sulla finestra senza espulsioni;
  la relazione con L2.15 storico richiede testo di emendamento esplicito.
- **S7–S17 — indirizzi indicati:** SHA-256 su JSON canonico; confronto L4
  dei campi semantici preregistrati; osservazioni L5 con `agente`,
  `soggetto`, `ordine ToM`, `proposizione`, `confidenza`, `sources`; ID
  deterministici di scenario/run/interazione; gate negativo con payload
  alterato e rifiuto `digest_mismatch`; replay indipendente di ogni azione e
  reward; versione obbligatoria nel manifest; rifiuto di log misti o non
  dichiarati. Gli oracoli si congelano a partire dai payload definitivi:
  non si indovinano valori attesi.

### Dettagli ancora da completare prima della firma

1. **Schema e canone:** tipi e domini di tutti i campi, regole di escaping,
   struttura di `data`/`provenance`/`sources` per ogni `level/kind`, e
   definizione byte-per-byte del JSON canonico. Chiarire se `tick` è solo il
   campo top-level L0 o compare anche nel payload, con regola di uguaglianza
   se duplicato; collocazione esatta di `protocol_version` e sua relazione
   con la versione obbligatoria nel manifest. I payload L0 attuali sono
   piatti: l'eventuale codifica interna va definita espressamente.
2. **Atomicità e fase:** definire il confine dell'append atomica, il
   comportamento in caso di errore e se la coppia close/summary sia atomica
   come unità o composta da due append atomiche con prefisso interrotto
   invalido. Chiarire se uno `STATE` compensativo abiliti AUX; come verificare
   l'ordine dopo genesi e tra cicli senza ridurre la verifica alla sola
   integrità hash.
3. **Origine e attivazioni:** se un AUX cita più seq L0 sorgenti, quale
   singola seq diventa `origin_seq` e come si valida la relazione con il
   nodo/arco; regola di riprova, conteggio di attivazioni e deduplicazione
   L2/L3 quando più AUX citano la stessa fonte, nel rispetto della decisione
   espressa su `origin_seq`. Tipi delle fonti, riferimenti precedenti e
   significato di `sources`/`provenance` restano da definire.
4. **Summary e origine:** schema esatto del summary, significato delle seq
   delle espulsioni, forma canonica del summary vuoto e gestione delle altre
   possibili cause di errore; la chiusura immediata sostituisce per v2 la
   semantica storica soltanto mediante testo firmato.
5. **L4 e L5:** elenco esatto dei campi semantici L4 e preimage di ogni hash;
   domini e codifica di agente, soggetto, ordine ToM, proposizione e
   confidenza, significato e causalità di `sources`; rapporto con le API e
   gli eventi storici. Uno snapshot diagnostico dopo L5 non diventa per
   questo una narrazione L4.
6. **Versionamento e compatibilità:** versione concreta, scelta e schema
   delle API, trattamento dei log v1 storici corredati da manifest esplicito,
   comportamento dei vecchi reader e definizione verificabile di «log
   misto»; il rifiuto dei log non dichiarati non equivale a cancellare o
   riscrivere gli artefatti storici.
7. **Gate e artefatti:** fixture positiva, byte e posizione della modifica
   negativa, punto in cui si verifica il digest e schema dell'errore
   `digest_mismatch`; significato, input e canone di ciascun digest,
   identificativi deterministici, percorsi, conservazione e assenza di
   riferimenti circolari. Un rifiuto negativo atteso dà PASS al gate e non
   conta come run invalida.
8. **Manifest e oracoli:** sequenza di eventi, tick e seq, fixture L2/L3,
   copertura ToM, timeout, procedura indipendente di replay L6 e valori
   attesi sono da derivare dai payload definitivi, preregistrare e approvare.
   Il commit da eseguire resta da fissare dopo l'emendamento e
   l'implementazione approvati.

**Fermata:** nessuna modifica alla logica o alle leggi vigenti, nessun test
e nessuna esecuzione Docker per effetto di questa appendice.

## 10. Proposte concrete sui dettagli aperti — da approvare

Questa sezione trascrive le proposte dell'agente presentate a kingspina dopo
le indicazioni della §9. **Non sono decisioni né firme.** Dove differiscono
dalle sezioni precedenti sono alternative da sottoporre alla firma, non
modifiche automatiche della legge. Gli schemi completi restano da redigere.

### 10.1 Campi e collocazione

L'evento mantiene i campi top-level L0 `seq`, `tick`, `type`, `payload`,
`prev_hash`, `hash`. Per `AUX` si propone un payload piatto e chiuso:

```text
protocol_version : stringa
level            : stringa
kind             : stringa
sources          : stringa con JSON canonico
provenance       : stringa con JSON canonico
data             : stringa con JSON canonico
```

Si propone che `tick` sia **solo top-level**, non duplicato nel payload.
`protocol_version = "2"` e la versione del manifest devono coincidere.
`sources` è un array canonico crescente e deduplicato di seq L0 sorgenti
precedenti; `provenance` e `data` sono oggetti canonici con schema specifico
per `level/kind`. Gli AUX sarebbero ammessi dopo genesi a tick 0 o fra
`STATE` di ciclo completato e `SENSE` successivo; tick uguale a quello
dell'ultimo STATE ammesso, nessun draw.

**Da firmare:** collocazione del tick e versione; schemi chiusi completi,
domini e regole byte-per-byte del JSON canonico per ogni coppia
`level/kind`, trattamento degli STATE compensativi e condizioni di fonti
vuote. Questa proposta non sostituisce l'indicazione di kingspina che `tick`
fa parte dello schema complessivo: propone di soddisfarla nel top-level L0.

### 10.2 Atomicità close/summary

Si propone una **singola operazione atomica di append della coppia**:
`AUX L2/barrier` con `phase="close"`, seguito da `AUX L2/summary`, anche
vuoto in assenza di espulsioni, stesso tick e seq consecutive. O vengono
apposti entrambi, oppure il log non cambia. Un log esterno che termini dopo
close o contenga un evento intermedio è rifiutato.

L'operazione richiede una nuova API: due chiamate separate all'attuale
`Log::append` non garantiscono da sole l'atomicità della coppia. Semantica
di errore, verifica e garanzie effettive restano da firmare e testare.

### 10.3 Sorgenti multiple e attivazioni

Si propone di distinguere `aux_seq`, seq globale dell'estrazione, da
`sources`, seq L0 sorgenti, e da `origin_seq`, **minore seq L0 sorgente
valida associata alla prima dichiarazione di quella specifica identità o
arco**. Se un AUX dichiara più membri, ciascuno indica le proprie sorgenti
in `provenance`; l'elenco complessivo dell'evento non basta a stabilire il
rapporto membro-fonte.

Per L2/L3 si propone una sola attivazione per identità e `aux_seq`, con
`origin_seq` stabile e sorgenti conservate per attivazione. **Decisione
esplicita richiesta:** due AUX distinti che citano la stessa sorgente L0
contano come due attivazioni, oppure si deduplica per identità e seq
sorgente? La scelta non è stata fatta. Tipi di fonte e validazione della
relazione con ogni membro restano da definire.

### 10.4 L4

Si propone di conservare coppie consecutive `AUX L4/narrate` con `at`,
`text`, `sources`, e `AUX L4/snapshot` con `at`, `self_hash`. `at` identifica
la seq globale dell'evento che genera la revisione; le sorgenti L0 sono
conservate nella provenienza.

Il confronto di divergenza preregistra `schema_version`, `statements[].at`,
`statements[].sources`, `statements[].text` come campi semantici. Esclude
`head_hash` e `self_hash` dal confronto di uguaglianza e verifica
separatamente ciascun hash sul proprio preimage canonico. Se eventi
irrilevanti spostano le seq, possono cambiare anche `at` e `sources`: la
fixture deve preservare tali seq oppure va firmato un confronto diverso.
Schema definitivo e preimage esatti restano da approvare.

### 10.5 L5

Per ogni osservazione `AUX L5/observation` si propongono:

```text
agente       : stringa non vuota
soggetto     : stringa non vuota
ordine_tom   : intero non negativo
proposizione : stringa non vuota
confidenza   : intero fra 0 e 1000
sources      : array canonico di seq L0 precedenti
```

La scala intera **0–1000 è una proposta**, non una scelta già approvata.
Occorre definire il rapporto con il modello L5 storico `altro`/
`comportamento` intero. Si propone un adattamento esplicito e versionato,
senza inventare una conversione da proposizione/confidenza a comportamento:
kingspina deve scegliere se mantenere i campi distinti o firmare una
funzione deterministica. Il test ToM deve fissare significato degli ordini
ammessi e output esatto. La relazione fra `sources` per osservazione e
`sources` dello schema AUX esterno resta da specificare.

### 10.6 Versioni e log storici

Proposta: v1 identifica log storici dichiarati nel manifest, senza AUX; v2
identifica log dichiarati con schema AUX v2. Versione mancante o sconosciuta
e `AUX` in un log dichiarato v1 danno rifiuto. Si propone il rifiuto in v2
di log che mescolino vecchie e nuove rappresentazioni delle decisioni
L1–L5; la definizione precisa di «misto» richiede firma.

Nessuna conversione, riscrittura o ricalcolo degli hash storici. Un nuovo
lettore deve poter verificare v1 quando dichiarato esplicitamente; un vecchio
lettore può rifiutare v2. Si propongono **API versionate separate** per
evitare interpretazioni silenziose; la compatibilità non è dimostrata finché
non è implementata e testata.

### 10.7 Manifest, oracoli e gate

Il manifest definitivo fisserebbe commit post-emendamento, immagine Docker,
schema, eventi, tick, ordine, identificativi deterministici, fixture,
metodo di replay indipendente, timeout di 5 secondi per coppia, campi degli
artefatti e metodo SHA-256. Gli oracoli attesi vengono **derivati dai
payload definitivi e congelati prima della run**, con preimage e metodo
documentati. Gli hash degli artefatti prodotti dalla run sono calcolati
dopo la run: il manifest iniziale può fissarne il metodo, non inventarli.

Per il gate negativo si propone di alterare un byte del payload della
fixture positiva conservando l'hash originale: `digest_mismatch` prima
della valutazione → PASS del gate negativo; accettazione o motivo diverso
da quello preregistrato → FAIL. Il rifiuto atteso non è una run invalida.
Byte precisi, fixture, punto del controllo e schema dell'errore restano da
definire.

### 10.8 Decisioni principali ancora richieste

1. Due AUX distinti con la stessa sorgente L0 contano una o due volte per
   le attivazioni L2/L3?
2. Come si collega la nuova proposizione con confidenza al comportamento
   intero del modello L5 vigente?
3. In v2 si vietano completamente le vecchie rappresentazioni L1–L5 nello
   stesso log? Qual è la definizione esatta di log misto?
4. È approvata la scala di confidenza 0–1000? Sono approvati tick solo
   top-level, la coppia di append atomica e le API versionate separate?

Resta sospesa qualsiasi implementazione o esecuzione fino ad approvazione
dell'emendamento completo e dei manifest/oracoli completi.

## 11. Proposta successiva di kingspina — chiusura L2 atomica

**Proposta da approvare, non legge vigente.** Per il protocollo esteso v2,
kingspina propone di **vietare la chiusura implicita** della finestra L2 e
richiedere, fra `STATE` e il `SENSE` successivo, il seguente batch atomico:

1. `AUX L2/barrier` con `phase="close"`;
2. `AUX L2/summary`, anche vuoto, con le espulsioni già registrate nella
   finestra.

I due AUX devono avere **seq globali consecutive**, lo **stesso tick** e
nessun evento intermedio; non avanzano il tick e non interrompono una
tripletta L0. Se queste condizioni non sono rispettate, **l'append viene
rifiutata**. Come nella proposta di operazione atomica della §10.2, il
confine e le garanzie di atomicità devono essere definiti nel testo da
firmare: nessuna chiamata all'API attuale viene dichiarata sufficiente.

Per la lettura dei log esterni resta da precisare l'errore applicabile a una
coppia incompleta o non valida; non si sostituisce con questa bozza la
semantica vigente dei log storici v1. Schema canonico del summary vuoto,
contenuto e provenienza delle espulsioni, eventi ammessi prima del batch e
mapping degli errori restano da completare.

Questa proposta non firma gli altri punti aperti e non autorizza modifiche
alla logica, test o esecuzione Docker.

## 12. Decisioni di dettaglio S6 per delega — testo da ratificare

Su delega di kingspina «decidile tu», le seguenti decisioni completano la
proposta S6 della §11. La delega riguarda i dettagli aperti di S6 qui sotto;
non si presume estesa agli altri punti S1–S17. **Testo ancora da ratificare:**
non modifica le leggi vigenti e non autorizza implementazione o esecuzione.

### 12.1 Batch e atomicità

La chiusura è richiesta tramite una sola operazione logica
`append_l2_close_summary`, nome provvisorio dell'interfaccia, che riceve
entrambi gli eventi già costruiti. Prima di mutare il log valida la fase
L0, finestra aperta, tick, schemi, `sources`, espulsioni e consecutività
prevista, e prepara entrambe le preimage e i rispettivi hash in memoria.
Se un controllo o la preparazione fallisce, **nessuno dei due eventi è
aggiunto**. In caso di successo, i due eventi sono aggiunti in ordine,
con seq `n` e `n+1`, stesso tick dell'ultimo STATE ammesso, e senza eventi
intermedi. `summary.prev_hash = close.hash`.

L'atomicità qui è la proprietà osservabile dell'API sul log in memoria:
successo con due eventi o errore senza mutazione. Non è una garanzia di
durabilità su disco, di crash consistency o di scritture concorrenti: tali
proprietà non sono specificate da questa decisione. La serializzazione
JSONL non deve essere presentata come commit transazionale durevole.

### 12.2 Contenuto e summary vuoto

Il summary riporta **solo** le espulsioni dei select già apposti nella
finestra appena chiusa, deduplicate e in ordine crescente secondo la seq di
origine L0 delle identità; la provenienza conserva anche i riferimenti ai
select registrati. Per l'identificazione delle identità con la stessa
`origin_seq` si conserva il criterio L2.26, da adattare esplicitamente al
formato AUX e alla decisione finale sulle attivazioni S5.

Il contenuto logico del summary vuoto ha `expulsions=[]` e `text=""`;
le rispettive fonti di espulsione sono vuote. L'evento summary è comunque
obbligatorio subito dopo close. Le esatte chiavi e la loro codifica nei
campi canonici `data`, `sources`, `provenance` restano subordinate allo
schema comune S4; non si deduce che ogni altro tipo di AUX ammetta fonti
vuote.

### 12.3 Altri eventi e casi incompleti

Durante una finestra aperta sono ammessi soltanto gli eventi L2 v2
autorizzati dallo schema della finestra; la chiusura avviene esclusivamente
tramite il batch sopra. Un tentativo di appendere un evento non-L2 prima
della chiusura è rifiutato senza mutare il log; non chiude implicitamente la
finestra. Una seconda apertura e una chiusura senza finestra aperta sono
errori.

Per un log esterno dichiarato v2, un `close` in coda, un evento intermedio,
un summary non corrispondente o una coppia con seq/tick errati dà errore
esplicito e **nessun replay accettato come valido**. Un log v2 che termina
con finestra aperta ma **senza** avere apposto close rimane un prefisso
incompleto tollerato in lettura, non una finestra completata; il gate dello
stress test può richiedere la chiusura completa. Questa tolleranza non
consente di proseguire il ciclo L0 finché la finestra non è chiusa.

### 12.4 Errori proposti e compatibilità

Si propongono codici distinguibili `l2_window_state` per apertura/chiusura
non ammesse, `l2_close_summary_incomplete` per coppia mancante o interrotta,
`l2_close_summary_mismatch` per tick, seq o contenuto della coppia non
conformi, e `l2_non_l2_before_close` per il tentativo di evento non-L2.
Nomi, mapping verso enum/API e precedenza se più controlli falliscono sono
proposti per la definizione coordinata S3; non si assume che esistano nel
codice attuale.

Per la **v2**, la chiusura implicita L2.25 e la regola storica L2.15 «nessun
summary senza espulsioni» sono sostituite dal batch esplicito con summary
anche vuoto. La semantica **v1 rimane invariata** per i log storici dichiarati
v1: nessuna riscrittura né ricalcolo degli hash. Restano aperti schema S4,
provenienza/attivazioni S5, versionamento e API S15–S17 e il manifest con
oracoli concreti.

## 13. Decisioni progettuali per delega — bozza coordinata da ratificare

kingspina ha delegato all'agente le decisioni progettuali ancora aperte
elencate dopo la §12. Le scelte di seguito sono assunte **per delega nella
bozza**, non rappresentano un'implementazione già esistente né una firma
umana già apposta sul testo definitivo. In caso di contrasto con le proposte
precedenti di questo file, questa sezione registra la scelta più recente;
non riscrive le leggi o gli artefatti storici. Manifest e oracoli concreti
vanno ancora completati e approvati: la delega non autorizza a inventare
hash o risultati attesi né ad avviare test o Docker.

### 13.1 Profilo v2 e codifica

Un evento AUX usa il consueto involucro L0 `seq,tick,type,payload,prev_hash,
hash`, con `type="AUX"`; `tick` è soltanto top-level. Il payload piatto ha
esattamente sei chiavi stringa: `data`, `kind`, `level`, `protocol_version`,
`provenance`, `sources`. `protocol_version` è `"2"`; `data` e `provenance`
contengono ciascuno un oggetto JSON canonico, `sources` un array JSON
canonico di interi. Ogni valore interno è una stringa L0; chiavi in ordine
lessicografico di byte UTF-8, nessuno spazio, stringhe e numeri nella
grammatica canonica L0/F1. I numeri interni rappresentano interi `i64` non
negativi quando sono seq, conteggi o classi; riferimenti non rappresentabili
sono errore, mai cast silenziosi. Oggetti annidati e array interni seguono
ricorsivamente lo stesso canone. I campi non elencati sono vietati.

`sources` è ordinato, deduplicato e contiene soltanto seq **precedenti** di
eventi L0 non-AUX nel log principale; per decisioni che dipendono da AUX,
gli identificativi di tali AUX stanno in `provenance`, non in `sources`.
La genesi non è una sorgente di estrazione L1. Per `L5/observation`,
`sources` identifica invece ancore temporali L0, non fonti del contenuto;
la genesi è ammessa come ancora a tick 0 secondo la §13.8. I tipi ammessi
come fonti L1 restano da fissare nel manifest definitivo. Una fonte esterna
non presente nel log non viene inventata come seq L0.

Per tutti i kind tranne `L5/observation`, `provenance` ha esattamente due
campi: `members` e `refs`, entrambi array. Per `L5/observation` aggiunge
esattamente il campo `external` specificato nella §13.8.
`refs` è array deduplicato crescente di seq AUX precedenti; `members` è
array di oggetti esatti `{"index":<i64>,"sources":[<seq>,...]}`, ordinato
per index, senza duplicati. Per `L1/extract` contiene una voce per membro,
indicizzato sulla concatenazione `nodes` poi `edges` nell'ordine del loro
array; ogni `sources` del membro è non vuoto, deduplicato crescente e
sottoinsieme di `sources` dell'AUX. L'unione delle fonti dei membri deve
coincidere con `sources`. Per gli altri kind `members=[]`; `refs` elenca
gli AUX da cui la decisione dipende. `sources=[]`, `refs=[]` sono ammessi
per configurazioni e barriere; gli altri kind devono soddisfare le fonti
minime indicate dal loro catalogo e dal manifest. Nessun riferimento a seq
futura o inesistente; l'hash della catena prova l'integrità del registro,
non la verità della provenienza.

Catalogo chiuso di `data` (gli array e oggetti indicati sono valori interni
del JSON canonico contenuto nella stringa `data`):

| level | kind | Campi esatti in `data` |
| --- | --- | --- |
| L1 | extract | `edges`, `nodes`: array di membri esatti come L1 F1 |
| L2 | config | `k`, `tau` |
| L2 | barrier | `phase`, stringa `open` o `close` |
| L2 | select | `at`, `downs`, `ups` |
| L2 | summary | `expulsions`, `text` |
| L3 | config | `H`, `T` |
| L3 | promote, archive, recall | `at`, `id`, `origin_seq` |
| L4 | narrate | `at`, `sources`, `text` |
| L4 | snapshot | `at`, `self_hash` |
| L5 | observation | `observations`, array dei membri §13.4 |
| L5 | predict | `at`, `id`, `previsto`, `sources` |
| L5 | model_update | `at`, `id`, `osservato`, `previsto`, `sources` |

I domini di `k`, `tau`, `T`, `H`, ID, `at`, liste e transizioni restano
quelli delle rispettive leggi, adattati alle seq globali v2 qui specificate.
Liste nel `data` interno sono array JSON, non stringhe JSON annidate una
seconda volta. `at` di L2/L3/L4/L5 si riferisce all'evento generatore
autorizzato dalla legge del livello; quando è un AUX, deve essere precedente
all'AUX decisionale. Le `sources` nel `data` delle decisioni conservano il
significato bibliografico specifico del livello, e possono citare seq AUX;
sono distinte dalle `sources` esterne, che contengono solo seq L0; per
`L5/observation` queste ultime sono ancore temporali, come nella §13.8.
Il controllo di coerenza e la propagazione delle sorgenti esterne avvengono
risalendo i `refs` in `provenance`; i riferimenti necessari e quelli vuoti
per ciascun evento del manifest vanno elencati prima della sua firma.

### 13.2 Provenienza e attivazioni

Per ogni membro L1, `origin_seq` è la minima seq L0 presente nelle fonti
della sua **prima dichiarazione** registrata, selezionata dall'array fonti
del membro stesso. Dichiarazioni successive non cambiano l'origine. Per L2 e
L3 si deduplica l'attivazione per coppia **(identità, seq L0 sorgente)**:
due AUX distinti che citano la stessa sorgente per la stessa identità contano
una sola attivazione; fonti distinte possono contarne una ciascuna. Questa
è una scelta v2 per delega, diversa dalla proposta precedente per `aux_seq`.
Si usa il tick dell'evento L0 sorgente per il decadimento L3 e la sua seq
per la salienza L2; la decisione è apposta successivamente e cita l'AUX che
ha registrato l'estrazione. Il manifest deve fissare anche il caso in cui
una dichiarazione abbia più sorgenti e dimostrare che i punteggi attesi
sono calcolati secondo questa stessa regola. L2.27 v1 resta invariata per
log v1; questa deduplicazione costituisce una nuova regola v2, non una sua
reinterpretazione retroattiva.

### 13.3 S6 e L4

Si adottano nella bozza le decisioni S6 della §12: append atomica in memoria
della coppia close/summary consecutiva, summary vuoto obbligatorio, nessun
evento non-L2 prima della chiusura. I codici della §12.4 sono la proposta
di errori v2; la precedenza è: fase/finestra, schema, riferimenti e infine
contenuto/hash. Un log esterno interrotto tra close e summary è invalido.

L4 v2 usa gli schemi del catalogo §13.1. `self_hash` è SHA-256 minuscolo
del documento canonico senza il campo `self_hash`, con `head_hash` uguale
all'hash del narrate AUX appaiato al momento del controllo snapshot; la
coppia narrate/snapshot deve restare immediatamente consecutiva. I template
L4.3 restano invariati nella forma; il campo `seq` nel testo è la seq
globale dell'evento che genera l'affermazione. La bibliografia narrativa
comprende le seq citate nei template, mentre le fonti L0 esterne sono
ricostruibili attraverso `provenance`. Il confronto fra documenti esclude
`head_hash` e `self_hash` e include `schema_version` e gli `statements`
completi (`at`, `sources`, `text`); ogni hash è verificato separatamente.
Se differenze irrilevanti cambiano le seq citate, non si pretende
l'uguaglianza letterale degli statements: la fixture deve preservare le seq
citate oppure dichiarare non applicabile quel confronto. Lo snapshot
diagnostico dopo L5 resta esterno e non è un evento L4.

### 13.4 L5: adattamento esplicito

Ogni membro dell'array `observations` è un oggetto esatto con `agente`,
`soggetto`, `ordine_tom`, `proposizione`, `confidenza`, `sources`.
`agente` e `soggetto` sono stringhe UTF-8 non vuote; `ordine_tom` è `0` o
`1`; `confidenza` è un intero `0..=1000`. `proposizione` è la stringa
decimale canonica di un intero `i64 ≥ 0`, senza segno né zeri iniziali salvo
`"0"`; `sources` è array non vuoto ordinato e deduplicato di seq L0
precedenti, utilizzate come ancore temporali e non come fonti del contenuto.
Per un AUX observation l'unione delle `sources` dei membri è
uguale alle `sources` esterne. Questo profilo binario di ordine ToM e la
scala di confidenza sono scelte v2 per delega, non requisiti della legge
L5 storica.

Per l'ordine `0`, il modello statistico L5 dell'identità `agente` usa come
`comportamento` l'intero rappresentato da `proposizione`: la conversione è
decimale esatta, senza saturazione o default; `confidenza` e `soggetto`
restano dati di provenienza e non pesano i contatori. L'id dell'agente
segue la formula L5.3; si conserva il tie-break dell'argmax. Per l'ordine
`1`, il dato registra che `agente` attribuisce a `soggetto` la proposizione;
è una vista derivata dalle sole osservazioni registrate, non accesso alla
mente di alcuno. Non aggiorna il contatore di ordine `0`. Il contratto di
secondo ordine e i suoi oracoli devono specificare le osservazioni effettive
da confrontare, senza assumere che una proiezione d'identità dimostri una
credenza indipendente. La fonte dichiarata del contenuto è la fixture
esterna preregistrata nella §13.8; l'ancora L0 non è presentata come causa.

### 13.5 Versione, API e compatibilità

Il manifest esterno obbligatorio dichiara `protocol_version` stringa `"1"`
o `"2"`; versione mancante, sconosciuta o incompatibile col contenuto è
errore. Un log `"1"` non contiene AUX ed è elaborato con la semantica
storica; un log `"2"` accetta solo rappresentazioni ausiliarie v2 per
estrazioni e decisioni L1–L5, senza vecchi payload `nodes`, `edges`, `l2`,
`l3`, `l4`, `l5`, `osservazioni` su eventi non-AUX. Un log che mescola
queste rappresentazioni è rifiutato, non deduplicato euristicamente. Il
trattamento di log privi di manifest è rifiuto di input non dichiarato,
non cancellazione né alterazione dell'artefatto storico.

Si scelgono API separate e nominate per versione; i nomi concreti dei
moduli/funzioni e il catalogo completo degli errori di compilazione sono da
redigere come specifica API prima della firma, non costituiscono autorizzazione
a modificare le API v1. I nuovi reader devono verificare i log v1 dichiarati
senza riscriverli né cambiarne gli hash; i reader v1 possono rifiutare AUX.
Non si afferma compatibilità fino a verifica indipendente.

### 13.6 Gate, identificativi, replay e artefatti

Gli ID di scenario/run/interazione sono SHA-256 minuscolo dei rispettivi
documenti canonici con un campo discriminante di dominio (`scenario`,
`run`, `interaction`) e campi fissati nel manifest; i campi esatti non sono
ancora preregistrati. Manifest e oracoli sono congelati prima della run con
SHA-256 sui loro byte canonici; i digest degli artefatti generati sono
calcolati dopo la run e registrati nel riepilogo, non predetti. Il riepilogo
JSONL di ogni coppia deve contenere scenario ID, run ID, seed, esiti per
braccio, delta, validità, motivo di invalidità o `null`, e hash dei due log;
il log unico di verifica registra anche versioni, commit, gate, errori e
timeout. Schema byte-per-byte, percorso e regola di digest del riepilogo
restano da congelare nel manifest.

Gate negativo: alterare un byte di payload della copia negativa senza
ricalcolare l'hash; rifiuto `digest_mismatch` prima della valutazione e zero
run avviate danno PASS del gate. Accettazione o motivo diverso danno FAIL;
il rifiuto atteso non è una run invalida. Il timeout interrompente è 5
secondi per coppia, misurati dall'avvio alla terminazione del processo della
coppia; nessuna esclusione dal resoconto. Il replay indipendente ricalcola
le decisioni dal prefisso di ogni braccio e confronta ogni azione e reward,
oltre a catena e round-trip canonico; non basta la sola fold del reward.
L'implementazione del verificatore va specificata e revisionata prima della
prova. Il test complessivo passa solo con gate e oracoli rispettati e tutte
le run previste valide, senza conclusione di efficacia statistica.

### 13.7 Lavoro ancora necessario, senza inventare risultati

Restano da preparare: sequenza completa di manifest con payload, tick e seq
effettivi; fixture di finestra L2, archive/recall L3 e ToM L5; semantica
verificabile delle ancore L0 e delle fixture esterne; nomi e firme delle
API; schemi di ID e riepilogo; calcolo indipendente e preregistrazione di
tutti gli oracoli e degli hash. La sola delega progettuale non determina
questi valori né approva l'emendamento completo. Nessuna legge, codice o
test è cambiato; nessuna esecuzione è avviata da questa bozza.

### 13.8 Osservazioni: ancora L0 e fonte esterna distinte

**Scelta progettuale per delega, da ratificare:** per la prova esplorativa la
fonte dichiarata dei contenuti sociali è una fixture esterna preregistrata,
non il payload dello stub PRNG. Il `sources` di ciascuna osservazione
contiene esattamente la seq dell'ultimo `STATE` completato precedente
all'AUX, oppure la seq di GENESIS per un'osservazione apposta dopo genesi a
tick 0. È un'**ancora temporale e di contesto**: non afferma che l'evento
contenga, generi o attesti l'osservazione. L'unione delle ancore dei membri
coincide con le `sources` esterne dell'AUX. L'ammissibilità di GENESIS come
ancora è limitata a L5 e non cambia `origin_seq` o le fonti L1.

Il manifest preregistrato elenca per ogni fixture un `fixture_id` stringa
non vuota e univoca, un percorso relativo, il digest SHA-256 esadecimale
minuscolo e l'ancora L0 attesa. Ogni fixture è un oggetto JSON canonico
UTF-8, senza BOM né newline finale, con la sola chiave `observations` e un
array conforme alla §13.4. Il digest copre i byte esatti del file; la
fixture non contiene il digest del manifest, evitando riferimenti circolari.

Per `L5/observation`, `provenance` è l'oggetto canonico chiuso
`{"external":{"fixture_id":"…","sha256":"…"},"members":[],"refs":[]}`.
L'identificativo e il digest devono corrispondere alla voce preregistrata.
Il driver legge la fixture all'esterno e appone l'AUX; lo stub
`cycle::run_cycle` e il suo SENSE restano invariati. Le viste consumano il
log senza leggere il file: il controllo della fixture è un gate esterno.

Prima della valutazione, il gate verifica la canonicità dei byte, il digest,
l'esistenza e correttezza dell'ancora e la corrispondenza byte per byte fra
fixture e JSON nella stringa `data` dell'AUX. Il controllo del digest
precede quello di canonicità, così la fixture negativa alterata con digest
originale è rifiutata come `digest_mismatch`; il rifiuto previsto è PASS del
gate, non una run invalida. Gli altri errori e la loro precedenza completa
restano da definire nel catalogo S3. Fixture, manifest e log di verifica
vanno conservati per ripetere il controllo senza una fonte viva.

Per questo gate si propone il seguente ordine deterministico dei controlli,
con arresto al primo errore: manifest/voce non validi
`fixture_manifest_invalid`; file assente o illeggibile `fixture_unreadable`;
digest diverso `digest_mismatch`; JSON non canonico o schema errato
`fixture_invalid`; ancora inesistente, non ammessa o diversa da quella
attesa `fixture_anchor_mismatch`; `data` o `provenance.external` non
corrispondenti `fixture_binding_mismatch`. Il confronto del digest usa la
voce del manifest, non un digest fornito dalla fixture o dall'AUX. Prima di
questi controlli non parte alcuna valutazione. I nomi sono proposte v2,
non errori già implementati; restano da integrare nel catalogo generale e
da definire le regole di accesso sicuro ai percorsi.

Il digest verifica la corrispondenza ai byte preregistrati, non la verità,
l'autenticità o la provenienza da un agente reale. Lo scenario resta
sintetico ed esplorativo. La scelta non è una firma né una dimostrazione di
implementabilità: payload, ancore, catalogo errori, manifest e oracoli
restano da completare e approvare prima dello stress test.

### 13.9 Interfacce e frontiere v2 — proposta di contratto

Le interfacce v1 rimangono inalterate. Si propone un modulo separato `v2`
con operazioni a sola lettura `verify_log`, `build_l1`, `build_l2`,
`build_l3`, `build_l4`, `build_l5` e `evaluate_l2`, `evaluate_l3`,
`evaluate_l5`: ricevono un log dichiarato v2 e, per le valutazioni, una base
con lunghezza e hash del prefisso; restituiscono rispettivamente vista o
decisioni, oppure un errore esplicito. Non accedono a file, orologio o RNG
e non scrivono nel log. Il chiamante serializza e appone le decisioni;
`append_l2_close_summary` segue l'atomicità in memoria della §12.1.
I nomi sono proposti, non firme Rust già disponibili. Tipi concreti,
validazione dei singoli payload e compatibilità devono essere completati
prima della firma e testati dopo l'implementazione.

Il verificatore esterno riceve manifest, fixture e log come input distinti:
controlla protocollo e integrità, poi i binding fixture/AUX della §13.8 e
infine invoca le viste. L'accettazione di un log isolato non va presentata
come verifica del binding con una fixture che il verificatore non possiede.
Un percorso fixture deve essere relativo alla directory preregistrata,
senza componenti `..` né link simbolici attraversati: il driver deve rifiutare
accessi che escano dalla directory. La verifica di integrità non autentica
la persona o il processo che ha predisposto gli input.

Le fonti L1 sono una questione distinta dalle ancore L5: non si può inferire
che un valore PRNG contenga semanticamente un nodo o un arco. Per le
estrazioni del futuro manifest occorre preregistrare una trasformazione
deterministica dalla sorgente L0 ai membri L1 e verificarne input e output;
`origin_seq` resta la minima seq L0 della prima dichiarazione secondo §13.2.
Senza tale trasformazione e i relativi casi completi, la sequenza integrata
L1–L4 non va dichiarata validata.

### 13.10 Artefatti esterni e identificativi — proposta da ratificare

Per evitare cicli fra digest, il manifest eseguibile futuro sarà JSON
canonico UTF-8 senza BOM né newline finale; un file separato conterrà il suo
SHA-256, senza includerlo nel manifest stesso. Le fixture e i piani
preregistrati sono riferiti nel manifest tramite percorso relativo e SHA-256
dei rispettivi byte. Gli oracoli preregistrati sono un file separato
referenziato dal manifest con digest; non contengono il digest del manifest.
Il riepilogo della run registra il digest del manifest e quelli degli
artefatti prodotti **dopo** la run, senza essere referenziato dal manifest.
I contenuti effettivi e le regole complete del JSON canonico vanno fissati
e revisionati prima del congelamento; i Markdown attuali non sono quel
manifest eseguibile.

Si propongono ID SHA-256 esadecimali minuscoli del JSON canonico UTF-8 dei
seguenti oggetti chiusi; `scenario_key` è stringa non vuota preregistrata,
`seed` intero, `arm` una delle stringhe indicate:

```text
scenario_id    = SHA256({"domain":"ariadne/scenario/v1","scenario_key":...})
run_id         = SHA256({"domain":"ariadne/run/v1","scenario_id":...,"seed":...})
interaction_id = SHA256({"arm":...,"domain":"ariadne/interaction/v1","run_id":...,"task_index":...})
```

`arm` è `"memoria"` o `"controllo"`; `task_index` va da 0 a 119. Gli ID
sono metadati esterni: non aggiungono campi ai payload a schema chiuso.
La chiave di scenario deve essere univoca tra scenari preregistrati; l'ID
non attesta l'unicità globale né l'autenticità del contenuto.

Per ogni coppia si propone una riga JSONL canonica con chiavi esatte
`control_log_sha256`, `control_test_milli_sum`, `delta_test_milli_mean`,
`error`, `memory_log_sha256`, `memory_test_milli_sum`, `run_id`,
`scenario_id`, `seed`, `valid`: somme intere di 40 reward, delta intero
`(memory_test_milli_sum-control_test_milli_sum)/40` per coppie valide,
`error=null` se valida, altrimenti codice stringa e campi numerici/hash non
disponibili `null`. La divisione è esatta perché ogni reward è multiplo di
1000. Ogni riga termina con LF; il digest dell'artefatto copre i byte JSONL
completi, LF inclusi, in ordine di seed crescente. L'aggregato statistico
L6 resta distinto: riporta anche eventuali valori float secondo la legge L6
nel proprio report, mai nei payload L0. Nessun record invalido viene omesso.

## 14. Profilo attuato dopo autorizzazione esplicita

Il successivo consenso «sì vai» autorizza definizione delle regole v2
mancanti, codice e test Docker, lasciando lo stress test sospeso. Il
contratto del profilo effettivamente implementato è ora
`AUX_V2_PROFILO_IMPLEMENTATO.md`, con codice separato `src/aux_v2/` e test
`tests/aux_v2.rs`. Non è una modifica delle leggi firmate v1 né una
ratifica del manifest dello stress test.

In particolare, i select v2 usano ID L1 in `ups` e `downs` per risolvere
le origini coincidenti, mentre summary mantiene origini e ID degli espulsi
effettivi. Il profilo attuato supporta una fonte per estrazione e un select
per finestra; non implementa tutte le possibilità delle §§10–13. I limiti
e le differenze sono elencati nel contratto. Il prefisso L5 e il piano
L1–L4 di queste bozze non sono automaticamente un unico manifest approvato.
