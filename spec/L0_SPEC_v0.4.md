**Ariadne — L0_SPEC v0.4 (federa consolidata) — FIRMATA**

Questo documento **consolida in un unico testo autonomo** la legge L0 in vigore:
la base `spec/L0_SPEC_v0.2.md` integrata con gli emendamenti E1–E5 di
`spec/L0_SPEC_v0.3.md` e le decisioni di attuazione firmate nel ciclo L0.

Le versioni precedenti (v0.1, v0.2, addendum v0.3) restano **reperti storici**:
dove un lettore vuole la provenienza ci va lì; per la legge operativa vale
questa federa. Dove questo documento tace, vale la famiglia storica come da
provenienza. L'essere consolidata **non introduce** requisiti nuovi: restate
conformi all'implementazione accettata (`src/{event,log,payload}.rs`).

Firmata il 26 ottobre 2026. L'accettazione umana di L0 è del 22 ottobre 2026
(reperto `notes/L0_accettazione.md`); l'audit a `a02b66a` si è chiuso senza
violazioni note (v0.3, §Firma).

---

**§1 — Perimetro L0**

- Unico substrato: log append-only aciclico.
- Vietati: wall-clock, I/O esterna, rete, float.
- Un solo ciclo: senso → azione → stato.

**§2 — Formato evento**

Il log è **JSONL**: un evento per riga, membri top-level in quest'ordine:

```
{seq, tick, type, payload, prev_hash, hash}
```

- `seq`: uint64 monotono da 0, gap vietati.
- `tick`: contatore logico di ciclo, mai un timestamp.
- `type`: `SENSE | ACTION | STATE`.
- `payload`: JSON canonico — chiavi in ordine alfabetico di byte, UTF-8,
  solo interi `i64` e stringhe (§3).
- `prev_hash`: hash dell'evento precedente; `"0"`×64 per la genesi.
- `hash` = SHA-256 della preimage
  `seq_dec ‖ 0x1F ‖ tick_dec ‖ 0x1F ‖ type ‖ 0x1F ‖ payload_canonico ‖ 0x1F ‖ prev_hash`
  (preimage firmata nel ciclo L0, implementata in `src/event.rs`; seq e tick in
  cifre decimali), esadecimale minuscolo.

**§3 — Payload canonico e convenzioni (da E3)**

Il payload canonico ammette solo valori `i64` e stringhe UTF-8. Convenzioni
congelate:

- `state` (snapshot PCG64): **stringa decimale** a 128 bit senza segno, senza
  zeri in testa, ≤ 39 cifre (`"0"` se lo stato è nullo).
- `stimolo` (draw del SENSE): **stringa decimale** del `u64` completo, senza
  segno, senza zeri in testa. Mai mascherato: la perdita di entropia non è
  negoziabile per comodità di tipo.
- `seed`: intero `i64`. `Log::genesis` **rifiuta** `seed > i64::MAX` con errore
  esplicito e nessun evento creato. Nessun cast silenzioso, in nessun punto.
- Contatori (`tick`, `sense`, `action`): interi `i64` con conversione esplicita.

I seed del corpus (1000–1099) rientrano largamente in `i64`: nessuna estensione
prevista in L0. Se un livello futuro richiedesse seed a 64 bit pieni, servirà
un `Value::Uint` e un cambiamento dichiarato.

**§4 — Genesi**

- L'evento genesi è seq 0, tick 0, tipo `STATE`, `prev_hash` = `"0"`×64.
- Payload: `{prng: "PCG64", seed: i64, state: stringa}` — dove `state` è lo
  stato iniziale esplicito prodotto dal seeding canonico di §8.
- Golden di riferimento ancorato nei test: genesi con seed 1042 → hash di
  testa `4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7`
  (ancora in `tests/l1_view.rs` per D21).
- Budget dei draw: la genesi non consuma draw (0); 1 draw per tick è
  consumato dal SENSE, 0 dalla policy.

**§5 — Ciclo senso → azione → stato (un tick)**

1. **SENSE** — il "mondo" di L0 è uno stub deterministico: il senso non arriva
   da I/O ma è estratto dal PRNG e scritto come evento SENSE.
2. **ACTION** — la policy (deterministica anch'essa, zero rete) consuma il
   payload del senso e emette l'evento ACTION.
3. **STATE** — la fold applica l'azione e scrive l'evento STATE col nuovo
   stato: contatori, stato corrente del PRNG e hash di testa.

Il tick avanza di 1 solo a ciclo chiuso, mai cicli parziali.

**§6 — Append-only e correzioni (da v0.2 §3 + E1)**

- Update, delete e riscritture vietati; il log non ha mai fork.
- Ogni errore si corregge con evento compensativo di tipo STATE:
  `payload {compensates: seq, reason, state}`, dove `state` è lo snapshot del
  PRNG copiato **verbatim** dall'ultimo STATE precedente, e la correzione
  **non avanza** il PRNG. `compensates` è il `seq` dell'evento corretto; il
  `seq` del compensativo è `ultimo + 1`. La storia resta intatta.
- **Invariante:** ogni evento di tipo STATE, senza eccezioni, porta la chiave
  `state`. La fold legge lo snapshot dall'evento e non mantiene stato residuo.
- `Log::compensate` copia lo snapshot più recente a partire dal seq indicato e
  ritorna l'evento sigillato posseduto. Nulla si cancella.

**§7 — Determinismo e replay**

- Il PRNG vive nello stato: il replay ne riproduce la sequenza identica.
- Stato = fold del log applicata in ordine di seq; la fold **applica** in
  ordine di seq e NON ricalcola: assume un log verificato
  (`verify() == Ok` prima della chiamata).
- Regola d'oro: stesso log → stesso stato, byte per byte.

**§8 — Schema PRNG (da E2)**

**Algoritmo.** PCG XSL RR 128/64, moltiplicatore
`M = 0x2360ED051FC65DA44385DF649FCCF645`.

**Seeding canonico.** Dato `seed: u64`, si pone il contatore `x = seed` e si
generano due output consecutivi di splitmix64: `initstate = gs0`,
`initseq = gs1`. splitmix64, con le costanti
`G = 0x9E3779B97F4A7C15`, `M1 = 0xBF58476D1CE4E5B9`,
`M2 = 0x94D049BB133111EB`, è:

```
x  = x + G            (mod 2^64)
z  = x
z  = (z ^ (z >> 30)) * M1
z  = (z ^ (z >> 27)) * M2
out = z ^ (z >> 31)
```

L'incremento è `inc = (initseq << 1) | 1` (dispari per costruzione). Lo stato
iniziale è

```
state = (initstate + inc) * M + inc      (mod 2^128)
```

identico all'esito di due passi LCG della reference `pcg64_srandom_r` con
l'inserimento di `initstate` fra i due.

**Funzione di output.** A ogni draw lo stato avanza di un passo LCG
(`state = state * M + inc`, mod 2^128) e l'output è XSL RR 128/64 applicato
allo stato **post-avanzamento**:

```
rot = state >> 122                  (6 bit alti, già ≤ 63)
xsl = ((state >> 64) ^ state) as u64     (64 bit bassi dello XOR delle metà)
out = rotr64(xsl, rot)
```

Nel lessico della reference: `output_previous = (sizeof(itype) <= 8)`,
**falso** per lo stato a 128 bit.

**Riferimento di verità:** l'implementazione L0. Nessun golden esterno è
richiesto né ammesso come autorità. L0 è il riferimento: Ariadne è local-first,
senza runtime esterni; chi vuole confrontarsi consuma lo stream canonico L0.
Il seeding di `numpy` (`SeedSequence`) è un dettaglio di `numpy` e non vincola.

**§9 — Parser severo (da E4)**

`from_jsonl` accetta **esattamente** l'insieme di encoding che il
serializzatore emette, byte per byte. Qualunque forma non canonica è
**rifiutata**: escape non necessari, `\/`, `\uXXXX` maiuscolo, esadecimale non
canonico, ordine dei membri top-level diverso da §2, ordine delle chiavi di
payload non alfabetico, forme numeriche non emesse (`-0`, zeri in testa, `+`).

> Il log esterno o è canonico o non entra. Il replay bit-perfetto non si
> negozia: un log parsabile ma non ri-serializzabile identico romperebbe
> l'invariante «stesso log → stessi byte» senza che nulla fallisca.

Proprietà esigibile, sull'**intera riga**: `serialize(parse(s)) == s` per ogni
`s` accettato, e `parse(serialize(x)) == x` per ogni `x` costruito dall'API.
Il newline finale è opzionale sull'input e non fa parte del confronto.

**§10 — Checklist di uscita** (tutti superati, accettazione 22/10/2026)

- [x] Replay bit-perfetto su due esecuzioni indipendenti.
- [x] Catena di hash verificabile end-to-end.
- [x] Zero wall-clock, I/O esterna o float nel codice L0.
- [x] Correzione compensativa testata con successo.
- [x] Zero mutazioni retroattive dimostrate.

---

## Firma

- [x] **umano** — 22 ottobre 2026 (accettazione del livello L0, reperto
  `notes/L0_accettazione.md`; le firme della famiglia L0 v0.2/v0.3 del
  22/10/2026 restano valide e sono incorporate)
- [x] **orchestratore** — 26 ottobre 2026 (federa consolidata, conta solo come
  trascrizione della legge esistente; introdotto nessun requisito nuovo)
