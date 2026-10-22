**Ariadne — L0_SPEC v0.2**

> **Emendata da `spec/L0_SPEC_v0.3.md` (firmato, 22 ottobre 2026).** In caso di
> divergenza prevale la v0.3. Nota di sola segnalazione, nessun cambio di contenuto.

**1. Perimetro L0**
- Unico substrato: log append-only aciclico.
- Vietati: wall-clock, I/O esterna, rete, float.
- Un solo ciclo: senso → azione → stato.

**2. Formato evento**
{seq, tick, type, payload, prev_hash, hash}
- seq: uint64 monotono da 0, gap vietati.
- tick: contatore logico di ciclo, mai un timestamp.
- type: SENSE | ACTION | STATE.
- payload: JSON canonico — chiavi ordinate alfabeticamente, UTF-8, solo interi e stringhe.
- prev_hash: hash dell'evento precedente ("0"×64 per la genesi).
- hash = SHA-256(seq ‖ tick ‖ type ‖ payload ‖ prev_hash), esadecimale minuscolo.

**3. Append-only e correzioni**
- Update, delete e riscritture vietati; il log non ha mai fork.
- Ogni errore si corregge con evento compensativo: payload {compensates: seq, reason}, la storia resta intatta.

**4. Determinismo e replay**
- L'evento genesi (seq 0, STATE) contiene il seed del PRNG.
- Il PRNG vive nello stato: il replay ne riproduce la sequenza identica.
- Stato = fold del log applicato in ordine di seq.
- Regola d'oro: stesso log → stesso stato, byte per byte.

**5. Criteri di uscita L0 (checklist)**
- [ ] Replay bit-perfetto su due esecuzioni indipendenti.
- [ ] Catena di hash verificabile end-to-end.
- [ ] Zero wall-clock, I/O esterna o float nel codice L0.
- [ ] Correzione compensativa testata con successo.
- [ ] Zero mutazioni retroattive dimostrate.

**Ciclo senso→azione→stato (un tick)**
1. **SENSE** — il "mondo" di L0 è uno stub deterministico: il senso non arriva da I/O ma è estratto dal PRNG e scritto come evento SENSE.
2. **ACTION** — la policy (deterministica anch'essa, zero rete) consuma il payload del senso e emette l'evento ACTION.
3. **STATE** — la fold applica l'azione e scrive l'evento STATE col nuovo stato: contatori, stato corrente del PRNG e hash di testa.
Il tick avanza di 1 solo a ciclo chiuso, mai cicli parziali.

**Schema PRNG**
- Algoritmo: PCG64, coerente coi seed 1000–1099 già fissati nella L6_SPEC — coerenza totale col corpus.
- Evento genesi (seq 0, STATE): payload {seed, prng: "PCG64", state: iniziale esplicito}.
- Il PRNG vive nello stato: ogni STATE snapshotta il suo stato, così il replay verifica per confronto invece di fidarsi della riesecuzione.
- Ordine dei draw fissato per contratto: prima i SENSE, poi eventuali draw della policy, budget draw per tick esplicito — niente dipendenze nascoste.

