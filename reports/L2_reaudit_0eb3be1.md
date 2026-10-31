# Ri-audit L2 — HEAD `0eb3be1`

## Modello

`modello: opencode/nemotron-3.5-super-vl-free` (pin del frontmatter, nessun override, nessun cambio provider).

Mnemosyne: non richiesto (nessuna sessione aperta, nessuna memoria richiamata).

---

## 1. Punto unico: L2.24 tie-break a `src/selection.rs:26` → **ROSSO**

### Codice esaminato

`src/selection.rs:26` (dentro il blocco `if !mentioned.is_empty()`, la riga contiene l'intera logica di espulsione):

```rust
let loser=active.iter().min_by(|a,b|score(a).cmp(&score(b)).then_with(||b.cmp(a))).unwrap().clone();
```

- `a, b: &String` sono id L1 da `active: BTreeSet<String>` (dichiarato riga 20).
- `origin_map: BTreeMap<String, u64>` è in scope (dichiarato riga 20) ma **non consultato** nel confronto.
- `score(...)` è la closure definita più sopra sulla stessa riga 26.

### Legge di riferimento

`notes/L2_legge_selezione.md`:
- **L2.6** (line 37-41): «Dopo ogni valutazione: se il set eccede `k`, esce l'elemento con punteggio minimo; pari merito, rimane quello con seq di origine minore.»
- **L2.17** (line 105-108): «Parità: entra il min seq di origine; l'espulsione (L2.6) si valuta sull'insieme aggregato attivi+candidati.»
- **L2.24** (line 165-170): «Se punteggi uguali e origin_seq uguali … l'ordine è determinato dall'id: **resta/entra** l'elemento con id L1 lessicograficamente minore; l'altro è espulso.» Per l'espulsione, esce quindi l'**id maggiore**.

### Confronto livello per livello

| Livello | Richiesto (legge) | Implementato (`src/selection.rs:26`) | Esito |
|---|---|---|---|
| 1: score | esce il punteggio minimo | `score(a).cmp(&score(b))` → min score esce | **CONFORME** |
| 2: origin_seq | a pari score, resta min `origin_seq` → esce max `origin_seq` | **ASSENTE**: nessun confronto su `origin_map` / `origin_seq` | **VIOLATO** |
| 3: id | a pari origin_seq, resta id minore → esce id max | `b.cmp(a)` → esce id max | **CONFORME ma a livello sbagliato** (posto al livello 2 invece che al 3) |

### Scala effettiva vs scala richiesta

- **Effettiva**: `score → id` (il livello `origin_seq` è saltato).
- **Richiesta**: `score → origin_seq → id`.

### Conseguenza

Quando due elementi hanno stesso punteggio ma `origin_seq` diverso (attivati in eventi diversi con età simile), il tie-break devia sull'id invece che sull'origin_seq: violazione di **L2.6** (resta min origin_seq) e **L2.17** (entra min seq di origine). Quando stesso punteggio **e** stesso `origin_seq`, il reverse-id è corretto per l'espulsione (esce id max, coerente con L2.24), ma il livello intermedio manca, quindi la scala a tre livelli non è implementata.

### Correzione indicata (solo audit — nulla applicato)

A `src/selection.rs:26`, sostituire:

```rust
let loser=active.iter().min_by(|a,b|score(a).cmp(&score(b)).then_with(||b.cmp(a))).unwrap().clone();
```

con:

```rust
let loser=active.iter().min_by(|a,b|score(a).cmp(&score(b))
    .then_with(||origin_map[b].cmp(origin_map[a]))
    .then_with(||b.cmp(a))).unwrap().clone();
```

- `origin_map[b].cmp(origin_map[a])`: l'origin_seq maggiore confronta come "maggiore", quindi in `min_by` l'elemento con origin_seq maggiore risulta "minore" e viene espulso (coerente con L2.6/L2.17: resta min origin_seq).
- `b.cmp(a)`: l'id maggiore confronta come "maggiore", quindi in `min_by` l'elemento con id maggiore risulta "minore" e viene espulso (coerente con L2.24: esce id max per l'espulsione).

Scala risultante: esce min score → a pari esce max origin_seq → a pari esce max id. Esattamente **L2.6 + L2.17 + L2.24**.

### Test dedicato

**Manca** (NOTA, non bloccante). Nessun test esercita il caso "pari punteggio **e** pari origin_seq".

Test minimale suggerito per dimostrare tutti e tre i livelli:
1. **Livello score**: stesso evento, stessi dati, punteggi diversi, `k=1` → deve uscire il min score.
2. **Livello origin_seq**: stesso punteggio, `origin_seq` diversi (es. attivati in eventi diversi), `k=1` → deve uscire il max `origin_seq` (resta il min).
3. **Livello id (L2.24)**: stesso evento (stesso `origin_seq`, stesso tick, stesso score), `k=1` → deve entrare/restare l'id lessicograficamente minore, esce il maggiore.

Un singolo test L2.24-specifico: attiva due nodi nello stesso evento (stesso `origin_seq`, stessa activazione → stesso score), `k=1`, assert che l'id lessicograficamente maggiore compaia in `downs` (espulso) e il minore resti in `active`/`ups`.

---

## 2. Fix dichiarati nel ri-gate → **VERDE** (confermati file:riga)

| Fix | Citazione | Esito |
|---|---|---|
| L2.16: `text` del summary = id L1 (SHA-256 hex 64), lessicografico, separati da `\|`, mai `origin_seq` hex | `src/selection.rs:18` (`id()` = SHA-256 di `{"key","kind}`); riga 21 (`expelled_ids: BTreeSet<String>`, `ids_sorted.sort()`, `ids_sorted.join("|")`) | **VERDE** |
| Cutoff decisionale solo su eventi `l2` decisionali (`select`/`summary`); barrier/config = controllo esclusi (L2.23); niente ri-emissione pre-`base.log_len` (L2.14/L2.22) | `src/selection.rs:30` (`evaluate`): `decision_cutoff` filtra solo `t=="select"\|t=="summary"`; `cutoff = decision_cutoff.max(log.events()[base.log_len-1].seq)` | **VERDE** |
| Copertura replay con eventi L2 preesistenti | `tests/l2_selection.rs:95-103` (`evaluate_uses_only_events_after_the_latest_existing_l2_event`) e `:106-112` (`evaluate_does_not_reemit_summary_before_base`) | **VERDE** |

Altre proprietà già verificate dal tester e non rieseguite qui: determinismo replay, zero scritture (`to_jsonl` invariato), zero wall-clock, `head_hash` Elementary (`src/view.rs:212-217` + check `src/selection.rs:19`).

---

## 3. Dead code (NOTA, non bloccante)

- `src/selection.rs:3`: `use crate::{EventType,Log,Value};` — `EventType` importato, mai letto nel file.
- `src/selection.rs:13`: `fn strv` — definito, mai chiamato.
- `src/selection.rs:15`: `fn arr_seq` — definito, mai chiamato.
- `last_decision`: **non riscontrata** in `src/selection.rs` (nessuna occorrenza; già assente in questa versione).

---

## 4. Ordine test-first → **VERDE**

- Primo commit con test L2: `1188d8f` (`test: specify L2 selection replay contract`) → solo `tests/l2_selection.rs` (3 test), suite rossa (modulo `selection` inesistente).
- `8b8019f`: solo test (+7 test), ancora suite rossa.
- Implementazione: `c8f7ace` (`L2 implement: selection read-only con legge L2.1-L2.22`) → arriva **dopo** i test.
- `0eb3be1` (HEAD): fix successivo misto codice+test, **non** è il primo commit del task. Requisito "primo commit solo test" soddisfatto.

---

## 5. Esito complessivo → **ROSSO**

**ROSSO** per la sola violazione di legge **L2.24** (e di riflesso L2.6/L2.17): la scala a `src/selection.rs:26` è `score → id`, manca il livello `origin_seq` e l'ordine è incompleto/inversato rispetto a `score → origin_seq → id`. Correzione indicata al punto 1 (non applicata: solo audit).

Tutto il resto è **VERDE**: fix L2.16 (`text` id L1), cutoff decisionale L2.14/L2.22/L2.23, copertura replay L2 preesistenti, determinismo, zero scritture, zero wall-clock, `head_hash`, ordine test-first.

**NOTE non bloccanti**: dead code (`EventType` import, `strv`, `arr_seq`; `last_decision` assente) e mancanza di test dedicato L2.24.

Nessuna accettazione di livello dichiarata: spetta all'umano.
