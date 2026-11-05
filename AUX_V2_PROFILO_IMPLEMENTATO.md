# AUX v2 — contratto del profilo coordinato implementato

Il consenso «sì vai» autorizza definizione delle regole v2 mancanti,
implementazione e test Docker. Questo documento definisce **il nuovo profilo
supportato**, non emenda retroattivamente le leggi firmate. Lo stress test
L0–L6 rimane sospeso fino all'approvazione di manifest e oracoli. Le bozze
precedenti restano storiche: dove differiscono da questo profilo non sono
descrizioni dell'implementazione.

## Separazione e garanzie

- API: `ariadne::aux_v2::V2Log`; tipi e reader v1 non cambiano. Un log v2
  con AUX viene rifiutato dal reader v1; un log composto dai soli cicli L0
  ha gli stessi byte e hash della versione v1.
- La genesi usa il seed e PCG64 canonici esistenti. `cycle()` appone tre
  eventi contigui SENSE/ACTION/STATE, tutti allo stesso nuovo tick, un solo
  draw e policy `stimolo mod 3`. Gli AUX non consumano draw e si appongono
  esclusivamente fra cicli, al tick dell'ultimo ciclo completato o a tick 0.
- Ogni operazione pubblica mutante prepara una copia del log e la rende
  visibile solo in caso di successo. Nessun errore restituito lascia una
  mutazione parziale. Questo non è un protocollo di durabilità su disco,
  gestione dell'esaurimento di memoria o scritture concorrenti.
- Nessuna funzione del modulo legge file, rete od orologio. Il chiamante
  fornisce fixture e metadati; le proiezioni in lettura non appongono eventi.
- I record sono esposti solo tramite slice immutabile. L'oggetto possiede
  un unico registro intercalato di eventi L0 e AUX, non due catene.

## Schema e codifica

I record conservano ordine top-level
`seq,tick,type,payload,prev_hash,hash`; seq e tick sono interi i64 non
negativi. Il preimage conserva il separatore 0x1f fra seq, tick, tipo,
payload canonico e hash precedente. Il tipo aggiuntivo è `AUX`.

Il payload AUX ha esattamente sei chiavi stringa: `data`, `kind`, `level`,
`protocol_version`, `provenance`, `sources`. La versione è `"2"`. I tre
campi strutturati contengono JSON canonico interno: chiavi ordinate UTF-8,
nessuno spazio, interi i64, stringhe con gli escape minimi L0, array e oggetti;
niente float, bool o null. Il parser rifiuta duplicati, numeri non canonici,
escape superflui e annidamento superiore a 32. Nel JSONL è obbligatorio LF
finale. Il confronto con la serializzazione rigenerata controlla anche
chiavi extra, ordine e schemi di ogni evento.

## Operazioni L1–L4

`configure(k,tau,T,H)` appone la coppia L2/config e L3/config, con
`k>=0`, `tau>=1`, `T>=0`, `H>=1`. Non sono ammessi default impliciti.

`extract(source,nodes,edges)` ammette una fonte SENSE del ciclo corrente
completo; non ammette fonti multiple. Ogni membro indica quella fonte,
`origin_seq` viene fissato alla prima dichiarazione, nodi processati prima
degli archi, endpoint già dichiarati obbligatori. Le attivazioni deduplicano
per identità e seq sorgente, comprese dichiarazioni e menzioni nello stesso
evento. Gli archi conservano la propria origine e sono deduplicati per
tripla sorgente/destinazione/relazione. `extract_synthetic(source)` applica
la tabella del manifest ai tick 1, 2, 253, 254; negli altri tick restituisce
errore. I nomi sono convenzioni sintetiche, non significato del PRNG.

**Risoluzione dell'ambiguità L2:** in v2 `data.ups` e `data.downs` di select
sono array di **ID L1**, ordinati lessicograficamente, non di origin_seq.
Un candidato escluso senza essere prima attivo non è un'espulsione.
`summary.expulsions` conserva invece le origin_seq degli elementi realmente
espulsi, deduplicate; `summary.text` conserva gli ID deduplicati ordinati e
uniti da `|`. Nessun cambiamento alla rappresentazione v1.

`open_selection_window(extraction)` ricalcola un select dall'estrazione
precedente più recente del tick corrente, appone open e select e lascia la
finestra aperta. Usa attivi precedenti più candidati attivati dalla fonte
dell'estrazione, salienza intera L2 con età in seq L0 e giudizio alla seq
dell'estrazione, scala score decrescente/origine crescente/ID crescente.
Una sola valutazione L2 per estrazione; giudizi precedenti al cutoff sono
rifiutati. Il profilo supporta **un select per finestra**.

`close_window()` appone close e summary, anche vuoto, con seq consecutive,
tick uguali e hash concatenati. Solo dopo il summary appone le revisioni
L4 del select e del summary. `close_selection_window(extraction)` esegue
l'intera apertura/selezione/chiusura transazionalmente. Nessuna chiusura
implicita: a finestra aperta ciclo, estrazione, configurazioni, L3 e L5
sono rifiutati. Il replay accetta il prefisso completo open/select ancora
aperto, ma non un log interrotto nel batch close/summary o nella revisione.

`evaluate_memory(extraction)` valuta le identità già tracciate all'ultima
estrazione del tick corrente: contributi interi saturanti, zero se età/H
almeno 25, promote con conteggio strettamente maggiore di T, archive a
score zero, recall soltanto se una fonte dell'estrazione attiva l'identità
e segue la seq dell'archive apposto. Il recall riporta a circuito senza
cambiare origine o azzerare attivazioni. Si emette al massimo una transizione
per identità nel giudizio: archive, recall, promote in ordine deterministico.
Le decisioni sono citate da revisioni L4 consecutive narrate/snapshot.

L4 genera una revisione per ogni estrazione con prime dichiarazioni e per
ogni decisione L2/L3. I template restano italiani secondo la legge L4;
la bibliografia narrativa cita seq globali e origini. Le fonti esterne del
record identificano la sorgente L0 del giudizio. Il documento cumulativo
usa `schema_version="2"`; snapshot verifica SHA-256 del documento senza
`self_hash`, con testa pari all'hash del narrate appaiato. Le proiezioni
`graph_document`, `memory_document`, `active_document`, `self_document`
sono canoniche e ancorate alla testa corrente; le prime due includono
rispettivamente `view_hash` e `archive_hash` con esclusione del proprio campo.

## Osservazioni L5 e fixture

`observe(Fixture)` riceve ID, digest preregistrato, byte UTF-8 canonici e
ancora attesa. Verifica prima il digest, poi schema e binding. L'ancora è
l'ultimo STATE completato o GENESIS a tick 0. Non è la fonte del contenuto.
Lo stesso ID non può essere riassociato a digest o ancora diversi. I bytes
di `data` coincidono con la fixture; il payload registra ID e digest in
`provenance.external`. La verifica non autentica persone o fatti.

L'oggetto fixture contiene solo `observations`; i membri hanno esattamente
`agente,soggetto,ordine_tom,proposizione,confidenza,sources`. Classi decimali
i64 non negative, label non vuote, ordine 0 o 1, confidenza 0–1000, fonte
singleton dell'ancora. L'array vuoto è ammesso. I duplicati di ordine 0
incrementano ognuno i contatori saturanti; le seq bibliografiche deduplicano.
La confidenza non pesa i contatori. L'ordine 1 è contenuto dichiarato e
non aggiorna i contatori di ordine 0.

`predict(id)` appone argmax, parità alla classe minore, e sostituisce la
finestra pendente di quell'ID. La prima successiva osservazione di ordine 0
per l'ID la chiude: nessun update in accordo, un solo model_update in
disaccordo, con bibliografia predict/osservazione. Nei batch con più membri
successivi dello stesso ID il primo chiude la finestra, tutti contano nel
modello. `project_prediction(A,T)` è composizione pura identità uguale alla
previsione diretta su T, richiede modelli distinti esistenti, non stabilisce
un accesso a menti indipendenti.

## Replay e confini della verifica

`from_jsonl(bytes,"2",fixtures)` verifica canonicità e catena, rigenera
genesi e ogni operazione dai suoi input registrati, confrontando tutti gli
eventi generati: non si limita alla fold del reward o all'hash. Un record
con contenuto alterato e catena ricalcolata viene comunque rifiutato se non
coincide con le decisioni rigenerate. Ogni osservazione richiede la fixture
esterna; ID duplicati nel registro di fixture sono errore. Non è accettata
la lettura silenziosa come v1 o con versione assente.

La rigenerazione riusa le funzioni del produttore: prova coerenza di replay,
**non una verifica indipendente dell'algoritmo**. I test confrontano inoltre
set e contatori attesi a mano, posizioni preregistrabili e preimage hash.
Non esiste ancora un audit indipendente. Nessun risultato qui dimostra
efficacia causale di L1–L5 e L6 rimane un esperimento separato.

## Cosa questo profilo non implementa

Non è la totalità delle possibilità delle bozze: non supporta estrazioni
multi-sorgente, più select nella stessa finestra, API arbitrarie di append
decisionale autorevole, base di prefisso pubblica per valutazioni pure,
compensazioni v2, caricamento sicuro dei percorsi fixture o runner dello
stress test. Le firme storiche v1 e la legge L6 non sono modificate.
`AuxSuffix` resta il vecchio prototipo limitato, non va usato come reader
v2. Queste esclusioni non devono essere presentate come funzionalità
implementate o verificate.

## Sequenza integrata verificata dai test

Il test `integrated_four_node_sequence_has_preregisterable_positions` usa
seed 1042, config a seq 1–2, 254 cicli L0 ed estrazioni sintetiche ai tick
1, 2, 253, 254. Non include il vecchio prefisso L5, che sposterebbe le seq.
Chiude una finestra L2 e valuta L3 dopo ciascuna delle quattro estrazioni.

| Seq | Evento |
| ---: | --- |
| 6 | Prima estrazione, quattro nodi e arco, fonte SENSE seq 3 |
| 10 | Primo select: delta, beta, gamma; alfa non era attivo e non è espulso |
| 20 | Seconda estrazione di alfa |
| 29 | Promote alfa |
| 785 | Estrazione beta al tick 253 |
| 794, 797, 800 | Archive alfa, gamma, delta, ciascuno seguito dalla revisione L4 |
| 803 | Promote beta |
| 809 | Estrazione alfa al tick 254, fonte successiva all'archive |
| 818 | Recall alfa, origine seq 3 conservata |
| 819–820 | Revisione L4 del recall |

Il log ha 821 eventi inclusa la genesi. Il test confronta le posizioni
attese, gli stati, ogni hash di evento e le proiezioni dopo ricostruzione
JSONL. È un test di integrazione circoscritto, **non lo stress test L0–L6**
né una preregistrazione approvata di tutti gli oracoli dello stress test.

## Decisioni ratificate — 5 novembre 2026

Le seguenti scelte erano ambigue nel profilo e nell'oracolo indipendente. Sono
state ratificate dall'umano («risolvi ambiguità») e sono ora parte del
contratto implementato. Non modificano le leggi v1 firmate.

1. **Ordine L3 nella stessa classe.** Le transizioni sono emesse nell'ordine:
   prima tutti gli `archive`, poi tutti i `recall`, poi tutti i `promote`.
   Dentro `archive`: `origin_seq` decrescente, poi ID decrescente.
   Dentro `recall` e `promote`: `origin_seq` crescente, poi ID crescente.
   Questo è l'ordine già usato da `evaluate_memory` ed è ora vincolante.
2. **Formula L3.** Per ogni identità con almeno un'attivazione:
   `age = floor((tick_corrente − tick_evento_sorgente) / H)`;
   se `age >= 25` il contributo è zero, altrimenti `2^24 >> age`;
   la somma è saturante. `promote` richiede stato `circuito` e conteggio
   delle attivazioni **strettamente maggiore** di `T`. `archive` richiede
   punteggio zero e stato non già archiviato. `recall` richiede stato
   `archiviato` e una fonte dell'estrazione con `seq > archived_at` che
   attivi l'identità.
3. **Insieme valutato da L3.** Sono valutate tutte le identità con almeno
   un'attivazione registrata, non solo quelle attivate dall'ultima
   estrazione. Un'identità mai attivata non entra nel giudizio.
4. **Nuovi archi e L4.** Una dichiarazione L1 produce la revisione
   `narrate` + `snapshot` se contiene almeno un nodo nuovo o un arco nuovo.
   Un arco già presente (stessa tripla `src_id,dst_id,rel`) non è nuovo e non
   genera una seconda revisione. I nodi già presenti non sono nuovi.
5. **Collocazione AUX e L5.** Gli eventi AUX sono apposti **tra** cicli L0
   completi, al tick dell'ultimo `STATE` completato (tick 0 per la
   configurazione iniziale), e non consumano draw del PRNG. Le osservazioni
   L5 usano una fixture esterna con ID, digest, byte canonici e ancora; non
   sono eventi L0 e non sono ricavate dallo stimolo casuale.

Conseguenze: l'oracolo `preparazione_stress/aux_v2_oracle.py` e il fixture
`aux_v2_small` restano validi; lo scenario principale evita comunque pareggi
L3, quindi non dipende dall'ordine ratificato al punto 1. Le bozze
`manifest_stress_L0_L6_bozza.md` e `oracoli_stress_L0_L6_bozza.md` non sono
state modificate: restano documenti da compilare, non leggi.
