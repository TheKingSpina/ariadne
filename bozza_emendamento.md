# Bozza di emendamento — scenario integrato L0→L6

**Stato: BOZZA NON FIRMATA.** Nessuna scelta contenuta in questa bozza è
attribuita a kingspina. Nessun TBD è stato deciso. Il presente documento non
modifica né sostituisce le leggi firmate.

**Revisione esaminata:** `493041addcd00df2cddfab65cbdd7e4c609be2ad`.
I riferimenti a file e sezioni si intendono a questa revisione. Le decisioni
umane riportate sotto sono quelle trascritte nei documenti del repository;
nessuna conversazione esterna ai documenti è assunta come fonte.

## 1. Elementi già descritti dalle fonti

### L0 — log, eventi e ciclo

- Evento JSONL con `seq`, `tick`, `type`, `payload`, `prev_hash`, `hash`;
  tipi `SENSE | ACTION | STATE`; payload canonico piatto di `i64` e stringhe;
  genesi `STATE` a seq 0/tick 0 con `prng`, `seed`, `state`:
  `spec/L0_SPEC_v0.4.md`, §§2–4.
- Ciclo `SENSE → ACTION → STATE`, tick logico, un draw SENSE per tick e
  zero per la policy; genesi senza draw. Correzioni append-only con `STATE`
  compensativo: stessa fonte, §§4–6.
- PCG64, replay e parser JSONL canonico: stessa fonte, §§7–9. La fonte
  indica anche un golden di genesi per seed 1042 (§4); non lo prescrive
  come seed dello scenario integrato.

### L1 — estrazioni

- Eventi L0 con `nodes` e/o `edges` come stringhe con array JSON canonici;
  nodi con `kind`, `key`; archi con `src_kind`, `src_key`, `dst_kind`,
  `dst_key`, `rel`: `spec/L1_SPEC_v0.3.md`, §§2–3.
- Nel medesimo evento i nodi precedono gli archi; `origin_seq`, id
  canonici e ancoraggio alla testa sono definiti dalla stessa fonte, §§3–6.

### L2 — configurazione ed eventi

- `ACTION` con `l2 = config | select | summary | barrier`; schema chiuso,
  liste codificate come stringhe JSON canoniche: `spec/L2_SPEC_v0.3.md`, §3.
- `config` contiene `k: i64 ≥ 0` e `tau: i64 ≥ 1`; nessun default implicito:
  stessa fonte, §4/L2.2. `select` contiene `at`, `ups`, `downs`; `summary`
  `expulsions`, `text`; `barrier` apre, oppure `phase="close"` chiude la
  finestra: stessa fonte, §§3, 5–6/L2.18–L2.19.
- L2.27 trascrive la firma umana «Lettura A — L2.3/L2.13»: ogni evento che
  dichiara o riprova un arco attiva gli endpoint di quell'evento, con dedup
  per seq: `notes/L2_legge_selezione.md`, L2.27.
- La fonte consolidata segnala che L2.25 abroga la frase di L2.23 relativa
  alle valutazioni durante la finestra: `spec/L2_SPEC_v0.3.md`, §6/L2.25.

### L3 — configurazione ed eventi

- `ACTION` con `l3 = config | promote | archive | recall`; `config` porta
  `T: i64 ≥ 0` e `H: i64 ≥ 1`; gli altri eventi portano `at`, `id`,
  `origin_seq`: `notes/L3_legge_memoria.md`, L3.1–L3.2.
- La firma umana trascritta stabilisce conteggio per eventi distinti,
  soglia `> T`, decadimento in tick e sequela per il recall: stessa fonte,
  epigrafe e L3.3–L3.7. E1/E2 ed E3 sono registrati in L3.3, L3.5 e L3.7;
  E3 dichiara che su identità archiviata `promote` è violazione e solo
  `recall` la riporta in circuito.

### L4 — narrazione e snapshot

- `ACTION` con `l4=narrate`, campi `at`, `text`, `sources`, oppure
  `l4=snapshot`, campi `at`, `self_hash`: `notes/L4_legge_identita.md`,
  L4.1. L4.3 descrive eventi rilevanti e template; L4.4 il documento e il
  preimage di `self_hash`.
- L4.6 specifica coppie narrate/snapshot, cutoff e `sources`; L4.7 richiede
  appaiamento immediato e controllo dell'hash, tollerando il narrate
  solitario soltanto in coda. Le decisioni di Q5/Q7/Q9/Q10 sono attribuite
  alla delega umana «scegli tu»: stessa fonte, introduzione e firma.

### L5 — osservazioni ed eventi

- Un'osservazione è un `SENSE` con `osservazioni` come stringa contenente
  array JSON canonico; ogni membro ha `altro` stringa non vuota e
  `comportamento: i64 ≥ 0`. Array vuoto e membri duplicati sono ammessi;
  `osservazioni` su non-SENSE è schema invalido:
  `notes/L5_legge_sociale.md`, L5.2.
- `ACTION` con `l5=predict`, campi `at`, `id`, `previsto`, `sources`, o
  `l5=model_update`, con in più `osservato`: stessa fonte, L5.1 e L5.10.
  La previsione usa argmax con parità alla classe minore; L5.7 descrive
  una correzione per disaccordo nella finestra predict/osservazione.
- La fonte trascrive la decisione Q4(a): la forma delle osservazioni è
  definita in legge; lo stub del mondo non viene modificato in v0.1:
  stessa fonte, introduzione, L5.2 e firma.

### L6 — protocollo di valutazione

- La legge definisce una run come coppia appaiata di log, 100 coppie con
  seed 1000–1099, 120 task ciascuna: 80 di apprendimento e 40 di test;
  ogni task usa `SENSE {cue}`, `ACTION {mossa}`, `SENSE {esito}`:
  `notes/L6_legge_valutazione.md`, terminologia e L6.2–L6.6.
- Le classi sono 0, 1, 2; reward 1000/0 milli; controllo sempre 0;
  memoria per maggioranza dal proprio log. Tabella bilanciata per run e
  generazione offline PCG64: stessa fonte, terminologia e L6.3–L6.5.
- Wilcoxon una coda, formula, soglie `p < 0.01`, `Δ̄ ≥ 0.30`, nessuna
  violazione; ricalcolo dell'intera sequenza di risposte e reward nel replay;
  limite di 5 secondi per coppia: stessa fonte, L6.7–L6.11.
- La fonte attribuisce la firma dei default e degli 80+40 task alla delega
  umana: stessa fonte, introduzione e firma. `readme.md`, sezione Stato L6,
  registra l'accettazione del ciclo; la casella «umano» nel file della legge
  stessa risulta non spuntata.

## 2. CONFLITTI e tensioni testuali da sottoporre a kingspina

**C1 — CONFLITTO L4, uguaglianza del documento.**
`notes/L4_legge_identita.md`, L4.4, include nel documento `head_hash`
dell'ultimo evento e calcola `self_hash` sul preimage che lo contiene;
la checklist, punto 2(i), richiede il documento intero identico byte per
byte anche per log che differiscono per eventi irrilevanti. La relazione
fra le due prescrizioni è da chiarire: **TBD-C1**.

**C2 — CONFLITTO L0/L6, forma del flusso.**
`spec/L0_SPEC_v0.4.md`, §5, prescrive il ciclo
`SENSE → ACTION → STATE` e un draw per tick; `notes/L6_legge_valutazione.md`,
L6.1–L6.3, prescrive per i task `SENSE → ACTION → SENSE` e zero draw nei
run, con generazione offline. La relazione normativa per uno scenario
integrato è **TBD-C2**.

**C3 — CONFLITTO L6, benchmark casuale.**
`spec/L6_SPEC_v0.2.md`, §4, indica random uniforme `0,50`;
`notes/L6_legge_valutazione.md`, L6.5, indica `333` milli e segnala
esplicitamente la differenza per tre classi. Il trattamento nella prova
integrata è **TBD-C3**.

**C4 — CONFLITTO L6, significato di «memoria L0–L5».**
`spec/L6_SPEC_v0.2.md`, §1, pone la domanda sul miglioramento dovuto alla
memoria accumulata L0–L5; `notes/L6_legge_valutazione.md`, L6.1 e L6.4,
definisce la policy di memoria come replay del proprio log e l'uso delle API
L1–L5 come facoltativo. Che cosa debba dimostrare il percorso integrato è
**TBD-C4**.

**C5 — CONFLITTO L6, timeout e artefatto per run.**
`spec/L6_SPEC_v0.2.md`, §3, dice «timeout per run» e «log unificato JSONL
per ogni run»; `notes/L6_legge_valutazione.md`, terminologia, L6.10–L6.11,
definisce una run come coppia, fissa il tetto per coppia e descrive due
file per coppia come formato unificato. La relazione con il log unico
richiesto per la dimostrazione è **TBD-C5**.

**Risoluzioni già trascritte, non scelte da questa bozza:** L2.27 dichiara
la risoluzione fra L2.5 e L2.3/L2.13; L2.25 dichiara l'abrogazione della
clausola in L2.23; L3.5/E3 dichiara la prevalenza di L3.8 sulla promozione
di un'identità archiviata. Fonti: `notes/L2_legge_selezione.md`, L2.25 e
L2.27; `notes/L3_legge_memoria.md`, L3.5/E3.

## 3. Domande numerate a kingspina

Ogni opzione è una possibilità da decidere, non una raccomandazione. Le
conseguenze descrivono ciò che resta indeterminato in assenza di scelta.

1. **[L0] TBD-01 — Seed e genesi dello scenario principale.** Opzioni:
   scegliere un seed ammesso da L0; collegarlo a un seed L6; altra regola.
   Senza scelta non è fissata la genesi del log principale.
2. **[L0] TBD-02 — Sequenza degli eventi e tick del log principale.** Opzioni:
   specificare una sequenza completa; specificare una regola generatrice;
   altra formulazione. Senza scelta non si può costruire la stessa prova.
3. **[L0/L6] TBD-C2 — Rapporto fra ciclo L0 e task L6.** Opzioni: chiarire
   un perimetro distinto; emendare esplicitamente i vincoli interessati;
   altra risoluzione firmata. Senza scelta C2 rimane aperto.
4. **[L1] TBD-03 — Nodi, archi e collocazione delle estrazioni.** Opzioni:
   indicare esplicitamente `kind`, `key`, endpoint, `rel` e ordine; indicare
   una regola deterministica; altra formulazione. Senza scelta la vista L1
   e i valori attesi non sono determinati.
5. **[L2] TBD-04 — Valori e posizione di `k` e `tau`.** Opzioni: scegliere
   una configurazione nel dominio L2.2; definire configurazioni successive;
   altra scelta conforme. Senza scelta non è fissata la selezione.
6. **[L2] TBD-05 — Decisioni e finestra offline da esercitare.** Opzioni:
   specificare select, apertura/chiusura e condizioni per summary;
   specificare un sottoinsieme; altra scelta. Senza scelta non è definita
   la copertura L2 della prova integrata.
7. **[L3] TBD-06 — Valori e posizione di `T` e `H`.** Opzioni: una
   configurazione nel dominio L3.2; configurazioni successive; altra scelta
   conforme. Senza scelta non è fissato il giudizio L3.
8. **[L3] TBD-07 — Transizioni L3 da esercitare e relative attivazioni.**
   Opzioni: specificare quali fra promote/archive/recall e la loro sequenza;
   altra scelta conforme a L3.3–L3.8. Senza scelta non è fissata la copertura
   delle transizioni né la sequela eventuale.
9. **[L4] TBD-08 — Revisioni narrative e checkpoint.** Opzioni: indicare
   gli eventi rilevanti e le coppie da verificare; indicare una regola di
   selezione degli eventi; altra scelta conforme. Senza scelta non sono
   fissati gli output narrativi attesi dello scenario.
10. **[L4] TBD-C1 — Documento identico e `head_hash`.** Opzioni: chiarire
    l'oggetto dell'uguaglianza; emendare la checklist; altra risoluzione
    firmata. Senza scelta C1 rimane aperto.
11. **[L5] TBD-09 — Osservazioni concrete.** Opzioni: indicare label,
    comportamenti e ordine delle osservazioni valide; indicare una regola
    deterministica; altra scelta conforme. Senza scelta non è determinato
    il modello sociale da verificare.
12. **[L5] TBD-10 — Previsione, disaccordo e secondo ordine.** Opzioni:
    specificare gli eventi e gli output da controllare, inclusa o meno una
    finestra con update; altra scelta. Senza scelta non è definita la
    copertura richiesta per questi aspetti L5.
13. **[L0–L6] TBD-11 — Topologia dei log e collegamento fra livelli.**
    Opzioni: indicare un log principale e log L6 distinti con un criterio
    di collegamento; altra topologia esplicitamente definita. Senza scelta
    non è definito cosa significhi una sola esecuzione integrata.
14. **[L6] TBD-C4 — Contributo di L1–L5 al confronto.** Opzioni: specificare
    uso obbligatorio delle viste; mantenere il protocollo L6.4 come descritto;
    altra risoluzione firmata. Senza scelta C4 rimane aperto e non è definita
    la conclusione ammessa sul contributo L1–L5.
15. **[L6] TBD-C3 — Benchmark casuale.** Opzioni: chiarire il valore e il
    suo perimetro; emendare esplicitamente una formulazione; altra
    risoluzione firmata. Senza scelta C3 rimane aperto.
16. **[L6] TBD-C5 — Timeout, log per run e log unico della verifica.**
    Opzioni: specificare l'unità temporale e le relazioni tra i file di
    evidenza; altra risoluzione firmata. Senza scelta C5 rimane aperto.
17. **[L0–L6] TBD-12 — Oracoli e criteri di PASS/FAIL integrati.** Opzioni:
    specificare valori, invarianti e controlli per ogni passaggio; altra
    formulazione verificabile. Senza scelta non è determinato l'esito della
    prova integrata.
18. **[L6] TBD-13 — Replay delle decisioni.** Opzioni: specificare come
    ricalcolare e confrontare l'intera sequenza per ciascun braccio secondo
    L6.9; altra formulazione firmata. Senza scelta non è definita la prova
    integrata di identità delle decisioni.
19. **[L0–L6] TBD-14 — Ambiente e provenienza dell'esecuzione.** Opzioni:
    indicare versioni e digest da fissare, artefatti e loro conservazione;
    altra specifica verificabile. Senza scelta l'ambiente riproducibile dello
    scenario integrato non è fissato nelle fonti citate.
20. **[L0–L6] TBD-15 — Gate negativo e positivo della prova integrata.**
    Opzioni: specificare input, regola di blocco e output attesi per entrambi;
    altra definizione verificabile. Senza scelta i due casi del gate
    integrato non sono determinati.

## 4. Testo della bozza di emendamento — da completare, non firmato

> **Perimetro.** La dimostrazione integrata L0→L6 usa la topologia di log
> **TBD-11**, la genesi **TBD-01**, la sequenza e i tick **TBD-02** e
> l'ambiente **TBD-14**. La relazione fra ciclo L0 e task L6 è **TBD-C2**.
>
> **Dati e configurazioni.** Le estrazioni L1 sono **TBD-03**; la
> configurazione L2 è **TBD-04**, con copertura **TBD-05**; la configurazione
> L3 è **TBD-06**, con transizioni **TBD-07**. Le revisioni L4 sono
> **TBD-08** e il chiarimento sull'uguaglianza del documento è **TBD-C1**.
> Le osservazioni L5 sono **TBD-09**, con copertura **TBD-10**.
>
> **Valutazione.** Il contributo delle viste L1–L5 è **TBD-C4**; il
> benchmark casuale è disciplinato da **TBD-C3**; timeout e organizzazione
> dei log da **TBD-C5**. La verifica del replay delle decisioni è **TBD-13**.
>
> **Verifica e output.** Gate negativo e positivo: **TBD-15**. Oracoli,
> valori attesi e criteri di esito integrati: **TBD-12**. Nessuna parte
> contenente TBD costituisce una scelta o un'autorizzazione all'esecuzione.
>
> **Firma kingspina:** non apposta. **Data:** TBD.

## 5. Elenco completo dei TBD aperti

`TBD-01`, `TBD-02`, `TBD-03`, `TBD-04`, `TBD-05`, `TBD-06`, `TBD-07`,
`TBD-08`, `TBD-09`, `TBD-10`, `TBD-11`, `TBD-12`, `TBD-13`, `TBD-14`,
`TBD-15`, `TBD-C1`, `TBD-C2`, `TBD-C3`, `TBD-C4`, `TBD-C5`, e la data
dell'eventuale firma (**TBD**). Le risposte e le proposte successive sono
riportate nell'appendice: non costituiscono approvazione dell'emendamento;
gli aspetti non ancora specificati restano TBD.

## Appendice — risposte di kingspina e proposta successiva

**Stato: testo da incorporare e approvare; nessun via libera di esecuzione.**
Questa appendice registra quanto comunicato da kingspina senza modificare le
leggi firmate né presentare la bozza come approvata. Le domande precedenti
sono conservate come traccia storica, non come affermazione che tutte siano
prive di risposta. I dettagli non specificati restano TBD.

1. Seed L0 principale `1042`; seed L6 separati `1000–1099`.
2. Sequenza deterministica fissata in un manifest, con tick e ordine
   espliciti, genesi senza draw e un ciclo L0 valido per tick. Sequenza
   esatta e manifest immutabile: **TBD**.
3. Ciclo L0 e task L6 distinti, collegati con `scenario_id` e identificativi
   correlati, senza imporre una sequenza unica incompatibile. Formato e
   valori esatti degli identificativi: **TBD**.
4. Nodi e archi generati deterministicamente usando soltanto `kind`, `key`
   e `rel` ammessi dalle specifiche; ordine e valori nel manifest. Valori e
   regola esatti: **TBD**.
5. Configurazione L2 una volta prima degli eventi di selezione: `k=3` e
   `tau=5`. Nella proposta successiva kingspina chiarisce che `tau=5` è
   espresso in **seq, non in tick**, coerentemente con
   `spec/L2_SPEC_v0.3.md`, §5/L2.4. Posizione esatta nel manifest: **TBD**.
6. Copertura L2: `select`, apertura e chiusura finestra, `summary`, confini
   temporali e condizioni previste dalla legge. Fixture e oracoli: **TBD**.
7. Configurazione L3 `T=1`, `H=10` tick prima delle transizioni. Posizione
   esatta nel manifest: **TBD**.
8. Esercitare `promote`, `archive`, `recall`, con soglia superata mediante
   eventi distinti, decadimento previsto e sequela prescritta. Sequenza e
   valori attesi esatti: **TBD**.
9. Narrare e verificare snapshot dopo promozione, archiviazione e richiamo;
   fonti e cutoff nel manifest. **Proposta successiva di kingspina:** dopo
   l'aggiornamento L5 registrare uno *snapshot diagnostico*, senza chiamarlo
   narrazione L4 né attribuirgli una regola L4 vigente. Struttura, contenuto,
   provenienza e verifica di tale snapshot: **TBD**.
10. Proposta di emendamento della checklist L4: confrontare campi semantici
    escludendo `head_hash` e `self_hash`, verificando separatamente ciascun
    hash. Formula normativa e procedura di verifica: **TBD**.
11. Osservazioni fisse: `agente_alpha` comportamento `1`, `agente_beta`
    comportamento `2`; ordine alternato soltanto secondo una sequenza
    preregistrata. Sequenza precisa e numero di osservazioni: **TBD**.
12. Coprire previsione, accordo, disaccordo, `model_update` e secondo ordine;
    output attesi calcolati applicando la legge, non inventati a mano.
    Metodo di calcolo indipendente e oracoli esatti: **TBD**.
13. Log L0 principale separato dai due log L6, uno per braccio, collegati
    tramite `scenario_id` e ID di run/interazione. Codifica e collocazione
    dei collegamenti: **TBD**.
14. Lo stress test attraversa L1–L5; L6 conserva la policy definita dalla
    propria legge. Il risultato dimostra integrazione e invarianti, non che
    L1–L5 causino un miglioramento di L6. Modalità esatta di attraversamento
    e invarianti: **TBD**.
15. Benchmark casuale `333` milli per tre classi; proposta di emendare
    esplicitamente la formulazione `0,50` limitando la scelta alla prova
    concordata. Formulazione finale e approvazione normativa: **TBD**.
16. Timeout massimo di 5 secondi per coppia; log separati dei bracci e una
    riga JSONL di riepilogo per coppia con esiti, delta, validità e hash
    degli artefatti. Schema esatto, implementazione del timeout e relazione
    con il log di verifica: **TBD**.
17. PASS solo se tutti gli invarianti e gli oracoli preregistrati sono
    rispettati; ogni violazione dà FAIL o run invalida. Nessuna conclusione
    statistica di efficacia da questo stress test. Elenco di oracoli e
    distinzione operativa FAIL/run invalida: **TBD**.
18. Replay deterministico dell'intera sequenza di azioni e reward per ogni
    braccio; confronto di output canonici e hash con i log originali.
    Procedura e oracolo di confronto esatti: **TBD**.
19. Fissare il commit esatto **dopo l'emendamento**; solo Docker con
    `rust:1.85.1@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4`;
    conservare manifest, log e artefatti con i rispettivi hash. Commit
    successivo, percorsi e formato della registrazione degli hash: **TBD**.
20. Gate positivo con fixture valida e appaiata; gate negativo mediante
    alterazione di un evento o della sequenza appaiata, da rifiutare prima
    della valutazione, con motivo registrato e zero run valide. Fixture,
    alterazione e condizioni esatte: **TBD**.

**Vincolo ribadito da kingspina:** nessuna modifica al progetto durante lo
stress test, nessun fallback sull'host. L'esecuzione resta sospesa fino
all'approvazione dell'emendamento; la presente appendice non è tale
approvazione.
