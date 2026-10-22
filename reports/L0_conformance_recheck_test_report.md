# L0 — Riverifica mirata di §3 dopo C1/C2 (compensativi autodescrittivi + `genesis` fallibile)

- HEAD verificato: `238d586` (`git log --oneline`: `238d586 L0 C1-fix2: il CLI non deve panicare su input invalido …`)
- SPEC: `spec/L0_SPEC_v0.2.md` (v0.2)
- Data: 2026-10-22
- Ambito: **sostituisce §3** del precedente `reports/L0_conformance_test_report.md` (chiuso su `5b495d7`), la cui §3 certificava un payload compensativo `{compensates, reason}` **che non esiste più**. Non si rifanno §1, §2, §4.
- Commit nuovi: `7f76fea` (test rossi) → `649deea` (implementazione) → `45ad98d` (fix aspettativa, 1 carattere) → `238d586` (CLI).
- Nessuna correzione di codice o test. Nessun file creato fuori da `reports/`.

## Metodo e ambiente

1. Checkout pulito: worktree dedicato a HEAD
   `git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-recheck HEAD`
   → `Preparing worktree (detached HEAD 238d586)` / `HEAD is now at 238d586 …`.
2. Prima dei test, nel worktree: `git status --porcelain` → **output vuoto** (checkout pulito).
3. Suite completa nel worktree pulito: `cargo test --all-targets` (debug) e `cargo test --release --all-targets` → **57 test, 0 failed** in entrambi (output integrali in Appendice A/B).
4. Replay §5 rifatto con due esecuzioni indipendenti del binario (output in §5).
5. Worktree rimosso a fine verifica (vedi "Pulizia").

Nota di sandbox: `printf`, `echo`, `sed`, `python3` e le **pipe** (`|`, process substitution) sono
rifiutati ("Permission denied: shell"); `git`, `cargo`, `rg`, `shasum`, `wc`, `ls` e la
**redirezione `>`** hanno funzionato. Ogni esito che avrebbe richiesto una pipe è stato ottenuto per
via alternativa (file di byte scritti in `reports/` e hashati con `shasum`); nessun esito dipende da
un comando negato.

---

## 1. §3 — compensazione (payload autodescrittivo)

| # | Voce | Esito | Prova |
|---|------|-------|-------|
| 1.1 | payload esatto `{compensates, reason, state}`: le tre chiavi, nient'altro | **VERDE** | `src/log.rs:141-145`; `tests/l0_event_log.rs:354-358`; `tests/l0_event_log.rs:389-393` |
| 1.2 | `compensates` = `seq` dell'evento compensato; `seq` del compensativo = ultimo+1 | **VERDE** | `src/log.rs:110`; `src/log.rs:146`; `tests/l0_event_log.rs:351`; `tests/l0_event_log.rs:387` |
| 1.3 | il compensativo NON avanza il PRNG: snapshot dopo == snapshot prima | **VERDE** | `tests/l0_state.rs:210-231` (`test_compensation_does_not_advance_prng`) |
| 1.4 | storia intatta: prefisso invariato byte per byte; evento compensato presente e non riscritto | **VERDE** | `src/log.rs:88-99` (append-only); `tests/l0_event_log.rs:360-365`; `tests/l0_event_log.rs:397-401`; `tests/l0_event_log.rs:417-419` |
| 1.5 | nessuna API di update/delete/riscrittura/riordino | **VERDE** | grep `rg -n 'fn (update\|delete\|…)' src/` → nessun match (sotto) |

### 1.1 — payload esatto in tre chiavi

`src/log.rs:141-145` costruisce il payload del compensativo:

```
141:        let payload = Payload::from_pairs(vec![
142:            ("compensates", Value::Int(comp_int)),
143:            ("reason", Value::Str(reason.to_string())),
144:            ("state", Value::Str(snapshot)),
145:        ]);
```

`Payload::from_pairs` produce un `BTreeMap` con solo queste tre chiavi; la serializzazione canonica
(`src/payload.rs:84-104`) ne ordina le chiavi alfabeticamente → `{"compensates":…,"reason":…,"state":…}`.
Nessuna quarta chiave: il costruttore non ne riceve altre.

Prova di asserzione (payload **esatto**, confronto `assert_eq!` sull'intera stringa canonica):
- `tests/l0_event_log.rs:354-358`:
  ```
  assert_eq!(
      comp.payload.canonical(),
      r#"{"compensates":1,"reason":"gain errato: il valore era 6","state":"43149974864253806286336044685702662359"}"#,
      "payload compensativo: compensates seq, reason, E lo snapshot dell'ultimo STATE (opzione A)"
  );
  ```
- `tests/l0_event_log.rs:389-393`:
  ```
  assert_eq!(
      comp.payload.canonical(),
      r#"{"compensates":1,"reason":"stimolo errato: era 42","state":"918273645040506070809101112131415161718"}"#,
      "payload compensativo in tre chiavi: STATE più RECENTE, non genesi"
  );
  ```
- `tests/l0_event_log.rs:434-438` (`test_immutable_events_slice_borrows`).

Entrambe verdi nella suite (Appendice A/B).

### 1.2 — `compensates` e `seq` del compensativo

- Il compensato è identificato dal `seq` passato come primo argomento:
  `src/log.rs:109-115` — `compensate(&mut self, compensates: u64, …)`, con rifiuto se
  `compensates as usize >= self.events.len()` (seq fuori dal log).
- Il `seq` del compensativo è `ultimo + 1` perché `compensate` delega a `append`
  (`src/log.rs:146`), e `append` calcola `seq = prev.seq.checked_add(1)`
  (`src/log.rs:92-95`).
- Asserzioni: `tests/l0_event_log.rs:351` (`assert_eq!(comp.seq, 2)` su gen 0 + ACTION 1);
  `tests/l0_event_log.rs:387` (`assert_eq!(comp.seq, 3, "… seq = ultimo + 1 = 3 …")` su
  gen 0 + SENSE 1 + STATE 2).

### 1.3 — il compensativo non avanza il PRNG

`tests/l0_state.rs:210-231` (`test_compensation_does_not_advance_prng`):

```
215:    let before = State::fold(&log).unwrap();
217:    let comp = log.compensate(1, "stimolo errato", 1).unwrap();
218:    assert_eq!(
219:        comp.payload.get("state"),
220:        Some(&Value::Str(SNAP.to_string())),
221:        "lo snapshot del compensativo è copia dell'ultimo STATE, NON un nuovo draw"
222:    );
225:    let after = State::fold(&log).unwrap();
226:    assert_eq!(after.prng, before.prng, "il PRNG NON avanza: correzione senza draw");
230:    assert_eq!(after.head, comp.hash, "solo head avanza: c'è un nuovo evento in coda");
```

Verde: `test test_compensation_does_not_advance_prng ... ok`. Lo snapshot del compensativo è una
**copia verbatim** (`src/log.rs:125-144`: scansione all'indietro che seleziona l'ultimo evento
`STATE` con `state` e ne clona la stringa); nessuna chiamata a `Pcg64` in `src/log.rs`
(il modulo importa `Pcg64` solo per la genesi, `src/log.rs:29`, `src/log.rs:78`).

### 1.4 — storia intatta

- Append-only strutturale: `append` fa solo `self.events.push(...)` (`src/log.rs:97`); `events()`
  presta `&[Event]` in sola lettura (`src/log.rs:150-152`).
- Asserzioni "prefisso invariato byte per byte":
  - `tests/l0_event_log.rs:360-362` (`after.starts_with(prefix_before)`),
  - `tests/l0_event_log.rs:397-399`,
  - `tests/l0_event_log.rs:417-419`.
- Evento compensato ancora presente e **non riscritto**:
  - `tests/l0_event_log.rs:365` (`log.events()[1].payload.canonical() == r#"{"gain":5}"#`),
  - `tests/l0_event_log.rs:400-401` (`log.events()[2].payload… == r#"{"state":"918…718"}"#`).
- Catena intera riverificata: `tests/l0_event_log.rs:402`, `:367`, `:420`.

Tutte verdi (Appendice A/B).

### 1.5 — nessuna API di update/delete/riscrittura/riordino

Comando (sul substrato):

```
$ rg -n 'fn (update|delete|remove|rewrite|reorder|truncate|swap|sort|insert_at|retain|drain|clear)' src/
(no output)
Exited with code 1
```

Superficie pubblica di `Log` (`rg -n 'pub fn' src/log.rs`):

```
src/log.rs:71:    pub fn genesis(seed: u64) -> Result<Log, Error> {
src/log.rs:90:    pub fn append(&mut self, tick: u64, r#type: EventType, payload: Payload) -> Result<&Event, Error> {
src/log.rs:109:    pub fn compensate(&mut self, compensates: u64, reason: &str, tick: u64) -> Result<Event, Error> {
src/log.rs:150:    pub fn events(&self) -> &[Event] {
src/log.rs:157:    pub fn verify(&self) -> Result<(), Error> {
src/log.rs:185:    pub fn to_jsonl(&self) -> String {
src/log.rs:197:    pub fn from_jsonl(input: &str) -> Result<Log, Error> {
```

Nessun `update`/`delete`/`remove`/`rewrite`/`reorder`/`truncate`/`swap`/`sort`; l'unico mutatore è
`append`-family (in coda). `Log.events` è campo privato (`src/log.rs:59-61`). **VERDE.**

---

## 2. Invariante «ogni STATE è autodescrittivo»

| # | Voce | Esito | Prova |
|---|------|-------|-------|
| 2.1 | su un log reale (genesi + N cicli + compensazione) **ogni** `STATE` porta `state` | **VERDE** | `tests/l0_state.rs:284-319` (`test_real_log_every_state_carries_snapshot_and_fold_agrees`) |
| 2.2 | il caso speciale è sparito da `src/state.rs` (non solo i test passano) | **VERDE** | diff `649deea -- src/state.rs`; `src/state.rs:162-177` |
| 2.3 | la fold legge lo snapshot dall'evento | **VERDE** | `src/state.rs:168-177` |

### 2.1

`tests/l0_state.rs:284-319` costruisce **genesi 1042 + 3 cicli + compensazione dell'ultimo STATE**,
poi:

```
312:    for e in log.events() {
313:        if e.r#type == EventType::State {
314:            assert!(
315:                matches!(e.payload.get("state"), Some(Value::Str(_))),
316:                "EVENTO STATE non autodescrittivo alla seq {}: {}", e.seq, e.payload.canonical()
317:            );
318:        }
319:    }
```

Nessuna eccezione per la genesi né per il compensativo: l'invariante vale su **tutti** gli STATE.
Verde: `test test_real_log_every_state_carries_snapshot_and_fold_agrees ... ok`.

### 2.2 — il caso speciale è davvero rimosso

`git show 649deea -- src/state.rs` mostra la sostituzione nel ramo `EventType::State`:

```
-                    // Solo uno STATE che PORTE uno snapshot aggiorna `prng`;
-                    // un compensativo {compensates, reason} non ne porta uno e
-                    // lascia vivo l'ultimo snapshot visto (vedi mod-doc).
-                    if let Some(Value::Str(s)) = e.payload.get("state") {
-                        prng = s.clone();
-                    }
+                    prng = match e.payload.get("state") {
+                        Some(Value::Str(s)) => s.clone(),
+                        _ => {
+                            return Err(StateError::Snapshot(format!(
+                                "evento STATE a seq {} senza `state` stringa nel payload: {}",
+                                e.seq,
+                                e.payload.canonical()
+                            )))
+                        }
+                    };
```

Il vecchio comportamento «ultimo visto» (un `if let` che lasciava vivo il valore precedente quando
`state` mancava) è **eliminato**: ora l'assenza di `state` è un errore rumoroso
(`StateError::Snapshot`, nuova variante `src/state.rs:63-68`). Nel file corrente la stringa
«ultimo visto» compare **solo nei commenti che ne documentano la rimozione**
(`src/state.rs:29`, `:66`, `:166-167`); nessun ramo di fallback:

```
$ rg -n 'ultimo_visto|ultimo visto|last_seen|ricord' src/state.rs
src/state.rs:29://!   porta `state`: la fold non ricorda più un «ultimo visto», lo snapshot è
src/state.rs:66:    /// conciliazione con un «ultimo visto»: niente stato è derivabile da uno
src/state.rs:166:                    // «ultimo visto»: uno STATE senza `state` stringa è un
src/state.rs:167:                    // errore rumoroso, non un silenzio da colmare col ricordo.
```

### 2.3 — lo snapshot è letto dall'evento

`src/state.rs:168-177`: `prng = match e.payload.get("state") { Some(Value::Str(s)) => s.clone(), _ => Err(Snapshot) }`.
Lo snapshot viene aggiornato **da ciò che l'evento porta**, non da un ricordo dello stato precedente;
`prng` non è mai ricalcolato (nessuna chiamata a `Pcg64` in `src/state.rs`; import a `src/state.rs:45-49`
privi di `prng`). Coerente con `src/state.rs:8-12` (mod-doc).

---

## 3. `genesis` fallibile

| # | Voce | Esito | Prova |
|---|------|-------|-------|
| 3.1 | `seed == i64::MAX` → ok | **VERDE** | `tests/l0_c1_genesis.rs:27-43` (`test_genesis_accepts_seed_i64_max`) |
| 3.2 | `seed == i64::MAX + 1` → errore, e **nessun evento** nel percorso `Err` | **VERDE** | `tests/l0_c1_genesis.rs:46-59`; `src/log.rs:71-85` |
| 3.3 | il seed resta un intero nel payload per i valori validi | **VERDE** | `tests/l0_c1_genesis.rs:33-42`; `tests/l0_event_log.rs:93-95` |

### 3.1 / 3.3

`tests/l0_c1_genesis.rs:29-42`:

```
29:    let log = Log::genesis(SEED_I64_MAX).expect("seed == i64::MAX è rappresentabile");
30:    assert_eq!(log.events().len(), 1, "la genesi è il solo evento");
31:    log.verify().expect("la genesi al confine è verificabile");
33:    assert_eq!(
34:        log.events()[0].payload.get("seed"),
35:        Some(&Value::Int(i64::MAX)),
36:        "il seed al confine entra nel payload ESATTAMENTE com'è"
37:    );
38:    let payload = log.events()[0].payload.canonical();
39:    assert!(
40:        payload.contains(r#""seed":9223372036854775807,"#),
41:        "seed al confine in forma canonica: {payload}"
42:    );
```

Il seed è `Value::Int` (intero), al confine esatto `9223372036854775807`. Anche la genesi golden usa
seed intero (`tests/l0_event_log.rs:93-95`: `"seed":1042`). Verde in debug e release.

### 3.2

`tests/l0_c1_genesis.rs:47-59`:

```
48:    let oltre = SEED_I64_MAX.checked_add(1).expect("2^63 è rappresentabile in u64");
49:    let err = match Log::genesis(oltre) {
50:        Ok(_) => panic!("seed 2^63 accettato: era esattamente il wrap silenzioso da vietare"),
51:        Err(e) => e,
52:    };
54:    let msg = format!("{err}");
55:    assert!(msg.contains("seed"), "il messaggio manda il fault al seed: {msg}");
56:    assert!(msg.contains("i64"), "il messaggio riferisce il limite i64: {msg}");
```

Verde: `test test_genesis_rejects_seed_beyond_i64_max_no_event_no_wrap ... ok`.

**Prova strutturale che nel percorso `Err` non esce alcun evento** (`src/log.rs:71-85`): il rifiuto
avviene **prima** di costruire payload ed evento —

```
71:    pub fn genesis(seed: u64) -> Result<Log, Error> {
72:        let seed_int = i64::try_from(seed).map_err(|_| {
73:            Error::Seed(format!(
74:                "seed {seed} oltre i64::MAX: nel payload canonico il seed resta intero i64 \
75:                 (contatore, non entropia), nessun cast silenzioso"
76:            ))
77:        })?;
78:        let state = Pcg64::new(seed).state_decimal();
79:        let payload = Payload::from_pairs(vec![ … ]);
84:        let genesis = Event::seal(0, 0, EventType::State, payload, GENESIS_PREV_HASH);
85:        Ok(Log { events: vec![genesis] })
86:    }
```

`i64::try_from(seed)?` ritorna `Err(Error::Seed)` **prima** di `Event::seal` e prima di costruire
`Log`: nessun evento è creato. Il tipo `Result<Log, Error>` non può trasportare un `Log` sul ramo
`Err`. (Il precedente cast `seed as i64` — che per `2^63` produceva `"seed":-9223372036854775808`
osservato sul gate precedente — è rimosso.)

---

## 4. Golden della genesi invariato

**Esito: VERDE** (riprova indipendente con `shasum`).

La costante nel test è `4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7`
(`tests/l0_event_log.rs:29`). Il preimage documentato (SPEC §2, separatore `0x1F`) è:

```
0x1F-separato:  seq="0" ‖ 0x1F ‖ tick="0" ‖ 0x1F ‖ type="STATE" ‖ 0x1F ‖
                payload='{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"}' ‖
                0x1F ‖ prev_hash = 64 zeri
```

Il preimage è stato scritto **byte per byte** in `reports/_l0_recheck_preimage_1042.bin` (152 byte,
verificato con `wc -c`) e hashato dall'esterno del crate:

```
$ wc -c reports/_l0_recheck_preimage_1042.bin
     152 reports/_l0_recheck_preimage_1042.bin

$ shasum -a 256 reports/_l0_recheck_preimage_1042.bin
4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7  reports/_l0_recheck_preimage_1042.bin
```

**64/64 caratteri coincidono** con la costante. Conferma del valore nel test:
`tests/l0_event_log.rs:29` e asserzione `tests/l0_event_log.rs:108` (`assert_eq!(log.events()[0].hash, GENESIS_1042_HASH)`),
verde (`test test_genesis_golden_hash_constant ... ok`, Appendice A/B).

Precedenza della fixture all'implementazione (regola "fixture valida se precede l'implementazione"):

```
$ git log -S '4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7' --oneline -- tests/l0_event_log.rs
d2f1610 L0 B2: test ROSSI per genesi con stato PCG64 stringa (chiude D1)
```

La costante è stata introdotta nel commit **di soli test** `d2f1610`, **prima** dell'implementazione
`b7543f7` (B2). Il golden non è circolare: nessuna funzione del crate è stata usata per calcolarlo
(qui: `shasum` su byte scritti a mano).

NOTA (non ROSSO): la porzione `state` del preimage è a sua volta prodotta dal crate
(`Pcg64::new(1042).state_decimal()`, `tests/l0_event_log.rs:31-34`): l'hash la *congela* ma non ne
dimostra la correttezza contro un riferimento esterno — già segnalato dal gate precedente
(`reports/L0_prng_genesis_test_report.md` §6) e fuori dallo scope di questa riverifica.

---

## 5. §5 — replay bit-perfetto a due processi, rifatto

**Esito: VERDE** — il digest atteso dal gate precedente **coincide**.

Worktree pulito a HEAD `238d586`. Due esecuzioni **separate** del binario (due processi distinti),
stessi argomenti, stdout catturato:

```
cargo run --quiet --bin l0_replay -- --seed 1042 --ticks 5 > /Users/spina/Ariadne/reports/l0_recheck_replay_a.jsonl
cargo run --quiet --bin l0_replay -- --seed 1042 --ticks 5 > /Users/spina/Ariadne/reports/l0_recheck_replay_b.jsonl
```

Digest:

```
967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784  /Users/spina/Ariadne/reports/l0_recheck_replay_a.jsonl
967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784  /Users/spina/Ariadne/reports/l0_recheck_replay_b.jsonl
```

I due digest **coincidono** tra loro e **coincidono con `967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784`**,
il valore atteso dal gate precedente per `--seed 1042 --ticks 5`. Dimensione identica:

```
$ wc -l -c reports/l0_recheck_replay_a.jsonl reports/l0_recheck_replay_b.jsonl
      16    3882 reports/l0_recheck_replay_a.jsonl
      16    3882 reports/l0_recheck_replay_b.jsonl
      32    7764 total
```

(16 righe = genesi + 3 eventi × 5 tick.)

**Spiegazione della coincidenza**: il bug-fix di C1/C2 **non ha toccato** i byte del log per
seed/tick normali. Il percorso del binario è `Log::genesis(1042)` → `cycle::run_cycles(5)` →
`to_jsonl()`: (a) la genesi con seed 1042 è invariata (payload `{prng,seed:1042,state}` con la stessa
stringa `state`, `src/log.rs:78-84`), e (b) i payload SENSE/ACTION/STATE di ciclo non cambiano
(`src/cycle.rs:100-133`); (c) **non ci sono compensativi** nel log generato dal binario, quindi la
nuova chiave `state` nel payload compensativo non compare; (d) nessuna modifica al preimage/hash.
Il log è quindi byte-identico al gate precedente. Il test in-suite
`due_processi_stessi_argomenti_stdout_byte_identico` (`tests/l0_replay_process.rs:49-63`) è verde
(Appendice A/B).

---

## 6. Suite su worktree pulito all'HEAD

**Esito: VERDE** — 57 test, 0 failed in debug **e** in release.

- Debug: `cargo test --all-targets` → 4 (lib) + 0 (bin) + 2 (l0_c1_genesis) + 11 (l0_cycle) +
  18 (l0_event_log) + 7 (l0_prng) + 6 (l0_replay_process) + 9 (l0_state) = **57 passed; 0 failed**.
- Release: `cargo test --release --all-targets` → **57 passed; 0 failed**.

Output integrali in Appendice A (debug) e Appendice B (release). Il worktree era pulito prima di
ogni build (`git status --porcelain` → vuoto).

---

## 7. Due test modificati dopo l'implementazione

### 7.1 — `a298cf9` (B4: il test non compilava)

Diff completo (`git show a298cf9`):

```
 tests/l0_cycle.rs | 3 ++-
 1 file changed, 2 insertions(+), 1 deletion(-)

-use ariadne::{cycle, EventType, Log, Pcg64, State};
+use ariadne::prng::Pcg64;
+use ariadne::{cycle, EventType, Log, State};
```

- **Il test nella forma precedente ha mai girato verde contro l'implementazione? NO.** A `b2790e9`
  (implementazione del ciclo presente) il test non **compila**: `E0432: unresolved import 'ariadne::Pcg64'`
  (il crate non ri-esporta `Pcg64` a radice, sta in `ariadne::prng`). Senza compilazione non c'è
  esecuzione, quindi nessun verde. Il primo momento in cui i test del ciclo compilano è `a298cf9`,
  **dopo** l'implementazione, grazie a una modifica al test.
- **Allentamento? NO.** La modifica è puramente sintattica (import): 2 righe aggiunte / 1 rimossa,
  1 token spostato di modulo (`prng::`), **0 asserzioni toccate, 0 vincoli allentati**.
- Classificazione: **difetto del test (import errato), non dell'implementazione**; la prova
  test-first di B4 è indebolita (nessun commit in cui il test immutato passi rosso→verde per sola
  implementazione). Conferma il precedente `reports/L0_commits_test_report.md` §5.5/§7.1
  (che su B4 parlava di ROSSO per archeologia; fuori ambito qui, ma coerente).

### 7.2 — `45ad98d` (C1: aspettativa aritmeticamente errata)

Diff completo (`git show 45ad98d`):

```
-    assert_eq!(comp.seq, 2);
+    assert_eq!(comp.seq, 3, "l'evento compensativo è IN CODA: seq = ultimo + 1 = 3 (compensava il SENSE seq 1)");
```

- **Il test nella forma precedente ha mai girato verde contro l'implementazione? NO.**
  Il log del test è `genesi(seq 0) + SENSE(seq 1) + STATE(seq 2)` = 3 eventi; `compensate` appende in
  coda → `seq = ultimo + 1 = 3`. L'aspettativa `comp.seq == 2` non può mai coincidere.
  - A `7f76fea` (test-only) il file `tests/l0_event_log.rs` compilava (genesi ancora infallibile:
    `let mut log = Log::genesis(1042);`, riga 380 del diff di `7f76fea`), ma l'intero build di test
    **falliva** per il compile-fail di `tests/l0_c1_genesis.rs` (stessa commit): nessuna asserzione
    fu eseguita. Inoltre il payload era ancora a due chiavi.
  - A `649deea` (implementazione) la suite compila tutta e la riga `comp.seq == 2` fallisce
    (il valore reale è 3): **prima esecuzione = ROSSA**.
  - Verde solo da `45ad98d` in poi.
- **Allentamento? NO.** La modifica cambia un literal di attesa da `2` a `3` e aggiunge un terzo
  argomento descrittivo ad `assert_eq!`; **l'asserzione sostanziale non è toccata** (payload in tre
  chiavi, storia intatta). È una **correzione di un'aspettativa falsa**, non un allentamento.
- Classificazione: **bug del test introdotto in `7f76fea`**, non dell'implementazione. La
  correzione è a un solo carattere di valore.

### 7.3 — Sintesi delle due modifiche

| Commit | Causa | Il test pre-fix è mai stato verde? | Allentamento? | Natura |
|---|---|---|---|---|
| `a298cf9` | import da radice anziché `ariadne::prng` | **NO** (non compilava) | **NO** | sintattica (import) |
| `45ad98d` | aspettativa `comp.seq == 2` errata (reale: 3) | **NO** (rossa a `649deea`) | **NO** | correzione di aspettativa (1 carattere) |

In **entrambi** i casi il test, nella forma precedente, **non è mai girato verde contro
l'implementazione**, e **non c'è allentamento**: in un caso la modifica è solo l'import, nell'altro
solo il literal atteso.

---

## Elenco ROSSI

**Nessun ROSSO.** Tutte le voci di questa riverifica (§3 compensazione, invariante ogni-STATE
autodescrittivo, `genesis` fallibile, golden genesi, §5 replay a due processi, suite pulita debug e
release) sono **VERDI** con prova citata.

## NOTE / APERTE (non ROSSI)

1. **Sandbox**: `printf`/`echo`/`sed`/`python3` e le **pipe** sono negati; la riproduzione del golden
   (§4) è stata fatta scrivendo i byte in `reports/_l0_recheck_preimage_1042.bin` e usando `shasum`
   su file (metodo alternativo alla pipe usata dal gate precedente). I file di evidenza del replay
   (§5) sono in `reports/`. Non è un rosso del codice: limite di permessi.
2. **`7f76fea` (test-only)**: la commit contiene **solo** i tre file di test
   (`tests/l0_c1_genesis.rs`, `tests/l0_event_log.rs`, `tests/l0_state.rs`), nessun codice di
   produzione (`git show 7f76fea --stat`). Il rosso dichiarato include un **compile-fail** di
   `l0_c1_genesis.rs`; questo significa che a `7f76fea` l'aspettativa errata poi corretta in
   `45ad98d` **non fu mai eseguita** (il build di test non completava). L'aspettativa errata è emersa
   come rosso solo a `649deea`, cioè nel commit di implementazione, e corretta in `45ad98d`. Vedi §7.
3. **Non circolarità del golden (residuo già noto)**: la porzione `state` del preimage è prodotta dal
   crate; l'hash esterno la congela ma non ne prova la correttezza contro un riferimento indipendente
   (già segnalato dal gate precedente; fuori scope qui).

## Pulizia

- Worktree `…/opencode/ariadne-l0-recheck` rimosso con `git worktree remove --force` (dopo i test).
- File di evidenza lasciati in `reports/` (superficie ammessa): `_l0_recheck_preimage_1042.bin`
  (152 byte), `l0_recheck_replay_a.jsonl`, `l0_recheck_replay_b.jsonl` (16 righe / 3882 byte ciascuno,
  digest identico riportato in §5). Non è un rosso del codice.

---

## Appendice A — `cargo test --all-targets` (worktree pulito a HEAD 238d586, debug)

```
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-recheck)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.12s
     Running unittests src/lib.rs (target/debug/deps/ariadne-f4839cd99eb39cde)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::genesis_line_parses ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/bin/l0_replay.rs (target/debug/deps/l0_replay-4d8c495979d3a6f9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_c1_genesis.rs (target/debug/deps/l0_c1_genesis-b9498d7e71f5ee2d)

running 2 tests
test test_genesis_rejects_seed_beyond_i64_max_no_event_no_wrap ... ok
test test_genesis_accepts_seed_i64_max ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_cycle.rs (target/debug/deps/l0_cycle-4f5d14b3bfa31aa5)

running 11 tests
test budget_draw_1_per_tick_0_per_la_policy ... ok
test mossa_e_parse_stimolo_mod_3_su_piu_tick ... ok
test mai_cicli_parziali_ritmo_sense_action_state ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
test nessun_tick_salta_sequenza_0_1_1_1_2_2_2 ... ok
test seme_diverso_produce_log_diverso ... ok
test determinismo_forte_due_log_indipendenti_byte_identici ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
test un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi ... ok
test stimolo_oltre_i64_max_parse_completo_senza_mascheratura ... ok
test stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-a17f73a430b2072f)

running 18 tests
test test_event_types_constrained_to_sense_action_state ... ok
test test_event_without_seq_is_rejected ... ok
test test_compensating_state_on_genesis_only_log_copies_genesis_snapshot ... ok
test test_genesis_golden_hash_constant ... ok
test test_compensating_payload_carries_most_recent_state_snapshot_exact ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_state_string_format ... ok
test test_immutable_events_slice_borrows ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_hash_chain_end_to_end ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_prng.rs (target/debug/deps/l0_prng-90961bfec5e735c0)

running 7 tests
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test l_incremento_non_e_mai_pari ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_replay_process.rs (target/debug/deps/l0_replay_process-37116a8ce7977c02)

running 6 tests
test output_non_vuoto_finisce_con_un_solo_newline ... ok
test state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
test argomenti_diversi_producono_byte_diversi ... ok
test argomenti_invalidi_exit_non_zero_stdout_vuoto ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s

     Running tests/l0_state.rs (target/debug/deps/l0_state-ab94ffc3f9b3494f)

running 9 tests
test test_fold_genesis_initial_state ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_head_is_hash_of_last_event_not_genesis ... ok
test test_fold_is_pure_byte_identical ... ok
test test_compensation_does_not_advance_prng ... ok
test test_counters_count_hand_built_sequence ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_compensating_state_carries_last_snapshot_and_fold_reads_it ... ok
test test_real_log_every_state_carries_snapshot_and_fold_agrees ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Appendice B — `cargo test --release --all-targets` (worktree pulito a HEAD 238d586, release)

```
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-recheck)
    Finished `release` profile [optimized] target(s) in 2.51s
     Running unittests src/lib.rs (target/release/deps/ariadne-e38db9ac10129f4e)

running 4 tests
test event::tests::compute_hash_matches_documented_preimage ... ok
test event::tests::event_type_parse_is_total_and_strict ... ok
test log::tests::genesis_line_parses ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/l0_replay.rs (target/release/deps/l0_replay-8285744b5bb2c0d4)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_c1_genesis.rs (target/release/deps/l0_c1_genesis-4b9261e86831ba9a)

running 2 tests
test test_genesis_rejects_seed_beyond_i64_max_no_event_no_wrap ... ok
test test_genesis_accepts_seed_i64_max ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_cycle.rs (target/release/deps/l0_cycle-e0de9be28f6b7f0a)

running 11 tests
test budget_draw_1_per_tick_0_per_la_policy ... ok
test mossa_e_parse_stimolo_mod_3_su_piu_tick ... ok
test mai_cicli_parziali_ritmo_sense_action_state ... ok
test seme_diverso_produce_log_diverso ... ok
test determinismo_forte_due_log_indipendenti_byte_identici ... ok
test nessun_tick_salta_sequenza_0_1_1_1_2_2_2 ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
test stimolo_oltre_i64_max_parse_completo_senza_mascheratura ... ok
test un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
test stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/release/deps/l0_event_log-3c8b31b9bd7edeb8)

running 18 tests
test test_compensating_event_payload_and_history_intact ... ok
test test_compensating_payload_carries_most_recent_state_snapshot_exact ... ok
test test_compensating_state_on_genesis_only_log_copies_genesis_snapshot ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_event_without_seq_is_rejected ... ok
test test_genesis_golden_hash_constant ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_state_string_format ... ok
test test_hash_chain_end_to_end ... ok
test test_immutable_events_slice_borrows ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok
test test_unknown_top_level_member_rejected ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/l0_prng.rs (target/release/deps/l0_prng-1854648b393fc0c0)

running 7 tests
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test l_incremento_non_e_mai_pari ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_replay_process.rs (target/release/deps/l0_replay_process-2552478134406c3c)

running 6 tests
test output_non_vuoto_finisce_con_un_solo_newline ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks ... ok
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
test argomenti_diversi_producono_byte_diversi ... ok
test argomenti_invalidi_exit_non_zero_stdout_vuoto ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s

     Running tests/l0_state.rs (target/release/deps/l0_state-7b2276bab614da0e)

running 9 tests
test test_compensating_state_carries_last_snapshot_and_fold_reads_it ... ok
test test_compensation_does_not_advance_prng ... ok
test test_counters_count_hand_built_sequence ... ok
test test_fold_genesis_initial_state ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_fold_is_pure_byte_identical ... ok
test test_real_log_every_state_carries_snapshot_and_fold_agrees ... ok
test test_head_is_hash_of_last_event_not_genesis ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
