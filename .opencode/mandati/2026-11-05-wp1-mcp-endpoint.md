# Mandato WP1 — MCP stabile nel container, errori chiari, tool di lettura del grafo

Data: 2026-11-05. Emesso dall'orchestratore su richiesta umana.
Stato: da implementare. Nessun commit, nessun push, nessun deploy.

## Contesto

Ariadne ha già:

- `mcp/ariadne_server.py`: server MCP **stdio** (JSON-RPC) con 11 tool. I tool
  persistenti chiamano l'API HTTP della dashboard (`ARIADNE_STORE_URL`,
  `ARIADNE_STORE_HOST`, `ARIADNE_WRITE_TOKEN`). I tool di verifica chiamano il
  binario Rust `ariadne_mcp_backend` via stdin/stdout.
- `dashboard/server.py`: dashboard + archivio SQLite append-only su porta 8090.
- `dashboard/Dockerfile`: costruisce solo `ariadne_mcp_backend` ed esegue la
  dashboard. Il container `ariadne-dashboard` è attivo con volume
  `ariadne-dashboard-data` e token in `/data/.write-token`.
- Il trasporto HTTP MCP era un proxy separato sul Mac, non persistente: ora non
  ascolta nulla su `127.0.0.1:8080`. Va sostituito da un endpoint HTTP dentro
  lo stesso container, avviato e fermato insieme alla dashboard.

Vincoli di progetto: log append-only; provenienza ≠ verità; le memorie testuali
sono dichiarazioni del chiamante, non fatti; non presentare L6 come prova
causale di L1–L5; nessuna modifica alle leggi/SPEC firmate.

## Obiettivo WP1

1. Esporre MCP **Streamable HTTP** su `127.0.0.1:8080/mcp` dallo stesso
   container della dashboard, con avvio/arresto coordinati.
2. Rendere comprensibili gli errori su nodi e archi, senza cambiare il
   comportamento del protocollo AUX v2.
3. Aggiungere tre tool MCP di sola lettura sul grafo salvato.

## 1. Refactor del protocollo MCP (stdio invariato)

In `mcp/ariadne_server.py` estrarre una classe `Protocol` con stato
`initialized` e un metodo `handle(request: dict) -> dict | None` che restituisce
la risposta JSON-RPC completa, oppure `None` per le notifiche. `main()` deve
continuare a leggere righe da stdin e a scrivere solo JSON-RPC su stdout.
Mantenere: limiti 2.000.000 byte, versioni `2024-11-05`/`2025-03-26`,
`initialize`, `ping`, `tools/list`, `tools/call`, errori -32700/-32600/-32601/
-32602/-32002, `initialize` una sola volta per istanza.

Aggiornare `serverInfo.version` a `0.5.0` e le istruzioni iniziali per citare i
nuovi tool e la distinzione fra operazioni temporanee (`persisted: false`) e
salvate (`persisted: true`).

## 2. Nuovo trasporto `mcp/http_server.py`

Server HTTP locale, solo ambiente di test, senza pretese di produzione.

- `POST /mcp`: accetta un oggetto JSON-RPC o un array (batch). Content-Type
  `application/json` obbligatorio; corpo massimo 2.000.000 byte. Risposta
  `application/json` con l'oggetto/array delle risposte; se tutte le richieste
  sono notifiche rispondere `202` senza corpo. Se il client accetta
  `text/event-stream` e non `application/json`, rispondere come singolo evento
  SSE `event: message\ndata: <json>\n\n`.
- `initialize` crea una sessione e restituisce l'header `Mcp-Session-Id`
  (UUID esadecimale). Sessioni successive usano lo stesso `Protocol`.
  `Mcp-Session-Id` sconosciuto → `404`. Richieste senza header usano un
  `Protocol` anonimo condiviso (compatibilità con client che ignorano l'header).
  `DELETE /mcp` termina la sessione (`204`); `GET /mcp` → `405` con `Allow`.
- `GET /healthz`: JSON con `status`, `sessions`, `protocol_versions`,
  `initialized`; nessun token o dato sensibile.
- CORS: `ARIADNE_MCP_ALLOW_ORIGIN` (default `*`, solo test), metodi
  `POST, GET, DELETE, OPTIONS`, header `Content-Type, Accept, Mcp-Session-Id`,
  esporre `Mcp-Session-Id`. `OPTIONS` → `204`.
- Header `Host` non controllato come nella dashboard: endpoint locale di test.
- Bind default `127.0.0.1:8080`, opzioni `--host`/`--port`.
- Log diagnostici solo su stderr; mai su stdout.

## 3. Avvio coordinato — `dashboard/supervisor.py`

- Legge il token con `load_write_token` **una volta** e lo passa a entrambi i
  processi via `ARIADNE_WRITE_TOKEN`.
- Esporta per il server MCP: `ARIADNE_STORE_URL=http://127.0.0.1:8090`,
  `ARIADNE_STORE_HOST=127.0.0.1:8090`.
- Avvia `dashboard/server.py --host 0.0.0.0 --port 8090 --database
  ${ARIADNE_DATABASE:-/data/ariadne.sqlite3}`, attende che la porta risponda,
  poi avvia `mcp/http_server.py --host 0.0.0.0 --port 8080`.
- Su SIGTERM/SIGINT termina entrambi i figli e attende. Se un figlio esce,
  termina l'altro ed esce con codice non zero. Prefissa i log con
  `dashboard:`/`mcp:`. Non usare `shell=True`.
- `dashboard/Dockerfile`: `EXPOSE 8080 8090`, `CMD ["python3",
  "dashboard/supervisor.py"]`. Aggiungere `dashboard/docker-compose.yml`
  (context `..`) con un solo servizio, porte `127.0.0.1:8080:8080` e
  `127.0.0.1:8090:8090`, volume `ariadne-dashboard-data:/data`, immagine
  `ariadne-dashboard-local`, `restart: unless-stopped`.
- Aggiornare `mcp/README.md` e `dashboard/README.md`: endpoint HTTP, comandi di
  avvio/arresto coordinati, token, limiti (CORS `*`, nessuna autenticazione
  client, solo test locale).

## 4. Errori comprensibili (Rust + Python)

In `src/aux_v2/protocol.rs` migliorare solo `Display` di `Error`, mantenendo
le varianti e `Debug`:

- `Schema` → struttura o valore non valido
- `Chain` → catena hash o sequenza non valida
- `Phase` → operazione fuori dalla fase consentita
- `Config` → configurazione L2/L3 assente o non valida
- `Reference` → riferimento a evento, nodo o arco inesistente
- `WindowState` → finestra di selezione L2 in stato non valido
- `IncompleteBatch` → batch incompleto
- `Mismatch` → contenuto non corrispondente al replay
- `NoModel` → modello sociale assente
- `DigestMismatch` → digest fixture non corrispondente
- `Overflow` → valore fuori intervallo

In `src/bin/ariadne_mcp_backend.rs`, per `extract_v2`, **prima** di chiamare
`V2Log::extract`, validare e restituire errori specifici che nominano l'indice e
l'elemento:

- nessun nodo e nessun arco → `extract: servono almeno un nodo o un arco`
- `rel` non valida → `extract: arco #N rel "<valore>" non valida; ammessi solo
  a-z, 0-9 e _`
- endpoint non dichiarato né presente nel log → `extract: arco #N: endpoint
  <kind>/<key> non dichiarato in nodes e assente nel registro`
- nodo duplicato in `nodes` o arco duplicato in `edges` → non è un errore:
  aggiungere una voce a `warnings` (array di stringhe) nel risultato.
- nodo già presente nel registro → consentito, nessun warning obbligatorio.

Sul replay fallito, restituire un messaggio con `Display` dell'errore e, quando
possibile, la prima `seq` in cui la catena strutturale (seq/prev_hash/hash)
si rompe: `v2 replay rejected a seq N: ...`. Non modificare la semantica di
`from_jsonl`.

In Python, ogni errore di `tools/call` deve iniziare con il nome del tool:
`<tool>: <messaggio>`. Aggiungere ai risultati:

- `ariadne_v2_create`, `ariadne_v2_extract`: `"persisted": false`
- `ariadne_add_graph`: `"persisted": true` (oltre a id/version/events/head_hash)
- `ariadne_find_nodes`, `ariadne_node_neighbors`, `ariadne_has_edge`:
  `"persisted": true, "read_only": true`

## 5. Nuovi tool di sola lettura

Tutti operano su un registro v2 salvato e leggono `/api/logs/<id>/views/graph`.
Non scrivono nulla. `id` con il solito pattern `[a-zA-Z0-9_-]{1,64}`.

- `ariadne_find_nodes`: `{id, query?, kind?, limit?}`. `query` è sottostringa
  case-insensitive su `key`; `kind` è uguaglianza esatta; `limit` 1..=200
  (default 50). Risposta: `{id, persisted:true, read_only:true, total,
  nodes:[{id,kind,key,origin_seq,degree}]}` dove `degree` conta archi entranti
  e uscenti.
- `ariadne_node_neighbors`: `{id, kind, key, direction?}` con `direction` in
  `out|in|both` (default `both`). Risposta: `{id, persisted:true,
  read_only:true, node:{...}, outgoing:[{rel, origin_seq, node:{id,kind,key,
  origin_seq}}], incoming:[...]}`. Nodo assente → errore che nomina
  `<kind>/<key>`.
- `ariadne_has_edge`: `{id, src_kind, src_key, dst_kind, dst_key, rel?}`.
  Risposta: `{id, persisted:true, read_only:true, exists:bool, edges:[{rel,
  origin_seq, src:{kind,key}, dst:{kind,key}}]}`. Endpoint assenti → `exists:
  false` con `edges: []`, non errore.

Aggiungere esempi sintetici solo dove utili; gli esempi non devono essere
eseguiti né salvati. Aggiornare la lista dei tool nei test esistenti.

## 6. Test richiesti

Aggiornare/aggiungere in `mcp/test_ariadne_server.py` e
`dashboard/test_dashboard.py`:

- trasporto HTTP: initialize con `Mcp-Session-Id`, tools/list, tools/call,
  sessione sconosciuta 404, DELETE, batch, notifica → 202, `/healthz`, limite
  body, `GET /mcp` 405;
- `ariadne_add_graph`: archi multipli, ciclo `a→b→a`, auto-arco, nodo esistente,
  nodo duplicato, arco duplicato;
- errori: endpoint mancante (messaggio con kind/key), `rel` con maiuscole o
  trattino (messaggio con valore), registro inesistente per i tool di lettura;
- `ariadne_find_nodes` (query/kind/limit), `ariadne_node_neighbors`
  (in/out/both), `ariadne_has_edge` (esiste/non esiste/rel filtro);
- persistenza: riaprire lo `Store` e vedere i nodi/archi aggiunti;
- nessuna regressione sui test esistenti.

Comandi di verifica (dalla radice del repository):

```sh
cargo build --locked --bin ariadne_mcp_backend
cargo test --locked --quiet
python3 -m unittest -v mcp.test_ariadne_server dashboard.test_dashboard
```

Non avviare/fermare container, non ricostruire l'immagine, non eseguire lo
stress test L0–L6, non modificare `manifest_stress_L0_L6_bozza.md` né gli
oracoli. Se un comando non è disponibile nell'ambiente, riportarlo invece di
aggirarlo.

## 7. Consegna attesa

Riportare all'orchestratore:

- file modificati/creati;
- comandi eseguiti e loro esito (test superati/falliti, output rilevante);
- decisioni prese su punti ambigui;
- limiti noti e prove non eseguite;
- conferma esplicita: nessun commit, nessun push, nessun container toccato.
