# Oracoli dello stress test L0–L6 — bozza non eseguibile

**Stato: BOZZA NON APPROVATA.** Questo documento separa le proprietà da
verificare dai valori ancora da calcolare. I `TBD` non sono valori attesi;
nessun risultato di prova è attestato. Manifest associato:
`manifest_stress_L0_L6_bozza.md`.

## Preregistrazione e indipendenza

Prima dell'esecuzione, fissare commit post-emendamento, manifest e questa
tabella compilata; registrare i loro hash e lo schema di serializzazione.
Gli hash attesi vanno accompagnati dai byte canonici/preimage e da un metodo
di calcolo indipendente o verificabile. Le funzioni oggetto della prova non
devono essere la sola fonte dell'oracolo. Metodo esatto, revisione e firme:
**TBD**.

| ID | Checkpoint e proprietà da verificare | Valore atteso / prova preregistrata |
| --- | --- | --- |
| O00 | Genesi seed 1042 | hash `4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7`, come `spec/L0_SPEC_v0.4.md` §4; payload e bytes canonici **TBD** |
| O01 | Cicli L0 | ordine/tick e budget draw conforme al regime approvato; sequenza dettagliata **TBD** |
| O02 | Log L0 | `verify` e round-trip JSONL; hash finali e bytes **TBD** |
| O03 | L1 | nodi, archi, `origin_seq`, id, `view_hash` e preimage: **TBD** |
| O04 | L2 | configurazione `k=3`, `tau=5` seq; input, salienze e set attesi: **TBD** |
| O05 | L2 finestra | select ed espulsioni nella finestra, apertura/chiusura, summary e ordine: **TBD** |
| O06 | L3 | configurazione `T=1`, `H=10` tick; attivazioni distinte e promote: **TBD** |
| O07 | L3 decadimento | tick, contributi, somma, giudizio e archive: **TBD** |
| O08 | L3 recall | seq di archive, seq dell'attivazione successiva, `at`, id e origine conservata: **TBD** |
| O09 | L4 | eventi rilevanti, template, `sources`, cutoff e coppie `narrate`/`snapshot`: **TBD** |
| O10 | L4 hash | preimage, hash snapshot e verifica separata di testa e `self_hash`: **TBD** |
| O11 | L5 | osservazioni, modelli, id, contatori, previsioni e `sources`: **TBD** |
| O12 | L5 | assenza update su accordo e update su disaccordo, campi e ancoraggi: **TBD** |
| O13 | L5 secondo ordine | A, T, proiezione e valore confrontato: **TBD** |
| O14 | Snapshot diagnostico | schema canonico esterno, `scenario_id`, seq e hash di testa/output: **TBD**; non è narrazione L4 |
| O15 | Appaiamento L6 | sequenze percettive per ogni coppia, modalità di confronto: **TBD** |
| O16 | Replay L6 | rigenerazione indipendente di ciascuna azione/reward, confronto canonico e hash per entrambi i bracci: **TBD** |
| O17 | Timeout L6 | limite 5 secondi per coppia; meccanismo effettivo, confini della misura e registrazione: **TBD** |
| O18 | Artefatti | schema riga JSONL per coppia, hash e verifica, log unico di verifica: **TBD** |

Il golden O00 è tratto dalla SPEC per la genesi isolata; non è l'hash finale
del log integrato. Tutti gli altri valori concreti rimangono da derivare e
approvare, anche se alcune relazioni sono definite dalle leggi.

## Gate e stati

| Caso | Esito previsto | Dettaglio ancora aperto |
| --- | --- | --- |
| Fixture positiva | PASS del gate se è valida e appaiata | byte esatti, verifiche, motivi e tracciato **TBD** |
| Fixture negativa | PASS del gate se il gate rifiuta la copia alterata per il motivo atteso, prima della valutazione | alterazione, errore codificato e prova che nessuna valutazione sia partita **TBD** |
| Fixture negativa accettata o rifiuto non conforme | FAIL del gate | output di errore **TBD** |
| Run prevista con prerequisito violato | Run invalida; test complessivo non superato | catalogo e rappresentazione delle cause **TBD** |
| Tutte le run valide, gate e oracoli rispettati | PASS complessivo | lista completa degli oracoli e codice di uscita **TBD** |

Il test deliberatamente negativo non è una run invalida. Non eliminare
run dal resoconto né sostituire un fallimento con una media positiva. Il
risultato dello stress test non costituisce una conclusione statistica di
efficacia.

## Dipendenze aperte

1. Approvazione della collocazione degli eventi L2–L5 rispetto ai cicli L0,
   senza assumere che un'eccezione sia già legge.
2. Sequenza completa con tick e seq effettivi, comprese le decisioni apposte;
   definizione di una finestra L2 che produca davvero il summary richiesto.
3. Definizione e verifica dell'oracolo di secondo ordine L5 e del replay
   indipendente L6.
4. Schema canonico degli artefatti, riferimenti al manifest e digest, commit
   esatto dopo l'emendamento.
5. Compilazione e revisione indipendente dei valori mancanti prima della
   firma e prima di qualsiasi esecuzione.

## Calcolo preliminare indipendente degli ID proposti

Questa tabella è una **candidatura alla preregistrazione**, non un risultato
dello stress test né un oracolo approvato. Calcolo indipendente: codifica
UTF-8, JSON senza spazi con chiavi ordinate e SHA-256, eseguito tramite
Python `json.dumps(..., ensure_ascii=False, separators=(',', ':'),
sort_keys=True)` e `hashlib.sha256`. È stato calcolato fuori dal container
come preparazione documentale, non come esecuzione della prova. I valori
valgono solo se i payload proposti sono confermati senza modifiche.

| Soggetto | Preimage canonica UTF-8 | SHA-256 esadecimale |
| --- | --- | --- |
| nodo alfa | `{"key":"alfa","kind":"elemento"}` | `c735556705ac4d4485024861e77e1c1b2d569542c8d5a4359f5ed96334aa8feb` |
| nodo beta | `{"key":"beta","kind":"elemento"}` | `45aba7c6bf7186d413becacb7f88e22665b4b50b25829e668df7435020664ee0` |
| nodo gamma | `{"key":"gamma","kind":"elemento"}` | `781246dd55b21c887bc8c3e7badd05c350dc931eebf5c7b98338ff5f727048dc` |
| nodo delta | `{"key":"delta","kind":"elemento"}` | `0b742e7b07b05582b4afb020715c9fff31e0bd3e107df888b0aa2cb4a49414f1` |
| agente alpha | `{"altro":"agente_alpha"}` | `b7b51403fc2c3c26314fc288bfa3b12b381deeed809870d5b78fca03c8784af3` |
| agente beta | `{"altro":"agente_beta"}` | `8626c5c95ee85f1a1e9eb37d0059b9b7d33a3a9c1d2ef00e28a970f605d6376f` |

Le formule di identità provengono da `spec/L1_SPEC_v0.3.md` §3 e
`notes/L5_legge_sociale.md` L5.3; l'applicazione al protocollo v2 resta
subordinata alla ratifica della bozza coordinata. Hash di eventi, documenti,
manifest e risultati non sono ancora calcolabili da questa tabella: mancano
sequenza, payload canonici completi e commit post-implementazione. Non sono
stati sostituiti con valori ipotetici.

## Candidati verificabili per la fixture L5 di genesi

Per `l5-genesis-01` nel manifest di bozza, la preimage è l'oggetto JSON
esatto lì riportato, senza newline finale: 136 byte, SHA-256
`c66da3195548417fcbcaea078e8bab66b98b539a68ddde6011223800c117fbac`.
Il byte a offset zero-based 91 è `0x31`; sostituendolo con `0x32` la
lunghezza rimane 136 e SHA-256 diventa
`7d4d7aa8059739b6b4c09c15a565254a05b208e662f51b3ee31f81307688f099`.
Calcolo documentale con Python `hashlib.sha256` sui byte UTF-8, non con il
codice del progetto; ripetere indipendentemente e confrontare i byte prima
della preregistrazione definitiva.

Per la positiva, l'osservazione dichiara ordine `0`, proposizione `"1"`,
confidenza `1000` e ancora `[0]`: l'intero risultante dalla conversione
proposta è `1`. L'ancora non prova il contenuto. L'accettazione positiva
richiede anche un manifest coerente, genesi effettiva, collocazione ammessa,
`data` e provenienza corrispondenti: questi presupposti non sono ancora
preregistrati nella loro interezza. Per la negativa il digest originale
rimane registrato e il gate deve emettere `digest_mismatch` prima della
valutazione: è PASS del caso negativo, non una run invalida.

Questi valori coprono soltanto il gate della fixture L5, non i checkpoint
O00–O18 né la verifica indipendente dei 100 abbinamenti L6. Non si deducono
da essi output del modello, hash della catena o successo dello stress test.

## Oracoli condizionali del prefisso L5

I seguenti risultati si derivano a mano dalle regole proposte in §13.4
della bozza coordinata e dalla tabella di seq del manifest, non dalla run.
Sono candidati alla preregistrazione: valgono soltanto se quel prefisso e
la semantica v2 vengono approvati esattamente come descritti.

| Dopo seq | Modello alpha: contatori, `n`, `ultimo` | Modello beta | Decisione attesa |
| ---: | --- | --- | --- |
| 1 | `{1:1}`, 1, 1 | assente | predict alpha `1`, fonti `[1]` |
| 3 | `{1:2}`, 2, 1 | assente | nessun update per l'accordo con predict seq 2 |
| 4 | `{1:2}`, 2, 1 | `{2:1}`, 1, 2 | predict alpha `1`, fonti `[1,3]` |
| 6 | `{1:2,2:1}`, 3, 2 | `{2:1}`, 1, 2 | un update: `at=6`, previsto `1`, osservato `2`, fonti `[5,6]` |
| 8 | `{1:2,2:1}`, 3, 2 | `{2:1}`, 1, 2 | ordine 1 non incrementa i contatori di ordine 0 |

La seq 7 registra l'update, ma non aggiunge osservazioni ai contatori. La
seq 8 non dimostra un modello mentale indipendente; resta aperto O13, il
confronto di secondo ordine, finché non ne sono fissate le API v2 e i byte
da confrontare. Restano altresì da derivare tutti gli hash della catena e
le rappresentazioni esatte delle viste; qui `{1:2,2:1}` è notazione
matematica, non una preimage JSON o un hash atteso.

## Procedura proposta per replay indipendente L6

Da implementare e revisionare separatamente dal produttore dei log, prima
della prova; non è un replay già eseguito:

1. Per ciascun seed `1000..=1099`, verificare separatamente integrità,
   serializzazione canonica e genesi di entrambi i log. Controllare 120
   triplette `SENSE cue → ACTION mossa → SENSE esito`, tick 1–120,
   domini e payload chiusi, senza confonderle con il ciclo L0 principale.
2. Confrontare cue ed esito dei due bracci per ogni posizione e confrontarli
   con il piano offline preregistrato, generato indipendentemente. Validare
   12 cue, tabella bilanciata e copertura dei 40 task di test. La
   implementazione candidata e i digest dei 100 piani sono conservati sotto;
   occorre una revisione indipendente prima della preregistrazione.
3. Nel controllo verificare ogni mossa uguale a zero. Nella memoria,
   ricostruire per ogni cue i conteggi degli esiti **precedenti** alla
   mossa: argmax, parità alla classe minore, zero se nessuna osservazione.
   Confrontare ogni mossa prima di aggiungere l'esito corrente ai conteggi.
4. Ricalcolare ciascun reward come 1000 se mossa uguale a esito, altrimenti
   zero; separare gli 80 task di apprendimento dai 40 di test. Calcolare la
   media test come somma dei 40 milli-reward divisa per 40, e confrontare
   le quantità dichiarate dal produttore, distinguendo somme e medie.
5. Registrare ogni disaccordo, errore e timeout; non escludere coppie.
   Verificare inoltre round-trip byte per byte e hash dei log. La sola
   `fold_run` non dimostra il ricalcolo indipendente delle mosse.

Questa procedura verifica il protocollo L6 sintetico: non dimostra che il
risultato sia causato dai livelli L1–L5 né sostituisce il criterio empirico
distinto della legge L6. I valori candidati delle 100 coppie sono riportati
negli artefatti sotto, ma non sono ancora approvati né risultati di run.

## Calcolo documentale candidato per il corpus L6

È stato effettuato un calcolo preliminare separato dal codice Rust, in
Python con interi a precisione arbitraria e maschere esplicite a 64/128 bit,
seguendo la procedura PCG64 e la costruzione del piano del manifest. Per
ogni seed si sono calcolati piano, mosse da conteggi delle sole osservazioni
precedenti e somme di reward sui 40 task di test. **Non è una run Ariadne,
non è una revisione indipendente del calcolo e non è ancora preregistrazione
approvata.** Il generatore `preparazione_stress/genera_candidati_l6.py` va
revisionato e rieseguito indipendentemente prima del congelamento.

Per seed `1000`, `rule` candidata è `[1,0,2,2,0,1,0,2,1,2,0,1]` e SHA-256
del piano JSON canonico senza newline è
`696f88a6d1ae2bf71a782275ecef782132b897c0e0bc5460da29d001c305d446`.
Le somme candidate sono memoria `40000`, controllo `13000` milli-reward;
le medie corrispondenti sono `1000` e `325`, delta `675` milli-reward.

Su tutti i 100 seed, le somme candidate delle somme-test sono `4000000`
per memoria e `1341000` per controllo. Per permettere la ricostruzione dei
digest aggregati, si definiscono due flussi UTF-8 JSONL con una riga per
seed crescente e LF anche dopo l'ultima riga:

- piani: ogni riga è il piano canonico `{"rule":...,"seed":...,"tasks":...}`
  come nello schema del manifest; SHA-256 candidato del flusso completo
  `64a424d28c16301659dfb21a885fb8e36ea2e823b09f5978826d098369bc6622`;
- righe oracolo: ogni riga è un oggetto canonico con sole chiavi
  `control_test_milli_sum`, `memory_test_milli_sum`, `plan_sha256`, `seed`;
  `plan_sha256` copre i byte del singolo piano **senza** LF. SHA-256
  candidato del flusso completo
  `b08ed60b5ab7dca7d0944fafb71a999da25a3d3a230046a54101b72ab8ac1ec3`.

I 100 piani e le 100 righe sono conservati rispettivamente in
`preparazione_stress/candidati_l6/piani.jsonl` e `oracoli.jsonl`. Per estrarre
il singolo piano, prendere la riga del seed ed escludere il solo LF finale;
il relativo digest è nella riga oracolo. Gli artefatti restano candidati,
non approvati. Nessuno di questi valori è un hash del log prodotto o una
prova dell'efficacia di L1–L5. Mancano revisione indipendente, replay e
risultato effettivo.
