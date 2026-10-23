**Ariadne — L1_SPEC v0.2 (addendum alla v0.1) — FIRMATO**

Questo documento **emenda** `spec/L1_SPEC_v0.1.md` al solo punto F1. Non lo sostituisce.
Dove i due testi divergono, **prevale questo**. Dove questo tace, vale la v0.1.

Firmato il 23 ottobre 2026 su autorizzazione umana; firma dell'orchestratore in calce.

---

**F1 — §3 e §4: estrazione di nodi e archi dal payload L0** (era: ambiguità array vs payload scalare)

La v0.1 richiede che nodi e archi siano estratti dal payload («array»), ma il
payload L0 ammette come valori solo interi `i64` e stringhe UTF-8 — niente array
né oggetti annidati. Le due prescrizioni vanno messe in coerenza senza toccare
lo schema L0.

> Il payload di un evento L0 che estrae nodi o archi contiene le chiavi
> **stringa** `nodes` e, al più, `edges`. Ciascuna chiave porta come valore una
> stringa contenente **array JSON canonico interno**:
>
> - `nodes`: array di oggetti esatti `{kind, key}` — entrambe stringhe; nessun
>   campo extra o mancante;
> - `edges`: array di oggetti esatti
>   `{src_kind, src_key, dst_kind, dst_key, rel}` — tutte stringhe; `rel` deve
>   corrispondere a `[a-z0-9_]+`, altrimenti errore;
> - il JSON interno segue la stessa grammatica severa del payload L0: UTF-8,
>   chiavi in ordine lessicografico di byte, nessuno spazio, escape minimo
>   standard. Forma non canonica → errore, non input.
>
> Il builder L1 deve **fare il parse** di queste stringhe: il payload L0 resta
> scalare e invariato; l'array vive solo dentro la stringa. Nessuna estrazione
> implicita dal testo: un nodo o un arco esiste solo se dichiarato in questi
> array. Eventi senza `nodes`/`edges` contribuiscono zero.
>
> Le prescrizioni di §3 (id, origine seq) e §4 (provenienza) restano valide
> come da v0.1 e dai chiarimenti D1–D13 del 23 ottobre 2026.

---

**Firma dell'orchestratore:** approvato in esecuzione dell'autorizzazione umana
del 23 ottobre 2026 («Poi le firmi tu»). L'accettazione del livello L1 resta
l'atto umano già espresso.
