# Dashboard e archivio locale Ariadne

La dashboard è un'app HTTP. Mostra i log effettivamente
archiviati, le memorie testuali, la sequenza di eventi e le viste `graph`, `active`, `memory`,
`self` disponibili per protocollo. Non presenta dati inventati quando
l'archivio è vuoto.

## Processi coordinati (dashboard + MCP HTTP)

`dashboard/supervisor.py` avvia e arresta insieme:

- `dashboard/server.py` (dashboard + archivio SQLite) sulla porta 8090;
- `mcp/http_server.py` (endpoint MCP Streamable HTTP) sulla porta 8080,
  che chiama l'API della dashboard.

Il token di scrittura è caricato una sola volta con `load_write_token` e
passato a entrambi i processi via `ARIADNE_WRITE_TOKEN`; allo store MCP sono
esportate anche `ARIADNE_STORE_URL=http://127.0.0.1:8090` e
`ARIADNE_STORE_HOST=127.0.0.1:8090`. Se un figlio esce, l'altro viene
terminato e il supervisor esce con codice non zero; SIGTERM/SIGINT li arresta
entrambi. I log figli sono prefissati `dashboard:`/`mcp:`.

Nel container la stessa coppia di processi è avviata automaticamente dal
`CMD` del Dockerfile. Il file `docker-compose.yml` riusa **esplicitamente** il
volume esistente `ariadne-dashboard-data` (campo `name`) e il contenitore
`ariadne` (`container_name`), così log e memorie attuali restano in
uso senza creare un secondo volume. Per la build e le porte:

```sh
docker compose -f dashboard/docker-compose.yml up -d --build
docker compose -f dashboard/docker-compose.yml logs -f
docker compose -f dashboard/docker-compose.yml down
```

### Migrazione una tantum dal contenitore creato a mano

Il contenitore attuale creato manualmente va sostituito, senza toccare il
volume: fermalo e rimuovilo, poi crea il contenitore compose che riusa
`ariadne-dashboard-data` (log e memorie sono conservati nel volume, non nel
contenitore):

```sh
docker stop ariadne
docker rm ariadne
cd dashboard && docker compose up -d --build
```

Dopo la migrazione l'endpoint MCP risponde su `http://127.0.0.1:8080/mcp`
(healthz su `/healthz`) e la dashboard su `127.0.0.1:8090`. Il servizio ha un
`healthcheck` Compose che interroga `/healthz`, `stop_grace_period` di 20
secondi e `init: true`. Non eseguire `docker compose down -v` né
`docker volume rm`: il volume è l'unico luogo in cui vivono gli eventi e le
memorie.

L'endpoint MCP su `http://127.0.0.1:8080/mcp` e `/healthz` è **solo test**:
nessuna autenticazione client, CORS `*` (configurabile con
`ARIADNE_MCP_ALLOW_ORIGIN`), header `Host` non controllato. Le porte sono
pubblicate solo su `127.0.0.1`. Dettagli del protocollo HTTP in
`mcp/README.md`. L'archivio resta nel volume `ariadne-dashboard-data`; non è
una seconda istanza di dati. Fuori da Docker:

```sh
python3 dashboard/supervisor.py --host 127.0.0.1
```

La dashboard apre `GET /api/stream` con Server-Sent Events. Dopo ogni commit
riuscito attraverso questa API viene inviata una notifica e le viste vengono
ricaricate automaticamente; alla riconnessione viene richiesta una nuova
lettura. È aggiornamento push, non un monitor generico di modifiche fatte
direttamente da altri processi al file SQLite. Il pulsante Aggiorna resta
disponibile come ripiego.

## Persistenza

L'archivio usa SQLite (`/data/ariadne.sqlite3` nel container) in un volume
Docker nominato. I log sono memorizzati evento per evento; anche le fixture
L5 necessarie per il replay vengono conservate. Ogni salvataggio verifica
prima il log completo col backend Ariadne e richiede che esso estenda
**byte per byte** il prefisso già archiviato. Troncamenti, fork, cambi di
versione e riassociazione delle fixture sono rifiutati. Le righe sono
protette da trigger SQLite contro `UPDATE` e `DELETE`; una transazione
registra insieme evento/i nuovi e binding di fixture.

Il volume sopravvive ai normali riavvii del container, non alla sua
eliminazione esplicita o alla perdita del disco: servono backup e procedure
di ripristino. I trigger non proteggono da chi ha accesso diretto al file
SQLite o al volume. SQLite usa WAL e `synchronous=FULL`, ma non è una
garanzia di durabilità assoluta contro guasti di hardware o filesystem.

Le memorie testuali sono una tabella append-only separata dai log L0–L5:
ogni voce contiene ID, contenuto, fonte dichiarata, tag e ora UTC. Sono
dichiarazioni di chi le salva, non fatti verificati né eventi della catena.
La ricerca è per sottostringa, non semantica; `lower` di SQLite non fornisce
una normalizzazione completa di tutte le lingue Unicode. La dashboard
consente di filtrare per etichetta esatta (`tag`), per sottostringa della
fonte (`source`) e per intervallo di giorni UTC; la ricerca semantica non è
implementata.

### Rettifiche senza cancellare la storia

Una correzione è una **nuova** memoria con `supersedes` uguale all'ID della
voce precedente. La voce originale resta nel database, nei risultati e nella
dashboard; viene solo marcata come `superata da #N`. Una memoria può avere una
sola rettifica diretta: un secondo tentativo viene rifiutato. Non esistono
update o delete: i trigger append-only restano attivi anche sulla colonna
`supersedes`. Nelle ricerche compaiono entrambe le voci, salvo
`current=true`, che mostra solo le memorie senza successore.

## Backup e ripristino

`dashboard/backup.py` è uno strumento CLI autonomo (solo standard library,
più il backend di verifica `mcp/ariadne_server.py` e lo schema di
`dashboard/server.py`) che copre il vuoto tra "il volume sopravvive ai
riavvii" e "il volume è un backup":

```sh
python3 dashboard/backup.py export --database /data/ariadne.sqlite3 --output backup.json
python3 dashboard/backup.py verify --input backup.json
python3 dashboard/backup.py restore --input backup.json --database /data/ariadne.sqlite3 --replace
```

- `export` legge l'archivio attraverso l'API di backup online di SQLite
  (`sqlite3.Connection.backup`) su una copia temporanea, quindi non legge un
  database in scrittura in modo incoerente. Il file di output è un singolo
  JSON `ariadne-backup/1` con permessi `0600`: `schema`, `created_at`,
  `source` (solo il nome del file del database, mai un percorso completo),
  le tabelle `logs`, `events`, `fixtures` e `memories`, e
  `integrity.digest`, lo SHA-256 del JSON canonico di `data`
  (`sort_keys=True`, separatori `","` e `":"`, `ensure_ascii=False`).
  Se `--output` esiste serve `--force` per sovrascriverlo.
- `verify` ricalcola il digest e stampa schema, conteggi e digest; esce con
  codice non zero se il file è alterato o ha uno schema sconosciuto.
- `restore` rifiuta di sovrascrivere in silenzio: se il database di
  destinazione esiste e contiene righe in una qualsiasi tabella serve
  `--replace`. Prima valida schema, digest e struttura del contenuto;
  proprio con `--replace` copia il database esistente, in modo coerente, in
  `<database>.pre-restore-<UTC>` nella stessa directory **prima** di
  sostituirlo; se il destino non esiste crea la directory padre. Poi
  ricostruisce il nuovo database in `<database>.restore-tmp` con lo schema
  e i trigger esatti della dashboard e valida ogni log con `ariadne_verify`
  (versione e fixture del registro). Solo se ogni log passa la verifica il
  nuovo database viene attivato con una `os.replace` atomica sulla stessa
  directory; in caso di errore i file temporanei vengono rimossi e il
  database esistente resta come era. Il ripristino è locale e offline:
  arrestare supervisor e container prima di agire su
  `/data/ariadne.sqlite3`.

Cosa contiene e cosa no: il file di export contiene memorie e log, quindi
va protetto come il volume stesso; il digest certifica l'integrità del
file, non l'autenticità o la verità di ciò che è archiviato. Il token di
scrittura (`/data/.write-token`) **non** è incluso, non viene letto e non
viene toccato dal ripristino: chi lo possiede resta l'unico abilitato a
apporre estensioni valide. La copia `pre-restore-*` creata nella directory
della destinazione contiene gli stessi dati del database sostituito: va
conservata o rimossa con una decisione consapevole, il restore non la
elimina mai.

## Vista grafo

Selezionare un registro e aprire **Grafo**: la vista SVG mostra nodi, archi
direzionati ed etichette delle relazioni della proiezione L1 verificata.
La barra sopra il grafico permette di cercare per chiave, tipo o ID, filtrare
per tipo, aumentare o ridurre lo zoom e ripristinarlo. Il conteggio mostra
quanti nodi sono visibili e quanti archi il filtro nasconde.

Selezionando un nodo, anche da tastiera, vengono mostrati tipo, ID, seq di
origine e l'elenco delle relazioni entranti e uscenti; ogni relazione può
essere aperta per raggiungere il nodo collegato e riporta la propria
`origin_seq`. L'intestazione dichiara sempre il registro e i primi caratteri
dell'hash di testa da cui deriva la vista. I dati JSON restano consultabili
sotto il grafico. La disposizione circolare è solo una scelta di
visualizzazione, non attribuisce significato alle distanze. Il grafico viene
aggiornato in seguito alle notifiche della dashboard; se il registro non
contiene archi, nessun collegamento viene inventato.

## API HTTP, distinta da MCP

- `GET /api/logs` elenca i registri.
- `GET /api/diagnostics` riporta raggiungibilità di archivio e MCP HTTP,
  conteggi, ultimo aggiornamento e versioni dei registri. Il campo
  `conformity.status` vale `not_verified`: la diagnostica non è una prova di
  conformità, verità o autenticità.
- `GET /api/memories?q=...&limit=50&tag=...&source=...&from=...&to=...&current=true`
  ricerca le memorie testuali; `tag` è un'etichetta esatta, `source` una
  sottostringa della fonte, `from`/`to` sono giorni UTC `YYYY-MM-DD` inclusi,
  `current=true` esclude le voci rettificate.
- `POST /api/memories` appende una memoria con `content` obbligatorio e
  `source` e `tags` opzionali; richiede il token di scrittura.
- `GET /api/logs/<id>` restituisce log e fixture salvati.
- `GET /api/logs/<id>/views/<view>` ricostruisce una vista dopo verifica;
  per v1 è disponibile `graph`, per v2 `graph`, `active`, `memory`, `self`.
- `POST /api/logs/<id>` crea un log o appende la sua estensione. Richiede
  `Content-Type: application/json` e header `X-Ariadne-Token`. Il corpo è
  `{"version":"1"|"2","log_jsonl":"...\n","fixtures":[]}`; le fixture
  v2 hanno forma `{"fixture_id":"...","sha256":"...","bytes":"...","anchor":0}`.
  L'ID log accetta solo lettere, cifre, `_` e `-`, massimo 64 caratteri.

Per ottenere il token generato all'avvio del container:

```sh
docker exec ariadne cat /data/.write-token
```

Nei log il token non viene stampato: compare solo un'impronta SHA-256 troncata
e il percorso del file. Il token generato viene conservato nel volume
`/data/.write-token` con permessi 0600 e rimane stabile ai riavvii;
`ARIADNE_WRITE_TOKEN` può esplicitamente sostituirlo. Trattalo come una
credenziale: chi lo possiede può apporre estensioni valide.
Non condividerlo con pagine web o agenti non fidati. Accesso al socket Docker
equivale a controllo privilegiato della macchina. Per la lettura non è
richiesto token: il servizio è pensato **solo per localhost e utenti locali
fidati**, non per l'esposizione a Internet o a una rete condivisa. Host e
Origin vengono controllati; la dashboard non abilita CORS.

Il server MCP può salvare e cercare memorie con `ariadne_remember` e
`ariadne_recall`, e salvare log con `ariadne_save_log`. Non intercetta
automaticamente ogni conversazione: serve una chiamata esplicita. I log
richiedono il contenuto completo verificabile e le eventuali fixture.
Per aggiungere **e salvare** nodi e archi in un'unica operazione l'agente può
usare `ariadne_add_graph` indicando l'ID del registro; con
`ariadne_find_nodes`, `ariadne_node_neighbors` e `ariadne_has_edge` può
leggere il grafo salvato senza scrivere nulla. `ariadne_v2_extract`
rimane invece un'operazione temporanea: per persisterla serve il successivo
`ariadne_save_log`.

## Test

```sh
python3 -m unittest -v dashboard.test_dashboard dashboard.test_backup
python3 -m unittest -v mcp.test_ariadne_server
```

`dashboard.test_dashboard` e `dashboard.test_backup` richiedono
`cargo build --locked --bins` prima dell'esecuzione. I test della dashboard
usano un database temporaneo proprio e coprono riapertura, append, rifiuto
fork, immutabilità, fixture, token, Origin e API. I test del backup coprono
roundtrip export → verify → restore, digest alterato, schema sconosciuto,
destinazione non vuota senza `--replace`, copia `pre-restore-*` con
`--replace`, token mai esportato né modificato, catena rotta ( il restore
abortisce e il database esistente resta invariato ), `--output` esistente
senza `--force` e codici di uscita della CLI. Nessun esecuzione dello stress
test.
