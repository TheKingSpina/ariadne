# L0 — Report di conformità (prima metà: §1–§5 e divieti)

- HEAD verificato: `5b495d7` (`git log --oneline`: `5b495d7 L0 B5: l0_replay — binario diagnostico ...`)
- SPEC: `spec/L0_SPEC_v0.2.md` (v0.2)
- Data: 2026-10-22
- Ambito: conformità §1–§5 + divieti. **L'archeologia dei commit / test-first è demandata a un dispatch separato e NON è giudicata qui.**
- Nessuna correzione di codice o test. Nessun file creato fuori da `reports/`.

## Metodo e ambiente

1. Checkout pulito: creato worktree dedicato a HEAD
   `git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate HEAD`
   → `Preparing worktree (detached HEAD 5b495d7)` / `HEAD is now at 5b495d7 ...`.
2. Prima dei test, nel worktree: `git status --porcelain` → **output vuoto** (checkout pulito).
3. Suite completa eseguita NEL worktree pulito: `cargo test --all-targets` → **51 test, 0 failed** (output integrale in Appendice A).
4. Replay §5.1 eseguito nel worktree pulito, due processi separati (output in §5.1).
5. Il worktree è stato rimosso a fine verifica (vedi "Pulizia").

Nota di sandbox: i comandi `git rev-parse` e le forme concatenate `cd … && git …` sono stati rifiutati
dalla sandbox ("Permission denied: shell"); `git status`, `git log`, `cargo`, `rg`, `shasum`, `wc`
hanno funzionato. Nessun esito di questo report dipende da un comando negato.

---

## §5 — Checklist di uscita

| # | Voce | Esito | Prova |
|---|------|-------|-------|
| 5.1 | Replay bit-perfetto su due esecuzioni indipendenti | **VERDE** | due `cargo run` separati, digest identici `967df4a0…8784`; `tests/l0_replay_process.rs:49-63` verde |
| 5.2 | Catena di hash verificabile end-to-end | **VERDE** | `src/log.rs:112-137`; `src/event.rs:95-122`; `tests/l0_event_log.rs:163-180` verde |
| 5.3 | Zero wall-clock, I/O esterna o float nel codice L0 | **NOTA** (substrato VERDE) | `rg` sul substrato: nessun match; eccezione harness documentata in `src/bin/l0_replay.rs:15-26` |
| 5.4 | Correzione compensativa testata con successo | **VERDE** | `src/log.rs:87-102`; `tests/l0_event_log.rs:342-368` verde; `tests/l0_state.rs:182-195` verde |
| 5.5 | Zero mutazioni retroattive dimostrate | **VERDE** | `tests/l0_event_log.rs:285-312`; `src/log.rs:105-107` |

### §5.1 — Replay bit-perfetto su due esecuzioni indipendenti

Worktree pulito a HEAD `5b495d7`. Due esecuzioni separate del binario (due processi distinti),
stessi argomenti, stdout catturato:

```
cargo run --quiet --bin l0_replay -- --seed 1042 --ticks 5 > /Users/spina/Ariadne/reports/l0_replay_a.jsonl
cargo run --quiet --bin l0_replay -- --seed 1042 --ticks 5 > /Users/spina/Ariadne/reports/l0_replay_b.jsonl
```

Digest:

```
967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784  /Users/spina/Ariadne/reports/l0_replay_a.jsonl
967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784  /Users/spina/Ariadne/reports/l0_replay_b.jsonl
```

I due digest **coincidono**. Dimensione identica:

```
      16    3882 reports/l0_replay_a.jsonl
      16    3882 reports/l0_replay_b.jsonl
      32    7764 total
```

(16 righe = genesi + 3 eventi × 5 tick, coerente con `tests/l0_replay_process.rs:101-122`.)
Controllo incrociato: la stessa coppia eseguita anche nel checkout principale allo stesso HEAD ha
prodotto lo stesso digest `967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784`
(4 esecuzioni totali, un solo digest).

Test dedicato, verde (output testuale):
```
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
```
`tests/l0_replay_process.rs:49-63` lancia `env!("CARGO_BIN_EXE_l0_replay")` due volte con
`std::process::Command` (due processi) e confronta `a.stdout == b.stdout`.
`tests/l0_replay_process.rs:81-95` verifica anche il round-trip
`Log::from_jsonl(stdout).verify()` + `to_jsonl() == stdout`.
Il binario stampa esattamente `Log::to_jsonl()` (`src/bin/l0_replay.rs:51-54`).

### §5.2 — Catena di hash verificabile end-to-end

- `src/log.rs:112-137` — `Log::verify`: `seq == posizione` (monotono da 0, nessun gap),
  `prev_hash` agganciato all'hash precedente (genesi: 64 zeri), `hash` ricalcolato dal preimage.
- `src/event.rs:95-122` — SHA-256, esadecimale minuscolo; preimage
  `seq ‖ 0x1F ‖ tick ‖ 0x1F ‖ type ‖ 0x1F ‖ payload ‖ 0x1F ‖ prev_hash`.
- Genesi con 64 zeri: `src/event.rs:24`, `src/log.rs:66`.
- Golden esterno della genesi seed 1042: `tests/l0_event_log.rs:27` (`4649d1f4…26b7`),
  asserito in `tests/l0_event_log.rs:100-109` e in `src/log.rs:475-483`.
- Prova indipendente evento-per-evento: `tests/l0_event_log.rs:163-180` ricalcola lo SHA-256
  con `sha2` direttamente nel test.

Output testuale:
```
test test_hash_chain_end_to_end ... ok
test test_genesis_golden_hash_constant ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
```

NOTA (non ROSSO): l'encoding della concatenazione usa il separatore `0x1F`, scelta documentata in
`src/event.rs:11-16` e `src/lib.rs:5-6` (la SPEC §2 non fissa l'encoding). Il golden fissato nei
test rende il byte-format verificabile.

### §5.3 — Zero wall-clock, I/O esterna o float nel codice L0

Comando eseguito sui soli file del substrato (`src/lib.rs`, `event.rs`, `log.rs`, `payload.rs`,
`prng.rs`, `state.rs`, `cycle.rs`):

```
$ rg -n 'std::time|SystemTime|Instant|std::fs|std::net|f32|f64|std::io' src/lib.rs src/event.rs src/log.rs src/payload.rs src/prng.rs src/state.rs src/cycle.rs
(no output)
Exited with code 1
```

Nessun match (exit 1 = nessuna riga). Comando complementare su tutto `src/`:

```
$ rg -n 'std::process|std::env|print!|println!|eprintln!' src/
src/bin/l0_replay.rs:38:use std::process::ExitCode;
src/bin/l0_replay.rs:51:                    // Solo i byte del log: `print!` NON aggiunge il newline
src/bin/l0_replay.rs:53:                    print!("{}", log.to_jsonl());
src/bin/l0_replay.rs:57:                    eprintln!("l0_replay: esecuzione del ciclo non riuscita: {e}");
src/bin/l0_replay.rs:63:            eprintln!("{messaggio}");
src/bin/l0_replay.rs:72:    let args: Vec<String> = std::env::args().skip(1).collect();
```

Tutte le occorrenze di I/O/processo/env sono **solo** in `src/bin/l0_replay.rs`, harness diagnostico.
L'eccezione è documentata nel mod-doc:

- `src/bin/l0_replay.rs:15-26` — "# Eccezione documentata al divieto di I/O esterna … DIVIETO CHE VALE
  PER `lib`, NON PER QUESTO HARNESS … input dichiarati `--seed` e `--ticks`, nessun tempo, nessun file,
  nessuna rete".
- `src/lib.rs:17-18` — "Determinismo: zero wall-clock, zero I/O esterna, zero reti, zero float."

Il binario non è raggiungibile dalla lib (la lib non lo importa; la dipendenza è bin → lib):
```
$ rg -n 'l0_replay|src/bin' src/lib.rs src/event.rs src/log.rs src/payload.rs src/prng.rs src/state.rs src/cycle.rs
(no output)
Exited with code 1
```

Esito: **NOTA** ai sensi del dispatch ("se c'è [la documentazione], NOTA"). Il substrato è pulito;
l'unica I/O è nell'harness, dichiarata e non raggiungibile dalla lib.

### §5.4 — Correzione compensativa testata con successo

- `src/log.rs:87-102` — `Log::compensate(compensates, reason, tick)`: rifiuta un `seq` inesistente
  (`src/log.rs:88-93`), appende un evento `STATE` con payload canonico
  `{"compensates":<seq>,"reason":"<reason>"}`; l'evento compensato resta nel log.
- `tests/l0_event_log.rs:342-368` — verifica `seq`, `type == STATE`, payload esatto, prefisso storico
  intatto (`after.starts_with(prefix_before)`), evento errato ancora presente, `log.verify()` verde,
  compensazione verso seq inesistente rifiutata.
- `tests/l0_state.rs:182-195` — l'evento compensativo non porta `state` e la fold mantiene l'ultimo
  snapshot PRNG.

Output testuale:
```
test test_compensating_event_payload_and_history_intact ... ok
test test_compensating_state_has_no_snapshot_and_keeps_last_prng ... ok
```

### §5.5 — Zero mutazioni retroattive dimostrate

- `src/log.rs:105-107` — `events(&self) -> &[Event]`: sola lettura, nessun `&mut`.
- `src/log.rs:54-173` — superficie API: `genesis`, `append`, `compensate`, `events`, `verify`,
  `to_jsonl`, `from_jsonl`. Nessun update/delete/rewrite/reorder.
- `tests/l0_event_log.rs:285-312` — tre mutazioni retroattive (payload vecchio, hash memorizzato,
  `prev_hash` memorizzato) rendono `verify()` errore; l'originale resta valido.
- `tests/l0_event_log.rs:370-382` — il prefisso serializzato resta intatto dopo un append
  compensativo.
- `tests/l0_state.rs:201-241` — la fold rifiuta una catena non agganciata anche senza `verify()`
  preventivo.

Output testuale:
```
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok
test test_immutable_events_slice_borrows ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
```

---

## §1 — Substrato

**Esito: VERDE.**

- Unico substrato log append-only aciclico: `src/lib.rs:20-25` espone solo i moduli
  `cycle`, `event`, `log`, `payload`, `prng`, `state`; `src/lib.rs:27-30` i re-export.
  Dipendenze fra moduli (nessun ciclo): `event → payload` (`src/event.rs:21`),
  `log → event/payload/prng` (`src/log.rs:22-24`), `state → event/log/payload`
  (`src/state.rs:45-47`), `cycle → event/log/payload/prng` (`src/cycle.rs:62-65`);
  `payload` e `prng` non importano altri moduli del crate.
- Un solo ciclo senso→azione→stato: `src/cycle.rs:87-138` (`run_cycle`) e `src/cycle.rs:141-146`
  (`run_cycles` che itera `run_cycle`). Nessun altro ciclo di eventi nel substrato.
- Divieti wall-clock/rete/float/I/O: vedi §5.3 e sezione Divieti (nessun match sul substrato).
- Append-only: vedi §3/§5.5.

## §2 — Forma evento

**Esito: VERDE** (con NOTA sull'encoding del preimage).

| Requisito SPEC §2 | Prova |
|---|---|
| forma `{seq, tick, type, payload, prev_hash, hash}` | `src/event.rs:67-74`; serializzazione ordine SPEC in `src/log.rs:175-189` |
| `seq` uint64 monotono da 0, gap vietati | `src/log.rs:72-81` (append = ultimo+1); `src/log.rs:112-120` (verify: `seq == posizione`); test `tests/l0_event_log.rs:266-279` |
| `tick` contatore logico, mai timestamp | `src/event.rs:5`; `src/cycle.rs:91-98` (ultimo tick del log + 1); nessun import tempo (§5.3) |
| `type ∈ {SENSE, ACTION, STATE}` | `src/event.rs:31-55` (enum + parse stretto); parser rifiuta altro `src/log.rs:266-271`; test `tests/l0_event_log.rs:182-193` |
| payload JSON canonico: chiavi ordinate, UTF-8, solo interi/stringhe | `src/payload.rs:42-44` (`BTreeMap`), `src/payload.rs:84-104` (serializzazione compatta); dominio `src/payload.rs:15-20`; parser rifiuta float/bool/null/array/object/duplicati `src/log.rs:322-334`; test `tests/l0_event_log.rs:207-240` |
| `prev_hash` = hash precedente, 64 zeri per genesi | `src/event.rs:24`; `src/log.rs:66`; verifica aggancio `src/log.rs:121-126` |
| `hash` = SHA-256 esadecimale minuscolo | `src/event.rs:95-122` (`to_hex` minuscolo, `sha2`); validazione 64 hex lowercase nel parser `src/log.rs:386-394` |
| `state` genesi stringa decimale (Schema PRNG) | `src/log.rs:59-65`; `src/prng.rs:120-122`; test `tests/l0_event_log.rs:115-157` |

Output testuale rilevante:
```
test test_payload_canonical_sorted_keys_minimal ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_genesis_state_string_format ... ok
```

NOTA (non ROSSO): il parser `from_jsonl` è più permissivo del serializzatore su escape JSON
accettati (`\/` e `\uXXXX` con esadecimale maiuscolo: `src/log.rs:405-434`), mentre il
serializzatore canonico non emette `\/` (`src/payload.rs:112-130`); un input con `\/` verrebbe
accettato e riserializzato in forma diversa. La SPEC non prescrive il comportamento del parser,
ma il mod-doc `src/log.rs:149-151` dichiara "parse rigoroso … solo linee canoniche".
Nessun test copre questo caso.

NOTA (non ROSSO): il test unitario `src/event.rs:128-149` usa un payload con `"state":1042`
(intero) e golden `e73ef6c4…`, cioè una forma diversa dalla genesi B2 (`state` stringa,
golden `4649d1f4…`, `tests/l0_event_log.rs:27`). È un test di `compute_hash` su una stringa
fornita a mano, non una violazione della produzione, ma la compresenza di due forme nel repo
può confondere.

## §3 — Assenza di mutazione

**Esito: VERDE.**

Superficie pubblica (output `rg`):
```
$ rg -n 'pub fn|pub struct|pub enum|pub use' src/lib.rs src/event.rs src/log.rs src/payload.rs src/state.rs src/cycle.rs
src/cycle.rs:74:pub fn mossa_from_stimolo(stimolo_dec: &str) -> Option<i64> {
src/cycle.rs:87:pub fn run_cycle(log: &mut Log, rng: &mut Pcg64) -> Result<(), Error> {
src/cycle.rs:141:pub fn run_cycles(log: &mut Log, rng: &mut Pcg64, n: u64) -> Result<(), Error> {
src/state.rs:52:pub enum StateError {
src/state.rs:81:pub struct State {
src/state.rs:98:    pub fn fold(log: &Log) -> Result<State, StateError> {
src/state.rs:178:    pub fn to_payload(&self) -> Payload {
src/state.rs:190:    pub fn canonical(&self) -> String {
src/payload.rs:17:pub enum Value {
src/payload.rs:42:pub struct Payload {
src/payload.rs:47:    pub fn empty() -> Self {
src/payload.rs:55:    pub fn from_pairs<I, K>(pairs: I) -> Self
src/payload.rs:67:    pub fn insert(&mut self, key: &str, value: Value) {
src/payload.rs:71:    pub fn get(&self, key: &str) -> Option<&Value> {
src/payload.rs:75:    pub fn is_empty(&self) -> bool {
src/payload.rs:79:    pub fn len(&self) -> usize {
src/payload.rs:84:    pub fn canonical(&self) -> String {
src/payload.rs:106:    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
src/lib.rs:27:pub use event::{compute_hash, Event, EventType, GENESIS_PREV_HASH};
src/lib.rs:28:pub use log::{Error, Log};
src/lib.rs:29:pub use payload::{Payload, Value};
src/lib.rs:30:pub use state::{State, StateError};
src/log.rs:27:pub enum Error {
src/log.rs:50:pub struct Log {
src/log.rs:59:    pub fn genesis(seed: u64) -> Log {
src/log.rs:72:    pub fn append(&mut self, tick: u64, r#type: EventType, payload: Payload) -> Result<&Event, Error> {
src/log.rs:87:    pub fn compensate(&mut self, compensates: u64, reason: &str, tick: u64) -> Result<Event, Error> {
src/log.rs:105:    pub fn events(&self) -> &[Event] {
src/log.rs:112:    pub fn verify(&self) -> Result<(), Error> {
src/log.rs:140:    pub fn to_jsonl(&self) -> String {
src/log.rs:152:    pub fn from_jsonl(input: &str) -> Result<Log, Error> {
src/event.rs:31:pub enum EventType {
src/event.rs:39:    pub fn as_str(self) -> &'static str {
src/event.rs:48:    pub fn parse(s: &str) -> Option<Self> {
src/event.rs:67:pub struct Event {
src/event.rs:95:pub fn compute_hash(
```
(la lista è completa; l'unico punto di costruzione degli eventi, `Event::seal`
`src/event.rs:81`, non compare perché `pub(crate)`.)

- Nessuna API `update`/`delete`/`remove`/`rewrite`/`reorder`/`truncate`/`swap`:
  ```
  $ rg -n 'fn (update|delete|remove|rewrite|reorder|truncate|swap|sort)' src/
  (no output)
  Exited with code 1
  ```
- `Log::events()` restituisce `&[Event]` (`src/log.rs:105-107`); `Log.events` è campo privato
  (`src/log.rs:50-52`); `Event::seal` è `pub(crate)` (`src/event.rs:81`), quindi non si può
  appendere un evento costruito dall'esterno.
- Correzione compensativa: `src/log.rs:87-102`, payload `{compensates, reason}`, storia intatta;
  test verde (§5.4).

NOTA (non ROSSO): `Event` ha campi pubblici (`src/event.rs:67-74`) e `Payload::insert`
(`src/payload.rs:67-69`) è pubblico; un chiamante può mutare copie locali, ma non il `Vec` dentro
`Log` (nessun accessore mutabile, nessun metodo di inserimento diretto). La storia del log resta
non mutabile dall'API.

## §4 — Genesi, replay, stato

**Esito: VERDE** (con NOTA su `seed` > `i64::MAX`).

- Engine PCG64 (PCG XSL RR 128/64): `src/prng.rs:1-19`, costanti e passi `src/prng.rs:60-122`;
  seeding documentato `src/prng.rs:21-47`, `src/prng.rs:83-100`.
- Genesi con seed: `src/log.rs:59-68` (seq 0, tick 0, STATE, prev_hash 64 zeri, payload
  `{prng:"PCG64", seed:<int>, state:"<decimale>"}`), verificata da
  `tests/l0_event_log.rs:81-98` e `tests/l0_event_log.rs:100-109`.
- PRNG nello stato, snapshot a ogni STATE: `src/cycle.rs:121-133`
  (`"state" => rng.state_decimal()`); la fold legge lo snapshot, non riesegue i draw
  (`src/state.rs:8-12`, `src/state.rs:153-160`); test `tests/l0_state.rs:131-158` verde.
- Stato = fold in ordine di `seq`: `src/state.rs:98-173` (guardia `seq == posizione` a
  `src/state.rs:107-121`, poi fold lineare `src/state.rs:141-164`).
- Stesso log → stesso stato byte per byte: `src/state.rs:188-192` (`canonical()` via `Payload`),
  test `tests/l0_state.rs:71-102` verde (due fold e riplay dai byte → stessa stringa).
- Ordine draw e budget espliciti: `src/cycle.rs:19-25` (1 draw/tick al SENSE, 0 policy, 0 genesi);
  implementazione `src/cycle.rs:100-106` (1 `next_u64`), `src/cycle.rs:108-119` (policy
  `parse % 3`, 0 draw), `src/cycle.rs:121-133` (snapshot, 0 draw); test
  `tests/l0_cycle.rs:190-241` e `tests/l0_cycle.rs:248-296` verdi.

Output testuale:
```
test test_fold_genesis_initial_state ... ok
test test_fold_is_pure_byte_identical ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test budget_draw_1_per_tick_0_per_la_policy ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
```

NOTA (non ROSSO): in `src/log.rs:63` il seed è castato `seed as i64` per il payload (che ammette
solo interi). Per seed ≥ 2^63 il campo `seed` della genesi diventa negativo e non rappresenta più
il seed u64. Riproduzione (verificata, worktree pulito):
```
$ cargo run --quiet --bin l0_replay -- --seed 9223372036854775808 --ticks 0
{"seq":0,"tick":0,"type":"STATE","payload":{"prng":"PCG64","seed":-9223372036854775808,"state":"247731279812972569658953206167489578021"},"prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","hash":"452c4e17b9933562eabcc871a1405ca1cca6120eaf0c3c6fa34c87abcb9b536c"}
```
Il corpus dichiarato dalla SPEC (seed 1000–1099) è interamente dentro `i64`, quindi la voce resta
VERDE per il dominio specificato; il caso è segnalato al producer.

NOTA (non ROSSO): né `Log::verify` (`src/log.rs:112-137`) né `State::fold` (`src/state.rs:98-121`)
impongono che l'evento a `seq 0` sia `type == STATE`; leggono solo `state` dal payload. Una riga
esterna con `seq 0` e type diverso ma payload con `state` stringa passerebbe. `Log::genesis`
produce sempre STATE (`src/log.rs:66`), quindi il percorso di produzione è conforme.

---

## Divieti — grep sul substrato

Comando (esclusi `src/bin/` per costruzione, elencando i file del substrato):

```
$ rg -n 'std::time|SystemTime|Instant|std::fs|std::net|f32|f64|std::io' src/lib.rs src/event.rs src/log.rs src/payload.rs src/prng.rs src/state.rs src/cycle.rs
(no output)
Exited with code 1
```

Output aggiuntivo per localizzare l'eccezione harness:

```
$ rg -n 'std::process|std::env|print!|println!|eprintln!' src/
src/bin/l0_replay.rs:38:use std::process::ExitCode;
src/bin/l0_replay.rs:51:                    // Solo i byte del log: `print!` NON aggiunge il newline
src/bin/l0_replay.rs:53:                    print!("{}", log.to_jsonl());
src/bin/l0_replay.rs:57:                    eprintln!("l0_replay: esecuzione del ciclo non riuscita: {e}");
src/bin/l0_replay.rs:63:            eprintln!("{messaggio}");
src/bin/l0_replay.rs:72:    let args: Vec<String> = std::env::args().skip(1).collect();
```

Nessuna occorrenza nel substrato; tutte nell'harness, la cui eccezione è documentata in
`src/bin/l0_replay.rs:15-26` e non è raggiungibile dalla lib (la lib non importa il binario).

---

## Test dichiarati dal producer (esistenza — non archeologia dei commit)

| Test richiesto | Esistenza | Esito |
|---|---|---|
| replay del prefisso storico byte-per-byte | `tests/l0_event_log.rs:314-336` (`test_replay_prefix_byte_per_byte`) | verde |
| immutabilità: fallisce se un evento vecchio cambia | `tests/l0_event_log.rs:285-312` (`test_log_is_append_only_no_mutation_truncation_or_reorder`) | verde |
| vieta un evento senza `seq` di origine | `tests/l0_event_log.rs:256-264` (`test_event_without_seq_is_rejected`) | verde |

(La verifica che questi test siano stati *primi e rossi* è del gate di archeologia, non di questo.)

---

## Elenco ROSSI

**Nessun ROSSO.**

Nessuna voce della checklist §5 è ROSSA; §1–§4 conformi; nessun accesso a input non dichiarati
rilevato (input del substrato: solo `Log`/`Pcg64`/payload; il binario accetta solo `--seed` e
`--ticks`, `src/bin/l0_replay.rs:71-105`).

## NOTE / APERTE (non ROSSI, per il producer)

1. §5.3 — I/O stdout/stderr/env/process esiste solo in `src/bin/l0_replay.rs`; eccezione
   documentata (`src/bin/l0_replay.rs:15-26`). Esito NOTA come da dispatch.
2. `src/log.rs:63` — `seed as i64`: seed ≥ 2^63 produce `seed` negativo nell'evento genesi
   (riproduzione sopra). Fuori dal corpus 1000–1099.
3. `src/log.rs:405-434` vs `src/payload.rs:112-130` — il parser accetta escape non canonici
   (`\/`, `\uXXXX` maiuscolo) che il serializzatore non emette; nessun test copre il caso.
4. `src/log.rs:112-137` / `src/state.rs:98-121` — non impongono `type == STATE` a `seq 0`.
5. `src/event.rs:128-149` — unit test con payload `"state":1042` intero, forma diversa dalla
   genesi B2 (`state` stringa). Test-only.

## Pulizia

- Worktree `…/opencode/ariadne-l0-gate` rimosso con `git worktree remove --force` (dopo i test);
  `git worktree list` ora mostra solo `/Users/spina/Ariadne  5b495d7 [main]`.
- `reports/l0_replay_a.jsonl` e `reports/l0_replay_b.jsonl`: la rimozione è stata **negata dalla
  sandbox** (`rm` e `git clean` → "Permission denied: shell"). Restano in `reports/` come evidenza
  (16 righe / 3882 byte ciascuno, digest identico riportato in §5.1). Non è un rosso del codice:
  è un limite di permessi.

---

## Appendice A — `cargo test --all-targets` (worktree pulito a HEAD 5b495d7)

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
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/ariadne-l0-gate)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.70s
     Running unittests src/lib.rs (target/debug/deps/ariadne-f4839cd99eb39cde)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::genesis_line_parses ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/l0_replay.rs (target/debug/deps/l0_replay-4d8c495979d3a6f9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_cycle.rs (target/debug/deps/l0_cycle-4f5d14b3bfa31aa5)

running 11 tests
test seme_diverso_produce_log_diverso ... ok
test nessun_tick_salta_sequenza_0_1_1_1_2_2_2 ... ok
test mai_cicli_parziali_ritmo_sense_action_state ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
test mossa_e_parse_stimolo_mod_3_su_piu_tick ... ok
test budget_draw_1_per_tick_0_per_la_policy ... ok
test determinismo_forte_due_log_indipendenti_byte_identici ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
test un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi ... ok
test stimolo_oltre_i64_max_parse_completo_senza_mascheratura ... ok
test stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-a17f73a430b2072f)

running 16 tests
test test_genesis_state_is_string_not_integer ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_state_string_format ... ok
test test_event_without_seq_is_rejected ... ok
test test_genesis_golden_hash_constant ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_immutable_events_slice_borrows ... ok
test test_hash_chain_end_to_end ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_prng.rs (target/debug/deps/l0_prng-90961bfec5e735c0)

running 7 tests
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test l_incremento_non_e_mai_pari ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_replay_process.rs (target/debug/deps/l0_replay_process-37116a8ce7977c02)

running 6 tests
test state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks ... ok
test output_non_vuoto_finisce_con_un_solo_newline ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
test argomenti_diversi_producono_byte_diversi ... ok
test argomenti_invalidi_exit_non_zero_stdout_vuoto ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s

     Running tests/l0_state.rs (target/debug/deps/l0_state-ab94ffc3f9b3494f)

running 7 tests
test test_head_is_hash_of_last_event_not_genesis ... ok
test test_fold_genesis_initial_state ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_compensating_state_has_no_snapshot_and_keeps_last_prng ... ok
test test_fold_is_pure_byte_identical ... ok
test test_counters_count_hand_built_sequence ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
