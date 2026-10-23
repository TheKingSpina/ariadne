# L1 — Domande puntuali sui residui di formato della vista (D14–D21)

Stato: L1 NON iniziato. Nessun test, nessun codice. Questo file è l'unico artefatto.
Basato su: `spec/L1_SPEC_v0.1.md` + addendum umano del dispatch del 23/10/2026 (che ha
chiuso D1–D13) + codice L0 accettato (`src/{event,log,payload}.rs`).

L'addendum determina già, e NON sono in lista qui: grammatica del payload
(`nodes`/`edges` stringhe con array JSON canonico interno), dichiarazione di nodo
solo da elemento `{kind,key}`, id = SHA-256 del JSON canonico di `{kind,key}`
(→ preimage `{"key":K,"kind":T}`: chiavi trattate in ordine di byte, `key` prima
di `kind`), `origin_seq` = seq minimo di dichiarazione, elementi arco
`{dst_kind,dst_key,rel,src_kind,src_key}`, regex rel `[a-z0-9_]+`, riferimento a
nodo non ancora dichiarato = errore (no skip, no nodi impliciti), eventi senza
`nodes`/`edges` saltati, cicli ammessi, builder su `&Log` con zero scritture,
ordinamento vista nodi per `id` e archi per `(src_id,dst_id,rel)`.

Restano NON determinati per produrre test byte-per-byte della vista. Per ognuno
è proposta un'opzione predefinita; l'umano può confermarla o correggerla.

## D14 — Ancoraggio all'hash di testa: il campo non è nell'addendum

La SPEC §5 è legge: «Ogni vista è versionata e ancorata all'hash di testa del
log che l'ha generata». L'addendum elenca i campi della vista (`schema_version`,
`view_hash`, nodi, archi) SENZA campo di ancoraggio. Discrepanza dichiarata, non
colmata per intuizione.

**Proposta (default):** campo `head_hash`, stringa, = hash dell'ultimo evento
del log (`events().last().hash`); la vista quindi: `{"edges":[…],"head_hash":"…",
"nodes":[…],"schema_version":…,"view_hash":"…"}` con chiavi in ordine canonico
alfabetico come le chiavi del payload L0.

## D15 — `schema_version`: valore e tipo

L'addendum richiede il campo ma non dice né tipo né valore.

**Proposta (default):** stringa `"1"`. Alternativa: `"0.1"` agganciato alla
versione del file SPEC. Su questa scelta dipende ogni byte della vista.

## D16 — Preimage esatto di `view_hash`

«SHA-256 della serializzazione canonica del contenuto»: che cos'è «il contenuto»?
Tre candidate, tutte plausibili: (a) solo `{"edges":[…],"nodes":[…]}`; (b) tutto
il documento tranne `view_hash` stesso, cioè
`{"edges","head_hash","nodes","schema_version"}`; (c) come (b) con separatore
0x1F. Il test «log vuoto ⇒ nodi/archi vuoti e il relativo hash» (e la checklist 3)
non è scrivibile in forma non ambigua senza la forma esatta.

**Proposta (default):** (b) — intero documento tranne il campo che lo porta;
esadecimale minuscolo; nessun separatore 0x1F (per l'id del nodo l'addendum usa
SHA-256 del JSON canonico puro, nessun separatore: stessa forma qui).

## D17 — Nomi esatti dei campi di nodo e arco nella vista serializzata

SPEC §3: nodo = `{id, tipo, origine: seq}`; §4: arco = `{sorgente, destinazione,
tipo, seq_di_origine}`. L'addumno nominalizza `id`, `kind`, `key`, `origin_seq`
(dove «tipo» = `kind`) ma per gli archi lascia aperto: `sorgente/destinazione`
come coppie kind+key o come `src_id`/`dst_id`? E `seq_di_origine` o `origin_seq`
(uniforme al termine dell'addendum)? Il sorting `(src_id,dst_id,rel)` dell'addendum
parla di id, ma un arco che porta solo id perde il «tipo» del §4 nel senso
dell'endpoint? Ogni byte della vista dipende dai nomi.

**Proposta (default):** nodo = `{"id","key","kind","origin_seq"}` (già determinato
dall'addendum); arco = `{"dst_id","origin_seq","rel","src_id"}` — endpoint come id
risolti (coerente col sorting addendum), uniformità `origin_seq`. Qui l'arco perde
kind+key grezzi: recuperabili via lookup del nodo. Alternativa: endpoint come
coppie kind+key oltre agli id.

## D18 — Arco provato da più eventi: quale `seq_di_origine`?

L'addendum fissa `origin_seq` minimo per i nodi; per gli archi silenzia. Due
eventi possono provare lo stesso arco `(src,dst,rel)` con seq diverse.

**Proposta (default):** il più basso, per simmetria con i nodi; l'arco è unico
nell'output (identità = trip `(src_id,dst_id,rel)`).

## D19 — Nodi e archi nello stesso evento: ordine di risoluzione

«Ordine di replay» è chiaramente l'ordine di seq tra eventi; nel singolo evento
con entrambi gli array non è detto. Un elemento arco che punta a nodi dichiarati
nel medesimo evento è valido o errore?

**Proposta (default):** nel singolo evento l'array `nodes` è processato per
intero prima dell'array `edges` (nodi-primi); il test ciclico A→B→C→A verrà
comunque costruito con dichiarazioni ed eventi separati, così non dipende da
questa regola.

## D20 — Il builder rivérifica la catena L0?

«Solo il log L0, letto in ordine di seq. Nient'altro.»: il builder assume un log
verificato (come la fold L0, memoria `mem_7b37a78d`) o ricalcola `verify()`?

**Proposta (default):** assume log verificato, nessun ricalcolo di hash: le
proprietà strutturali L0 sono di L0, L1 non le re-implementa (regola di
provenienza ereditata).

## D21 — «Log vuoto»: come si costruisce il test?

Nella L0 pubblica non esiste un `Log` senza eventi: `genesis` è obbligatoria e
`from_jsonl` rifiuta l'input vuoto. Non c'è modo di avere un `&Log` con zero
eventi da fuori del crate.

**Proposta (default):** «log vuoto» = log senza estrazioni (genesi soltanto, il
cui payload non porta `nodes`/`edges`): vista con `nodes=[]`, `edges=[]` e hash
relativo; `head_hash` = hash della genesi (golden conosciuto `4649d1f4…`).

---

## Cosa NON farò finché D14–D17 non sono fissate

- Nessun test L1, nessun codice L1, nessuna funzione di supporto parziale: i
  residui D14–D17 definiscono ogni byte dell'output che i test dovrebbero pinnare
  (checklist 1 «byte per byte», 3 «ancorata», e il golden del log vuoto).
- D18–D21 hanno default forti (simmetria nodi/archi, direzione di lettura,
  precedenti approvati): se l'umano conferma tout court, si fermano le domande.
- La SPEC non viene modificata; `BLOCKER.md` resta reperto storico di D1–D13.
