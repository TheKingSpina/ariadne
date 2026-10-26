**Ariadne — L1_SPEC v0.3 (federa consolidata) — FIRMATA**

Questo documento **consolida in un unico testo autonomo** la legge L1 in
vigore: base `spec/L1_SPEC_v0.1.md`, addendum F1 di `spec/L1_SPEC_v0.2.md`,
le risposte D1–D13 dell'umano del 23 ottobre 2026 e le decisioni D14–D21
accettate col livello. Le versioni precedenti e `notes/L1_domande_vista.md`
restano **reperti storici**.

L'essere consolidata **non introduce** requisiti nuovi: restate conforme
all'implementazione accettata (`src/view.rs`, ciclo L1 `c8c69d7` + `87bc9e9`,
messa a punto F1 `d082209`).

Firmata il 26 ottobre 2026. L'accettazione umana di L1 è del 23 ottobre 2026
(gate: 11 test L1, 91 totali; audit reviewer conforme; addendum F1 `d082209`).

---

**§1 — Perimetro**

- L1 è una vista derivata: il grafo è una proiezione del log, mai una fonte
  separata.
- Cicli ammessi e liberi — l'aciclicità è legge solo in L0, qui il grafo può
  collegarsi come vuole.
- Zero scritture: L1 non tocca mai il log.

**§2 — Input**

Solo il log L0 (eventi SENSE/ACTION/STATE), letto in ordine di seq.
Nient'altro. L'implementazione ammette estrazioni da ogni tipo di evento: ciò
che determini le estrazioni è il payload, non il tipo (F1).

**§3 — Estrazione: `nodes` e `edges` nel payload L0 (F1 + D1–D8)**

Il payload di un evento L0 che estrae nodi o archi contiene le chiavi
**stringa** `nodes` e, al più, `edges`. Ciascuna chiave porta come valore una
stringa contenente **array JSON canonico interno**:

- `nodes`: array di oggetti esatti `{kind, key}` — entrambe stringhe; nessun
  campo extra o mancante;
- `edges`: array di oggetti esatti
  `{src_kind, src_key, dst_kind, dst_key, rel}` — tutte stringhe; `rel` deve
  corrispondere a `[a-z0-9_]+`, altrimenti errore;
- il JSON interno segue la stessa grammatica severa del payload L0: UTF-8,
  chiavi in ordine lessicografico di byte, nessuno spazio, escape minimo
  standard. Forma non canonica → errore, non input.

Il builder L1 **fa il parse** di queste stringhe: il payload L0 resta scalare
e invariato; l'array vive solo dentro la stringa. Nessuna estrazione implicita
dal testo: un nodo o un arco esiste solo se dichiarato in questi array.
Eventi senza `nodes`/`edges` contribuiscono zero.

Identità e provenienza:

- la vista ha **id canonico e deterministico**: l'id di un nodo è
  `SHA-256` minuscolo del JSON canonico di `{"key":K,"kind":T}` — chiavi in
  ordine di byte, quindi `"key"` prima di `"kind"`; stesso payload → stesso
  id, ovunque;
- `origin_seq` di un nodo = seq **minimo** (per ordine di dichiarazione) fra
  gli eventi che la dichiarano (D1–D4, D18);
- un riferimento a nodo non ancora dichiarato è **errore**: niente skip,
  niente nodi impliciti (D6);
- cicli ammessi; il test del ciclo A→B→C→A è scritto con dichiarazioni in
  eventi separati (grazie a §4 nodes-primi, non dipende da quel criterio);
- nessun contratto semantico a livello L0: "entità" e "relazione" esistono
  solo per queste convenzioni di payload.

**§4 — Ordine di replay e risoluzione (D19)**

Tra eventi vale l'ordine di seq. Nel **singolo evento** che dichiara entrambi
gli array, `nodes` è processato per intero **prima** di `edges`
(nodi-primi): un arco che punta a nodi dichiarati nello stesso evento è
quindi valido.

**§5 — Vista serializzata: schema e campo (D8–D10, D14–D17)**

La vista è un documento canonico con membri in ordine lessicografico:

```
{"edges":[…],"head_hash":"…","nodes":[…],"schema_version":"1","view_hash":"…"}
```

- `schema_version`: stringa `"1"` (D15).
- `head_hash`: stringa = hash dell'**ultimo evento** del log
  (`events().last().hash`), compresa l'eventuale genesi-only (D10, D14).
- Nodo serializzato: `{"id":…,"key":…,"kind":…,"origin_seq":…}` — le
  "coppie kind+key" di §3/§4 sono risolte in endpoint per id; grezzo è
  recuperabile via lookup del nodo (D17). `tipo` = `kind`.
- Arco serializzato: `{"dst_id":…,"origin_seq":…,"rel":…,"src_id":…}` (D17).
- Ordinamento: nodi per `id`; archi per `(src_id, dst_id, rel)`.
- Arco unico nell'output per trip `(src_id, dst_id, rel)`; il `origin_seq`
  dell'arco è il seq **minimo** fra gli eventi che lo provano (D18).

**§6 — `view_hash` (D16)**

`view_hash` = SHA-256 minuscolo del **documento canonico senza `view_hash`**:
preimage = `{"edges":…,"head_hash":…,"nodes":…,"schema_version":"1"}`.
Nessun separatore 0x1F (stessa forma del preimage dell'id di nodo).

**§7 — Ricostruzione (v0.1 §5)**

Replay del log → estrazione deterministica: stesso log → stesso grafo,
byte per byte. Ogni vista è versionata e ancorata all'hash di testa del log
che l'ha generata: il grafo porta con sé la sua verità.

**§8 — Postura del builder (D11–D12, D20)**

- Il builder opera su `&Log` (`view::build(log)`): zero scritture per firma.
- Assume un **log L0 verificato** (come la fold L0): nessun ricalcolo di hash;
  le proprietà strutturali L0 sono di L0, L1 non le ri-implementa.
- Impl. pubblica: `src/view.rs` con `build(&Log)` ed esportazione della
  struttura vista; anch'essa zero I/O e zero wall-clock.

**§9 — Log "vuoto" (D21)**

«Log vuoto» = log **senza estrazioni**: genesi soltanto (payload di genesi
senza `nodes`/`edges`). Vista corrispondente: `nodes=[]`, `edges=[]`,
`head_hash` = hash della genesi e `view_hash` calcolato normalmente.
Golden: genesi con seed 1042 → hash di testa
`4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7`.

**§10 — Checklist di uscita** (tutte superate; gate 11/11 test L1 al 23/10/2026)

- [x] Ricostruzione deterministica su due esecuzioni indipendenti.
- [x] Ogni arco tracciato alla sua seq di origine.
- [x] Vista ancorata all'hash di testa del log.
- [x] Zero scritture al log dimostrate.
- [x] Test con grafo ciclico valido superato.

---

## Firma

- [x] **umano** — 23 ottobre 2026 (accettazione del livello L1, con D14–D21;
  addendum F1 `d082209`)
- [x] **orchestratore** — 26 ottobre 2026 (federa consolidata, trascrizione
  della legge esistente; nessun requisito nuovo)
