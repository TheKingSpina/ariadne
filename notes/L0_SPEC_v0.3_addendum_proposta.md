# L0_SPEC v0.3 — addendum di proposta

> **STATO: PROMOSSA E FIRMATA — conservata per tracciabilità.**
> Il testo in vigore è **`spec/L0_SPEC_v0.3.md`**, firmato il 22 ottobre 2026 (umano +
> orchestratore). Questo file resta come traccia della proposta: non è legge e non va
> citato come tale. La storia non si riscrive; il percorso si conserva.
>
> Scopo: chiudere l'ambiguità emersa in fase di gate, senza riscrivere la v0.2.

## E1 — §3, payload del compensativo (era: ambiguità con lo Schema PRNG)

La v0.2 prescrive `payload {compensates: seq, reason}` e, altrove, che *ogni* STATE
snapshotta il suo stato. Le due prescrizioni sono incompatibili quando l'evento
compensativo è di tipo STATE, ed è l'unico tipo ammissibile per una correzione.

**Testo emendato:**

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

## E2 — Schema PRNG, sequenza canonica

La v0.2 non fissa la procedura di espansione del seed, e chiede "coerenza coi seed
1000–1099" senza dire coerenza con *quale* implementazione. **Deciso: L0 è il
riferimento.** Ariadne è local-first, senza runtime esterni: chi vuole confrontarsi
consuma lo stream canonico L0. Il seeding di `numpy` (`SeedSequence`) è un dettaglio
di `numpy` e non vincola.

**Testo emendato (Schema PRNG):**

> **Seeding canonico.** Dato `seed: u64`, si generano due output consecutivi di
> splitmix64 a partire dal contatore `x = seed`: il primo è `initstate`, il secondo
> `initseq`. L'incremento è `(initseq << 1) | 1` (dispari per costruzione). Lo stato
> iniziale è ottenuto con due passi LCG con l'inserimento di `initstate` in mezzo
> (procedura `pcg64_srandom_r` del riferimento C++).
>
> **Funzione di output.** XSL RR 128/64 applicata allo stato **post-avanzamento**
> (nel riferimento: `output_previous = (sizeof(itype) <= 8)`, falso per lo stato a
> 128 bit).
>
> **Algoritmo.** PCG XSL RR 128/64, moltiplicatore
> `0x2360ED051FC65DA44385DF649FCCF645`.
>
> **Riferimento di verità:** l'implementazione L0. Nessun golden esterno è richiesto
> né ammesso come autorità.

## E3 — Rappresentazione nel payload (convenzioni congelate)

Il payload canonico ammette solo interi `i64` e stringhe. Convenzioni:

- `state` (snapshot PCG64): **stringa decimale** a 128 bit senza segno, senza zeri in
  testa, ≤ 39 cifre.
- `stimolo` (draw del SENSE): **stringa decimale** del `u64` completo, senza segno,
  senza zeri in testa. Mai mascherato: la perdita di entropia non è negoziabile per
  comodità di tipo.
- `seed`: intero `i64`. `Log::genesis` **rifiuta** `seed > i64::MAX` con errore
  esplicito e nessun evento creato. Nessun cast silenzioso, in nessun punto.
- Contatori (`tick`, `sense`, `action`): interi `i64` con conversione esplicita.

I seed del corpus (1000–1099) rientrano largamente in `i64`: nessuna estensione
prevista in L0. Se un livello futuro richiederà seed a 64 bit pieni, servirà un
`Value::Uint` e un cambiamento dichiarato.

## E4 — Parser severo

`from_jsonl` accetta **esattamente** l'insieme di encoding che il serializzatore
emette, byte per byte. Qualunque forma non canonica (escape non necessari, `\/`,
`\uXXXX` maiuscolo, esadecimale non canonico) è **rifiutata**.

> Il log esterno o è canonico o non entra. Il replay bit-perfetto non si negozia:
> un log parsabile ma non ri-serializzabile identico romperebbe l'invariante
> "stesso log → stessi byte" senza che nulla fallisca.

Proprietà esigibile: `serialize(parse(s)) == s` per ogni `s` canonico, e
`parse(serialize(x)) == x` per ogni `x` costruito dall'API.

## E5 — Cosa NON cambia

Restano validi dalla v0.2: §1 perimetro, §2 formato evento, §4 determinismo, §5
checklist di uscita, il ciclo senso→azione→stato e il budget dei draw (1 draw per
tick consumato dal SENSE, 0 dalla policy, 0 dalla genesi).

## Firma

- [ ] umano
- [ ] orchestratore

---

## Promemoria per il registro (non fa parte dell'addendum)

Il gate di L0 ha chiuso con un rosso **processuale** e nessun rosso funzionale:

- la coppia `08b1536 → b2790e9` (ciclo SENSE→ACTION→STATE) ha un test che non ha mai
  compilato contro l'implementazione: il verde arriva in `a298cf9` modificando il
  test. La proprietà test-first di L0 è **indebolita**, non falsificata: le altre
  cinque coppie hanno diff dei test vuoto.
- Sono due i test corretti dopo l'implementazione: `a298cf9` (import) e `45ad98d`
  (aspettativa aritmetica errata). In entrambi il difetto è del test, non del codice.
- La storia non si riscrive per cancellarli. Il valore dell'esercizio è che il
  cedimento è registrato.
