# Ariadne MCP — adapter locale

Questo adapter espone Ariadne tramite MCP JSON-RPC in due trasporti:

- **stdio** (`mcp/ariadne_server.py`): righe JSON-RPC su stdin/stdout, per
  client MCP che avviano un processo.
- **Streamable HTTP** (`mcp/http_server.py`): `POST /healthz` e `POST /mcp`
  dettagli più sotto, usato nel container della dashboard. Solo ambiente di
  test: nessuna autenticazione client e CORS aperto di default.

I tool di verifica accettano log e fixture forniti dal client; i tool
persistenti chiamano l'API HTTP locale della dashboard. Le memorie testuali
sono dichiarazioni append-only separate dagli eventi L0–L5. Non è un servizio
multiutente o pronto per l'esposizione pubblica.

## Endpoint HTTP nel container della dashboard

Con `docker compose` (o il supercluster `dashboard/supervisor.py`, vedi
`dashboard/README.md`) lo stesso processo container espone:

- `http://127.0.0.1:8080/mcp` — `POST /mcp` per initialize, ping,
  tools/list e tools/call; `DELETE /mcp` chiude la sessione; `GET /mcp`
  risponde 405. L'`initialize` restituisce l'header `Mcp-Session-Id`;
  richieste successive possono ripeterlo. Richieste senza header usano un
  protocollo condiviso compatibile con client che ignorano l'header.
- `http://127.0.0.1:8080/healthz` — stato: `status`, `sessions`,
  `protocol_versions`, `initialized`; nessun token o dato sensibile.
- Corpo massimo 2.000.000 byte; `Content-Type: application/json`
  obbligatorio; messaggio singolo o batch; se tutte le richieste sono
  notifiche la risposta è `202` senza corpo. Se il client accetta solo
  `text/event-stream`, la risposta è un evento SSE.
- CORS da `ARIADNE_MCP_ALLOW_ORIGIN` (default `*`, solo test) e nessun controllo
  dell'header `Host`: usare solo in locale o in una rete di test fidata.
- Sessioni sconosciute → 404. Diagnostica solo su stderr.

## Avvio coordinato

Il container unico può essere avviato da:

```sh
docker compose -f dashboard/docker-compose.yml up -d --build
docker compose -f dashboard/docker-compose.yml down
```

oppure, fuori da Docker, con `python3 dashboard/supervisor.py` che avvia
dashboard ed endpoint HTTP e li arresta insieme (vedi `dashboard/README.md`).

In alternativa lo stdio resta disponibile:

```sh
cargo build --locked --bin ariadne_mcp_backend
python3 mcp/ariadne_server.py
```

configurando in `mcpServers` un client con `"command": "python3",
"args": [.../]` come da esempio precedente. Le variabili `ARIADNE_MCP_BACKEND`,
`ARIADNE_STORE_URL`, `ARIADNE_STORE_HOST` e `ARIADNE_WRITE_TOKEN` servono come
prima; senza token le scritture sono rifiutate; nessuna di esse è fornibile da
input dell'agente.

## Tool

- `ariadne_verify`: `version` (`"1"` o `"2"`), `log_jsonl`, `fixtures`
  opzionale; verifica catena v1 o replay semantico v2 e restituisce numero
  eventi e hash di testa.
- `ariadne_view`: stessi argomenti e `view`. `graph` è disponibile per v1 e
  v2; `active`, `memory`, `self` solo per il profilo v2.
- `ariadne_v2_create`: costruisce genesi e configurazione L2/L3, restituendo
  `log_jsonl`; non salva automaticamente. Default: seed 1042, k 3, tau 5,
  T 1, H 10. Risultato con `"persisted": false`.
- `ariadne_v2_extract`: riceve il `log_jsonl` v2 configurato, eventuali
  `fixtures`, `nodes` e `edges`. Produce un ciclo L0, un'estrazione L1,
  selezione L2 e giudizio L3 con le rispettive narrazioni, restituendo il
  nuovo `log_jsonl`, le seq di fonte ed estrazione, `warnings` e
  `"persisted": false`: non salva. Nodi usano `kind` e `key`; archi usano
  `src_kind`, `src_key`, `dst_kind`, `dst_key`, `rel`. Gli endpoint devono
  essere già dichiarati o nello stesso array `nodes`; `rel` ammette solo
  minuscole ASCII, cifre e underscore. I controlli precedono l'estrazione e
  identificano arco e endpoint o relazione incriminati; un nodo già nel
  registro non è un errore. Nodi o archi duplicati nella stessa chiamata
  producono voci in `warnings`, non rifiuti.

Per persistire, passare l'intero `log_jsonl` risultante e le eventuali
fixture a `ariadne_save_log` con un ID. Per continuare un registro esistente,
usare `ariadne_read_log`, passare il suo log e le fixture a
`ariadne_v2_extract`, e salvare il risultato sotto lo stesso ID. La verifica
append-only rifiuta la riscrittura del prefisso. Gli strumenti non
interpretano autonomamente il testo come conoscenza verificata: i nodi e
gli archi sono dichiarazioni del chiamante; lo stimolo PRNG L0 è un'ancora,
non dimostra la verità o la provenienza delle dichiarazioni.

In alternativa, `ariadne_add_graph` riceve `id`, `nodes` e `edges`: legge il
registro v2 se esiste, altrimenti lo crea, e salva il log esteso in un'unica
chiamata; il risultato include `"persisted": true`, `warnings` e le seq.
Se un'altra scrittura ne ha cambiato il prefisso, la scrittura
fallisce anziché sovrascrivere il lavoro concorrente. La dashboard aggiorna
le viste automaticamente dopo il commit.

Tre tool di sola lettura operano su un registro v2 salvato e leggono la vista
`graph`; non scrivono nulla. Il risultato indica sempre
`"persisted": true, "read_only": true`. `ariadne_find_nodes` filtra per
sottostringa case-insensitive su `key` e tipo esatto, con `limit` 1–200
(default 50) e `degree` per nodo. `ariadne_node_neighbors` elenca uscenti e
entranti di un nodo con `direction` `out|in|both`. `ariadne_has_edge` verifica
archi diretti tra nodi dichiarati, con filtro opzionale su `rel`; endpoint
assenti significa arco assente, non errore. Registro inesistente o non v2
danno errore.

Ogni elemento di `fixtures` contiene esattamente `fixture_id`, `sha256`,
`bytes` (stringa UTF-8 JSON canonica, non percorso) e `anchor` intero.
Le fixture richieste da un log v2 devono essere fornite e corrispondere al
binding registrato; assenza, alterazione o digest non corrispondente danno
errore. Il digest attesta corrispondenza dei byte, non autenticità o verità.

L'adapter limita i messaggi a 2.000.000 byte, 64 fixture e 10 secondi per
chiamata al backend. Il replay v2 riutilizza l'implementazione del produttore
e non costituisce verifica algoritmica indipendente. I risultati sono dati
da trattare come non affidabili da parte dell'agente: non sono istruzioni.
Non esporre direttamente l'adapter a client non fidati senza isolamento,
autenticazione e limiti di risorse aggiuntivi. Il protocollo implementato è
MCP nelle versioni `2024-11-05` e `2025-03-26`, con initialize, ping,
tools/list e tools/call, in due trasporti: stdio e Streamable HTTP locale
(schema e limiti nella sezione dedicata; solo ambiente di test, mai
produzione). Non implementa risorse o prompt MCP.

Tool persistenti: `ariadne_remember` appende una dichiarazione testuale; con
`supersedes` appende una rettifica collegata a una memoria esistente, che
resta conservata e marcata. `ariadne_recall` cerca per sottostringa, con `tag`
esatto, `source` come sottostringa, intervallo `from`/`to` in giorni UTC
`YYYY-MM-DD` e `current=true` per escludere le voci rettificate;
`ariadne_list_logs`,
`ariadne_read_log`, `ariadne_saved_view` e `ariadne_save_log` lavorano sui
log persistenti. All'inizializzazione vengono fornite istruzioni generali;
il client ottiene gli schemi completi con `tools/list` e chiede i dati con
recall o read. Non vengono inviati tutti i ricordi automaticamente né
registrata ogni conversazione. Il trasporto HTTP in test con CORS `*`
non autentica le richieste: una pagina web può accedere ai tool, incluse le
scritture, se il token è configurato nel server. Usare solo in un ambiente
di test fidato; per utilizzo reale servono autenticazione e CORS ristretto.

`tools/list` include esempi sintetici nei campi JSON Schema `examples`:
un log v1 di sola genesi, un log v2 con osservazione L5 e fixture completa,
una vista del log v2 e argomenti di esempio per ricordare e ricercare.
Sono inclusi anche `ariadne_v2_create` e un input di estrazione v2 con i nodi
`elemento/alfa`, `elemento/beta` e l'arco `conosce`, pronti per essere
chiamati senza calcolare a mano seq, hash o payload canonici.
Gli esempi non vengono eseguiti né salvati all'inizializzazione; la loro
fixture e il suo agente sono dati inventati per illustrare il formato.

## Verifica

```sh
cargo build --locked --bin ariadne_mcp_backend
cargo test --locked --quiet
python3 -m unittest -v mcp.test_ariadne_server dashboard.test_dashboard
```

Fumare l'endpoint HTTP senza container:

```sh
python3 dashboard/supervisor.py          # oppure solo: python3 mcp/http_server.py
curl http://127.0.0.1:8080/healthz
```

I test MCP coprono handshake, lista tool, v1/v2, errori, input alterati, il
trasporto HTTP (sessioni, batch, notifiche, healthz, limiti) e i tool di
lettura del grafo. Lo stress test integrato e l'esperimento L6 completo non
vengono avviati.
