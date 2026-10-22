**Ariadne — L0_SPEC v0.3 (addendum alla v0.2) — FIRMATO**

Questo documento **emenda** `spec/L0_SPEC_v0.2.md` nei punti E1–E5. Non lo sostituisce.
Dove i due testi divergono, **prevale questo**. Dove questo tace, vale la v0.2.

Firmato il 22 ottobre 2026. Firma umana e firma dell'orchestratore in calce.

---

**E1 — §3, payload del compensativo** (era: ambiguità con lo Schema PRNG)

La v0.2 prescrive `payload {compensates: seq, reason}` e, altrove, che *ogni* STATE
snapshotta il suo stato. Le due prescrizioni sono incompatibili quando l'evento
compensativo è di tipo STATE, ed è l'unico tipo ammissibile per una correzione.

> Ogni errore si corregge con evento compensativo di tipo STATE:
> `payload {compensates: seq, reason, state}`, dove `state` è lo snapshot del PRNG
> copiato **verbatim** dall'ultimo STATE precedente, e la correzione **non avanza** il
> PRNG. `compensates` è il `seq` dell'evento corretto; il `seq` del compensativo è
> `ultimo + 1`. La storia resta intatta.
>
> Il payload di §3 è da leggersi come *almeno* queste chiavi: `state` è aggiuntiva e
> obbligatoria.

**Invariante conseguente:** *ogni* evento di tipo STATE, senza eccezioni, porta la
chiave `state`. La fold legge lo snapshot dall'evento e non mantiene stato residuo.

**E2 — Schema PRNG, sequenza canonica**

La v0.2 non fissa la procedura di espansione del seed, e chiede «coerenza coi seed
1000–1099» senza dire coerenza con *quale* implementazione. **L0 è il riferimento.**
Ariadne è local-first, senza runtime esterni: chi vuole confrontarsi consuma lo stream
canonico L0. Il seeding di `numpy` (`SeedSequence`) è un dettaglio di `numpy` e non
vincola.

> **Algoritmo.** PCG XSL RR 128/64, moltiplicatore
> `M = 0x2360ED051FC65DA44385DF649FCCF645`.
>
> **Seeding canonico.** Dato `seed: u64`, si pone il contatore `x = seed` e si generano
> due output consecutivi di splitmix64: `initstate = gs0`, `initseq = gs1`. splitmix64,
> con le costanti `G = 0x9E3779B97F4A7C15`, `M1 = 0xBF58476D1CE4E5B9`,
> `M2 = 0x94D049BB133111EB`, è:
>
> ```
> x  = x + G            (mod 2^64)
> z  = x
> z  = (z ^ (z >> 30)) * M1
> z  = (z ^ (z >> 27)) * M2
> out = z ^ (z >> 31)
> ```
>
> L'incremento è `inc = (initseq << 1) | 1` (dispari per costruzione). Lo stato iniziale
> è
>
> ```
> state = (initstate + inc) * M + inc      (mod 2^128)
> ```
>
> identico all'esito di due passi LCG della reference `pcg64_srandom_r` con
> l'inserimento di `initstate` fra i due.
>
> **Funzione di output.** A ogni draw lo stato avanza di un passo LCG
> (`state = state * M + inc`, mod 2^128) e l'output è XSL RR 128/64 applicato allo stato
> **post-avanzamento**:
>
> ```
> rot = state >> 122                  (6 bit alti, già ≤ 63)
> xsl = ((state >> 64) ^ state) as u64     (64 bit bassi dello XOR delle metà)
> out = rotr64(xsl, rot)
> ```
>
> Nel lessico della reference: `output_previous = (sizeof(itype) <= 8)`, **falso** per lo
> stato a 128 bit.
>
> **Riferimento di verità:** l'implementazione L0. Nessun golden esterno è richiesto né
> ammesso come autorità.

**E3 — Rappresentazione nel payload** (convenzioni congelate)

Il payload canonico ammette solo interi `i64` e stringhe. Convenzioni:

- `state` (snapshot PCG64): **stringa decimale** a 128 bit senza segno, senza zeri in
  testa, ≤ 39 cifre (`"0"` se lo stato è nullo).
- `stimolo` (draw del SENSE): **stringa decimale** del `u64` completo, senza segno,
  senza zeri in testa. Mai mascherato: la perdita di entropia non è negoziabile per
  comodità di tipo.
- `seed`: intero `i64`. `Log::genesis` **rifiuta** `seed > i64::MAX` con errore
  esplicito e nessun evento creato. Nessun cast silenzioso, in nessun punto.
- Contatori (`tick`, `sense`, `action`): interi `i64` con conversione esplicita.

I seed del corpus (1000–1099) rientrano largamente in `i64`: nessuna estensione
prevista in L0. Se un livello futuro richiederà seed a 64 bit pieni, servirà un
`Value::Uint` e un cambiamento dichiarato.

**E4 — Parser severo**

`from_jsonl` accetta **esattamente** l'insieme di encoding che il serializzatore emette,
byte per byte. Qualunque forma non canonica è **rifiutata**: escape non necessari,
`\/`, `\uXXXX` maiuscolo, esadecimale non canonico, ordine dei membri top-level diverso
da §2, ordine delle chiavi di payload non alfabetico, forme numeriche non emesse (`-0`,
zeri in testa, `+`).

> Il log esterno o è canonico o non entra. Il replay bit-perfetto non si negozia: un log
> parsabile ma non ri-serializzabile identico romperebbe l'invariante «stesso log →
> stessi byte» senza che nulla fallisca.

Proprietà esigibile, sull'**intera riga**: `serialize(parse(s)) == s` per ogni `s`
accettato, e `parse(serialize(x)) == x` per ogni `x` costruito dall'API. Il newline
finale è opzionale sull'input e non fa parte del confronto.

**E5 — Cosa NON cambia**

Restano validi dalla v0.2: §1 perimetro, §2 formato evento, §4 determinismo, §5
checklist di uscita, il ciclo senso→azione→stato e il budget dei draw (1 draw per tick
consumato dal SENSE, 0 dalla policy, 0 dalla genesi).

---

## Firma

- [x] **umano** — 22 ottobre 2026
- [x] **orchestratore** — 22 ottobre 2026

Con questa firma E1–E5 entrano in vigore. L'audit di L0 a `a02b66a` si chiude **senza
violazioni note**: l'unica rilevata dal reviewer (§3, payload a tre chiavi contro le due
prescritte dalla v0.2) è risolta da E1, che ha rango di legge.
