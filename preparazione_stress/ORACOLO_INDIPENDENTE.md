# Oracolo indipendente del replay AUX v2

**Data:** 5 novembre 2026. Mandato: `.opencode/mandati/2026-11-05-wp3-oracolo.md`.
**Stato:** implementato e verde; non è una prova di verità né di efficacia causale.

## Perché esiste

`V2Log::from_jsonl` ricostruisce il log rigenerando ogni operazione con le
stesse funzioni usate per produrla (`extract`, `close_selection_window`,
`evaluate_memory`). Una suite verde dimostra **coerenza interna del replay**,
non conformità al contratto né correttezza delle decisioni L1–L4.

Questo oracolo è una **seconda implementazione**, scritta dal profilo
(`AUX_V2_PROFILO_IMPLEMENTATO.md`) e non importata dal codice di produzione.
Il fixture atteso è congelato prima del test: il driver Rust lo legge dal disco
e ne confronta i byte; non lo rigenera.

## Componenti

| Percorso | Ruolo |
|---|---|
| `preparazione_stress/aux_v2_oracle.py` | generatore e verificatore indipendente (solo `json`, `hashlib`, `argparse`, `pathlib`) |
| `preparazione_stress/fixtures/aux_v2_small/expected.jsonl` | fixture congelato, 121 eventi, SHA-256 `690831b39906070c40eff25e64092d4c8f005a04d017ed4cb347d514f98cd0b4` |
| `preparazione_stress/fixtures/aux_v2_small/expected.sha256` | digest del fixture |
| `preparazione_stress/fixtures/aux_v2_small/manifest.json` | parametri, tabella seq, valori attesi, digest dei negativi |
| `preparazione_stress/fixtures/aux_v2_small/negative/` | 11 fixture negativi preregistrati con l'errore atteso |
| `tests/aux_v2_independent.rs` | driver: costruisce lo scenario con la sola API pubblica, confronta i byte, esercita i negativi |

L'oracolo non importa `ariadne`, non chiama il backend Rust e non usa
`V2Log::from_jsonl`, `extract`, `close_selection_window`,
`evaluate_memory`, `node_id`, `Json` o `to_jsonl`. Reimplementa PCG64, JSON
canonico, `seal` SHA-256, identità dei nodi e la macchina a stati L1–L4.

## Scenario `small`

Seed `1042`, `k=1`, `tau=5`, `T=1`, `H=1`. Nodi `elemento/alfa` e
`elemento/beta`, arco `alfa→beta` con `rel="conosce"`.

Sequenza: genesi, configurazione L2/L3, tick 1 con estrazione e selezione,
tick 2 con promozione di `alfa` ed espulsione di `beta`, 24 cicli L0, tick 27
con archivio di `alfa` e promozione di `beta`, tick 28 con richiamo di `alfa`.
Totale 121 eventi (seq 0–120). La tabella completa è nel `manifest.json` e nel
mandato.

Valori di controllo:

- hash genesi `4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7`;
- `alfa` `c735556705ac4d4485024861e77e1c1b2d569542c8d5a4359f5ed96334aa8feb`,
  `beta` `45aba7c6bf7186d413becacb7f88e22665b4b50b25829e668df7435020664ee0`;
- tick 1: parità di punteggio e di origine, vince l'ID minore → `ups=[beta]`,
  `downs=[]`; `alfa` escluso non è un'espulsione;
- tick 2: `score(alfa,20)=18.874.368`, `score(beta,20)=2.097.152` →
  `ups=[alfa]`, `downs=[beta]`, `expulsions=[3]`, `text=beta`;
- tick 27: `score(alfa)=0` con status `memoria_lunga` → `archive`;
- tick 28: `recall` di `alfa` perché la fonte 114 > 108 e la attiva.

## Regole verificate

**Catena e forma.** Ordine delle chiavi `seq,tick,type,payload,prev_hash,hash`;
preimmagine `seq␟tick␟type␟payload␟prev_hash` con SHA-256 minuscolo;
`prev_hash[0]` a 64 zeri; JSONL con LF finale, senza CR e senza righe vuote;
payload canonico (chiavi ordinate, niente float/bool/null, escape minimi);
AUX con esattamente sei chiavi e `protocol_version="2"`.

**L1.** Una sola estrazione per fonte; `data.nodes` prima di `data.edges`;
endpoint dichiarato nello stesso evento o già presente; identità
`SHA-256({"key":K,"kind":T})`; `origin_seq` fissato alla prima dichiarazione;
attivazione deduplicata per `(identità, seq sorgente)`; archi deduplicati per
tripla con origine della prima dichiarazione; revisione L4 solo per le nuove
dichiarazioni.

**L2.** Configurazione L2/L3 consecutiva; `barrier open` + `select`
consecutivi; candidati = attivi precedenti ∪ identità attivate dalla fonte;
punteggio `Σ 2^24 >> min(floor((extraction−activation)/tau),24)`; ordine per
punteggio decrescente, origine crescente, ID crescente; primi `k`;
`ups`/`downs` come **ID**; espulsioni solo per chi era attivo prima;
`barrier close` + `summary` consecutivi; `expulsions` = origini deduplicate e
crescenti; quattro revisioni L4 dopo il summary; nessuna chiusura implicita.

**L3.** Stato per identità; `promote` solo da `circuito` e con conteggio
attivazioni **strettamente** maggiore di `T`; `archive` con punteggio zero;
`recall` solo da `archiviata` con fonte `seq > archived_at` che attiva
l'identità; ordine archive → recall → promote; L4 dopo ogni transizione.

**L4.** Ogni `narrate` seguito da `snapshot`; `self_hash` calcolato su
`{head_hash: hash(narrate), schema_version:"2", statements}` e **non** sullo
hash dello snapshot; statements cumulativi; i documenti di vista usano l'hash
corrente e si calcolano senza il proprio campo hash.

## Cosa copre e cosa no

**Copre:** catena e canonicità; transizioni L1–L4 dello scenario `small`;
tie-break per ID; espulsione reale vs esclusione; soglia `T`; soglia 25 tick;
archiviazione e richiamo; prefissi troncati; alterazioni con catena
ricalcolata (`Mismatch`), alterazione di un byte (`Chain`), CRLF/LF/riga vuota
e `protocol_version` (`Schema`); accettazione del prefisso aperto dopo
`select`.

**Non dimostra:**

- **L6 / efficacia causale**: nessun replay L1–L4 prova che la memoria migliori
  le prestazioni; L6 resta un esperimento separato;
- **verità dei fatti**: il digest autentica i byte, non il contenuto; una
  fixture può essere fabbricata;
- **autenticità** di persone o agenti, né identità del sé;
- **crash consistency, concorrenza, durabilità** su disco: il clone-and-commit
  è in memoria e il replay è offline;
- **assenza di bug nell'oracolo**: due implementazioni riducono il rischio, non
  lo eliminano; la tabella attesa è derivata a mano dal profilo e va rivista da
  una persona;
- **copertura totale**: un fixture non esaurisce le combinazioni possibili;
- **correttezza del contratto**: l'oracolo verifica la conformità al profilo
  scritto, non che il profilo sia giusto o completo.

## Ambiguità dichiarate

**Ratificate il 5 novembre 2026.** Le scelte sotto sono ora parte del
contratto in `AUX_V2_PROFILO_IMPLEMENTATO.md`, sezione «Decisioni ratificate».
L'oracolo e il fixture restano validi; lo scenario principale evita comunque
pareggi L3 e non dipende dall'ordine interno.

Il profilo non fissa alcuni punti; l'oracolo e il fixture adottano
un'interpretazione esplicita:

1. **Ordine L3 nella stessa classe** (più `archive` o più `promote` nello
   stesso giudizio): il fixture principale evita i pareggi. L'ordine interno
   resta da ratificare e non è imposto dall'oracolo.
2. **Formula L3**: adottato l'adattamento di L2 (`2^24`, `tau→H`, `seq→tick`,
   soglia 25). Non è scritto letteralmente nel profilo.
3. **Insieme valutato da L3**: tutte le identità con almeno un'attivazione,
   non solo quelle attivate dall'ultima estrazione. Il fixture non dipende da
   identità mai attivate.
4. **Nuovi archi e L4**: il fixture segue l'implementazione, che narra anche i
   nuovi archi, mentre il profilo parla di «prime dichiarazioni». Va ratificato.

## Riesecuzione

```sh
python3 preparazione_stress/aux_v2_oracle.py build   # rigenera e congela
python3 preparazione_stress/aux_v2_oracle.py verify  # non riscrive nulla
cargo test --locked --quiet --test aux_v2_independent
```

`verify` ricalcola lo scenario dal profilo, confronta i byte col fixture,
verifica catena, digest, manifest e i negativi. Se il fixture manca o il
digest non coincide, il test Rust fallisce: non esiste un ramo che rigenera
l'atteso durante il test.

## Nota sulla generazione

La prima generazione dei fixture negativi concatenava le righe senza `\n`,
producendo un JSON non valido (`Schema` invece di `Mismatch`/`Chain`). Il
difetto era nel **generatore dei negativi**, non nel fixture principale né nel
replay. Corretto in `write_negative_fixtures` con `"\n".join(...)`; fixture e
manifest rigenerati e tutti i test verdi. Il fixture principale non è cambiato
(digest invariato).
