# BLOCKER — L1_SPEC_v0.1.md: lacune che impediscono test non ambigui

Stato: L1 NON iniziato. Nessun test, nessun codice. Questo file è l'unico artefatto.
Motivazione: la SPEC L1 è troppo breve per derivare test verificabili senza colmare
lacune per intuizione — operazione vietata dal dispatch. Ogni ambiguità è citata
per sezione e riga della SPEC (`spec/L1_SPEC_v0.1.md`), con lo stato di fatto L0.

Fatti L0 già pinnati (non ambigui, legge esistente):
- Payload = mappa libera `String → {Int(i64), Str(String)}`, ordine alfabetico,
  JSON canonico compatto (`src/payload.rs:15-20, 40-44, 84-104`). **Nessun
  contratto semantico**: nessuna chiave "entità", "relazione", "sorgente",
  "destinazione" è definita da L0.
- Hash evento = SHA-256 di `seq ‖ 0x1F ‖ tick ‖ 0x1F ‖ type ‖ 0x1F ‖
  payload_canonico ‖ 0x1F ‖ prev_hash`, esadecimale minuscolo
  (`src/event.rs:95-113`).
- Eventi ammessi: SENSE / ACTION / STATE; payload ammesso anche vuoto
  (GENESIS_PREV_HASH per genesi; STATE compensativo con `compensates`).

---

## A. §3, righe 12-13 — Nodi: regola di estrazione e preimage dell'id

> «Nodo = entità estratta dal payload: {id, tipo, origine: seq}»
> «id canonico e deterministico dall'hash del contenuto: stesso payload → stesso id»

**D1.** Quali chiavi del payload identificano un'entità? L0 non definisce nessuna
convenzione semantica: "estratta dal payload" è senza regola. Possibili interpreti:
una chiave dedicata (es. `entity`), una convenzione di prefisso, o il payload intero.

**D2.** «hash del contenuto»: di COSA esattamente? (a) del payload intero
dell'evento; (b) di un sottoinsieme dichiarato; (c) del payload dell'entità.
Se (a): due eventi con payload identico ma seq/tick diversi producono lo stesso
id — è voluto? Se (c): quale forma canonica del sottoinsieme (payload.canonical
del BTreeMap? una serializzazione dedicata?).

**D3.** L'algoritmo è SHA-256 come L0 (`src/event.rs:95-113`)? Il preimage include
il campo `tipo`? Separatori (0x1F come L0 o concatenazione pura)? Qualche
collisione seq/tick bypass è richiesta come in L0 (mem_3adbd62d)?

**D4.** «origine: seq» — se la stessa entità ricorre in eventi multipli, il campo
origine è il seq del PRIMO evento che la produce, o l'entità è duplicata per ogni
occurrence? (Interazione diretta con D1: se id è hash del payload intero, payload
identico in seq diversi → stesso id → origine ambigua.)

## B. §4, righe 15-17 — Archi: forma della relazione nel payload

> «Arco = relazione con prova: {sorgente, destinazione, tipo, seq_di_origine}»

**D5.** Quali chiavi del payload codificano una relazione? L0 non ne definisce
alcuna. Serve: una chiave dedicata? Un prefisso? Un formato di valore (stringa
strutturata? due chiavi accoppiate?).

**D6.** «sorgente»/«destinazione» sono id di nodo (output di D2) o valori grezzi
del payload? Se id di nodo, un arco esiste solo se entrambe le entità sono state
estratte da qualche evento — regola di dangling reference non specificata
(ignorare, errore, nodo implicito?).

**D7.** «tipo» dell'arco: libero dal payload? Ammesso anche vuoto?

## C. §2/§5, righe 9, 19-21 — Ricostruzione: formato di output e comportamento vuoto

> «Solo il log L0 (eventi SENSE/ACTION/STATE), letto in ordine di seq. Nient'altro.»
> «Replay del log → estrazione deterministica: stesso log → stesso grafo, byte per byte.»
> «Ogni vista è versionata e ancorata all'hash di testa del log»

**D8.** «byte per byte» di CHE serializzazione? Il formato della vista è
completamente non specificato: JSONL con ordine membri? JSON unico? Quale ordine
di nodi e archi nell'output (seq di origine? id lessicografico? ordine di
inserimento)? Senza il formato, il test "stesso log → stesso grafo byte per byte"
non è scrivibile.

**D9.** «versionata»: quale forma (intero monotonico, semver, stringa)? È un
campo dell'output serializzato o metadato separato? Chi lo incrementa (il
produttore della vista, non L1 in sé)?

**D10.** «ancorata all'hash di testa del log»: è l'hash dell'ultimo evento
del log? È un campo dell'output? Su log vuoto/genesisi-only quale valore?

**D11.** Eventi senza dati semantici (payload vuoto, STATE compensativo
`{compensates, reason, state}`, payload arbitrario senza entità): la vista
li salta, li conta, restituisce errore? §2 dice "Nient'altro" come input ma
non dice cosa succede quando un evento ammesso da L0 non contiene nulla di
estrattibile.

## D. §6, righe 27-28 — Zero scritture e ciclo

> «[ ] Zero scritture al log dimostrate.»
> «[ ] Test con grafo ciclico valido superato.»

**D12.** «Zero scritture»: dimostrato come? L'API Log è append-only ma `&mut`
esiste; serve una firma della vista che prenda solo `&Log`/`&str` non modificabile,
o un test che verifica che l'API di scrittura non è chiamabile? La forma del
test dipende dall'interfaccia scelta per L1 (non specificata: funzione libera?
modulo `l1`? struct `View`?).

**D13.** «Test con grafo ciclico»: un arco A→B e B→A è ammesso, ma con quale
payload L0 concreto si costruisce il ciclo? Dipende da D5 (formato della
relazione). Il test è scrivibile solo dopo la decisione su D5/D6.

---

## Domande puntuali all'umano (riassunto per risposta rapida)

| # | Domanda | Opzione minima necessaria |
|---|---------|---------------------------|
| D1-D4 | Come si estrae un'entità e qual è il preimage esatto dell'id canonico? | Regola estrazione + algoritmo hash + preimage + comportamento su occorrenze multiple |
| D5-D7 | Quali chiavi/formato codificano una relazione? Cosa sono sorgente/destinazione? | Formato relazione + regola dangling + tipo arco |
| D8 | Qual è il formato di serializzazione della vista (per il test byte-per-byte)? | Schema output + ordine nodi/archi |
| D9 | Forma del versionamento? | Campo/metadato + tipo |
| D10 | Che cos'è "hash di testa" nel campo di ancoraggio? | Definizione + comportamento su log triviale |
| D11 | Eventi senza dati semantici: errore, skip, o tracciati? | Comportamento |
| D12-D13 | Interfaccia L1 (funzione/struct) per dimostrare zero scritture? Formato ciclo? | Firma API + payload ciclo |

## Cosa NON farò finché il blocker non è risolto

- Nessun test L1, nessun codice L1. Non colmerò D1-D13 per intuizione.
- La SPEC non viene modificata: se le risposte richiedono addendum, è compito
  dell'umano/governance (pattern L0: addendum v0.2 → v0.3 firmati).
