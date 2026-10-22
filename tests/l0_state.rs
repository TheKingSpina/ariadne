//! L0 B3 — fold dello State (SPEC §4 "Determinismo e replay" + riga 43 dello
//! Schema PRNG).
//!
//! Questo file è il commit TEST-ONLY: se resta solo qui la suite è ROSSA
//! perché il modulo `state` non esiste ancora.
//!
//! Contratto testato:
//! - Stato = fold del log applicato in ordine di `seq` (SPEC §4);
//! - lo stato del PRNG NON è ricalcolato dai draw: si LEGGE dallo snapshot
//!   nell'evento STATE (riga 43: "il replay verifica per confronto invece di
//!   fidarsi della riesecuzione") — se la fold rieseguisse i draw, il replay
//!   sarebbe un confronto fra due esecuzioni identiche per costruzione;
//! - la fold è PURA: stesso log → stesso stato, byte per byte (golden rule §4);
//! - `tick` = ultimo tick visto; `sense`/`action` = numero di eventi SENSE/ACTION
//!   applicati; `head` = hash dell'ultimo evento applicato;
//! - la fold ASSUME un log verificato (`verify() == Ok`): un log con catena
//!   rotta che arriva alla fold senza un verify preventivo fa fallire la fold.
//!
//! Regola di golden rule lacinata nel test principale: lo stato canonico
//! serializzato riusa `Payload` (stringhe come `Value::Str`, contatori come
//! `Value::Int`) e ha forma stabile.

use ariadne::prng::Pcg64;
use ariadne::state::{State, StateError};
use ariadne::{EventType, Log, Payload, Value};

/// Hash di riferimento della genesi seed 1042 (stessi golden del test del log:
/// SHA-256 del preimage §2 con separatore 0x1F, calcolato indipendentemente).
const GENESIS_1042_HASH: &str = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";

/// Stato interno PCG64 a 128 bit per seed 1042 (`Pcg64::new(1042).state_decimal()`).
const GENESIS_1042_STATE_DEC: &str = "43149974864253806286336044685702662359";

/// Snapshot di `state` per i tick costruiti a mano: stringa decimale qualsiasi,
/// letta da fold VERBATIM (la fold non reinterpretà mai lo snapshot).
const SNAP: &str = "123456789012345678901234567890123456789";

// ---------------------------------------------------------------------------
// Genesi: lo stato iniziale
// ---------------------------------------------------------------------------

#[test]
fn test_fold_genesis_initial_state() {
    let log = Log::genesis(1042);
    log.verify().expect("la genesi da sola è un log verificato");
    let s = State::fold(&log).expect("la fold di un log verificato non fallisce");
    assert_eq!(s.tick, 0, "tick 0 alla genesi");
    assert_eq!(s.sense, 0, "contatore SENSE a zero alla genesi");
    assert_eq!(s.action, 0, "contatore ACTION a zero alla genesi");
    assert_eq!(s.prng, GENESIS_1042_STATE_DEC, "prng = snapshot della genesi, non ricalcolato");
    assert_eq!(s.head, GENESIS_1042_HASH, "head = hash della genesi");
    // stesso contratto su un secondo, generico assieme di seed: coerenza campo a campo col log
    for seed in [1u64, 7777, 1099] {
        let log = Log::genesis(seed);
        let g = &log.events()[0];
        let expected_prng = match g.payload.get("state") {
            Some(Value::Str(v)) => v.clone(),
            other => panic!("state genesi non è una stringa: {other:?}"),
        };
        let s = State::fold(&log).unwrap();
        assert_eq!(s.prng, expected_prng, "seed {seed}: prng dal payload genesi");
        assert_eq!(s.head, g.hash, "seed {seed}: head = hash genesi");
        assert_eq!(s.tick, 0);
    }
}

// ---------------------------------------------------------------------------
// Purezza e forma canonica stabile
// ---------------------------------------------------------------------------

#[test]
fn test_fold_is_pure_byte_identical() {
    let build = || {
        let mut log = Log::genesis(1000);
        log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(42))]))
            .unwrap();
        log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("V".to_string()))]))
            .unwrap();
        log.append(1, EventType::State, Payload::from_pairs(vec![("state", Value::Str(SNAP.to_string()))]))
            .unwrap();
        log
    };
    let log = build();
    let a = State::fold(&log).expect("prima fold").canonical();
    let b = State::fold(&log).expect("seconda fold").canonical();
    assert_eq!(a.as_bytes(), b.as_bytes(), "due fold dello stesso log → byte identici");
    // la fold non consuma né muta il log: può essere riffatta da un clone dei byte
    let jsonl = log.to_jsonl();
    let c = State::fold(&Log::from_jsonl(&jsonl).unwrap()).unwrap().canonical();
    assert_eq!(c.as_bytes(), a.as_bytes(), "riplay dai byte → stesso stato");

    // forma canonica della genesi: chiavi esatte e in quest'ordine esatto,
    // stessa stringa a ogni run (ordine alfabetico: action, head, prng, sense, tick)
    let genesis = State::fold(&Log::genesis(1042)).unwrap();
    assert_eq!(
        genesis.canonical(),
        concat!(
            r#"{"action":0,"head":"4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7","#,
            r#""prng":"43149974864253806286336044685702662359","sense":0,"tick":0}"#,
        )
    );
}

// ---------------------------------------------------------------------------
// Contatori su sequenza costruita a mano
// ---------------------------------------------------------------------------

#[test]
fn test_counters_count_hand_built_sequence() {
    let mut log = Log::genesis(7);
    log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(1))])).unwrap();
    log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("a".to_string()))])).unwrap();
    log.append(1, EventType::State, Payload::from_pairs(vec![("state", Value::Str(SNAP.to_string()))])).unwrap();
    log.append(2, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(2))])).unwrap();
    log.append(2, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("b".to_string()))])).unwrap();
    log.append(2, EventType::State, Payload::from_pairs(vec![("state", Value::Str(SNAP.to_string()))])).unwrap();
    log.verify().expect("sequenza costruita a mano è valida");
    let s = State::fold(&log).unwrap();
    assert_eq!(s.sense, 2, "SENSE applicati: 2");
    assert_eq!(s.action, 2, "ACTION applicati: 2");
    assert_eq!(s.tick, 2, "tick = ultimo tick visto nel log");
    assert_eq!(s.prng, SNAP, "prng = snapshot dell'ultimo STATE");
    let last = log.events().last().unwrap().hash.clone();
    assert_eq!(s.head, last, "head = hash dell'ultimo evento");
}

// ---------------------------------------------------------------------------
// Il punto della SPEC (riga 43): prng letto dallo snapshot, MAI ricalcolato
// ---------------------------------------------------------------------------

#[test]
fn test_prng_read_from_last_state_snapshot_not_recomputed() {
    let seed = 42u64;
    // un'eventuale riesecuzione dal seed per questa sequenza farebbe UN draw
    // (SENSE): questo sarebbe lo stato PCG64 "onesto" dopo il tick
    let mut replay_engine = Pcg64::new(seed);
    let draw = replay_engine.next_u64();
    let recompute_one_draw = replay_engine.state_decimal();
    // lo snapshot scritto nell'evento STATE è DIVERSO da qualunque ricalcolo:
    // due draw (un caso apposito per escludere ogni coincidenza)
    let mut snap_engine = Pcg64::new(seed);
    let _ = snap_engine.next_u64();
    let _ = snap_engine.next_u64();
    let snapshot = snap_engine.state_decimal();
    assert_ne!(snapshot, recompute_one_draw, "controllo: il ricalcolo darebbe un valore diverso");
    assert_ne!(snapshot, Pcg64::new(seed).state_decimal(), "e diverso anche dalla genesi");

    let mut log = Log::genesis(seed);
    log.append(1, EventType::Sense, Payload::from_pairs(vec![("draw", Value::Int(draw as i64))]))
        .unwrap();
    log.append(1, EventType::State, Payload::from_pairs(vec![("state", Value::Str(snapshot.clone()))]))
        .unwrap();
    log.verify().expect("log verificato");
    let s = State::fold(&log).unwrap();
    assert_eq!(s.prng, snapshot, "fold: prng LEGGUTO dallo snapshot dell'ultimo STATE");
    assert_ne!(s.prng, recompute_one_draw, "la fold NON ha rieseguito i draw: avrebbe dato l'altro valore");
    assert_ne!(s.prng, Pcg64::new(seed).state_decimal(), "né il valore di genesi");
}

// ---------------------------------------------------------------------------
// head = hash dell'ultimo evento, NON quello della genesi
// ---------------------------------------------------------------------------

#[test]
fn test_head_is_hash_of_last_event_not_genesis() {
    let mut log = Log::genesis(9);
    log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(5))])).unwrap();
    log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("x".to_string()))])).unwrap();
    log.verify().expect("log verificato");
    let genesis_hash = log.events()[0].hash.clone();
    let last_hash = log.events().last().unwrap().hash.clone();
    assert_ne!(genesis_hash, last_hash);
    let s = State::fold(&log).unwrap();
    assert_eq!(s.head, last_hash, "head segue l'ultimo evento applicato, in ordine di seq");
    assert_ne!(s.head, genesis_hash, "head NON resta fermo all'hash della genesi");
}

// ---------------------------------------------------------------------------
// Evento compensativo (§3, opzione A del dispatch C1): è un STATE
// AUTODESCRITTIVO — porta lo snapshot dell'ultimo STATE precedente
// ---------------------------------------------------------------------------

/// Il compensativo porta lo STESSO snapshot dell'ultimo STATE precedente e la
/// fold lo legge da LÌ (opzione A): nessuno STATE non-autodescrittivo, la
/// fold non ha più bisogno di ricordarsi l'«ultimo visto».
#[test]
fn test_compensating_state_carries_last_snapshot_and_fold_reads_it() {
    let mut log = Log::genesis(1042);
    log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(3))])).unwrap();
    let comp = log.compensate(1, "stimolo errato", 1).unwrap();
    log.verify().expect("log con correzione compensativa è verificato");
    // l'unico STATE precedente è la genesi: il compensativo ne copia lo snapshot
    assert_eq!(
        comp.payload.get("state"),
        Some(&Value::Str(GENESIS_1042_STATE_DEC.to_string())),
        "il compensativo porta lo snapshot dell'ultimo STATE precedente (qui la genesi)"
    );
    let s = State::fold(&log).unwrap();
    assert_eq!(s.prng, GENESIS_1042_STATE_DEC, "fold: prng è ciò che il compensativo PORTE");
    assert_eq!(s.sense, 1, "gli eventi applicati contano ancora");
    assert_eq!(s.action, 0);
    assert_eq!(s.head, comp.hash, "head = ultimo evento, che è il compensativo");
    assert_eq!(s.tick, 1);
}

/// Il compensativo NON avanza il PRNG: lo snapshot dopo la compensazione è
/// identico a prima (opzione A: lo snapshot del compensativo è COPIATO —
/// nessun draw, nessun ricalcolo in correzione).
#[test]
fn test_compensation_does_not_advance_prng() {
    let mut log = Log::genesis(1042);
    log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(3))])).unwrap();
    log.append(1, EventType::State, Payload::from_pairs(vec![("state", Value::Str(SNAP.to_string()))])).unwrap();
    log.verify().expect("log verificato");
    let before = State::fold(&log).unwrap();

    let comp = log.compensate(1, "stimolo errato", 1).unwrap();
    assert_eq!(
        comp.payload.get("state"),
        Some(&Value::Str(SNAP.to_string())),
        "lo snapshot del compensativo è copia dell'ultimo STATE, NON un nuovo draw"
    );
    assert_ne!(SNAP, GENESIS_1042_STATE_DEC, "controllo servente: i due snapshot distinguono");

    let after = State::fold(&log).unwrap();
    assert_eq!(after.prng, before.prng, "il PRNG NON avanza: correzione senza draw");
    assert_eq!(after.sense, before.sense);
    assert_eq!(after.action, before.action);
    assert_eq!(after.tick, before.tick);
    assert_eq!(after.head, comp.hash, "solo head avanza: c'è un nuovo evento in coda");
}

// ---------------------------------------------------------------------------
// La fold assume un log verificato: catena rotta → la fold fallisce
// ---------------------------------------------------------------------------

#[test]
fn test_fold_fails_on_broken_chain_before_any_verify() {
    // log valido di partenza, poi le due rotture della catena
    let mut log = Log::genesis(1001);
    log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(1))])).unwrap();
    log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("a".to_string()))])).unwrap();
    let jsonl = log.to_jsonl();

    // rottura 1: gap nello seq (seq 1 → 7). Il parse di schema lo ACCETTA
    // (from_jsonl non verifica la catena): il chiamante che salta verify()
    // arriva alla fold con una catena rotta, e la fold fallisce.
    let gapped = Log::from_jsonl(&jsonl.replacen(r#""seq":1,"#, r#""seq":7,"#, 1)).unwrap();
    assert!(gapped.verify().is_err(), "verify sarebbe fallito: la catena è rotta");
    match State::fold(&gapped) {
        Err(StateError::Chain(m)) => assert!(
            m.contains("seq"),
            "il messaggio segnala il buco nello seq: {m}"
        ),
        other => panic!("gap di seq deve fallire la fold con errore di catena: {other:?}"),
    }

    // rottura 2: aggancio prev_hash disallineato. La fold (che non ricalcola
    // nessun hash) constata che i RIFERIMENTI memorizzati non si agganciano,
    // e rifiuta: nessuno stato è derivabile da una catena non agganciata.
    let target = format!("\"prev_hash\":\"{}\"", log.events()[1].prev_hash);
    let undone = jsonl.replacen(&target, &format!("\"prev_hash\":\"{}\"", "0".repeat(64)), 1);
    let unhooked = Log::from_jsonl(&undone).unwrap();
    assert!(unhooked.verify().is_err(), "verify sarebbe fallito anche qui");
    match State::fold(&unhooked) {
        Err(StateError::Chain(m)) => assert!(
            m.contains("prev_hash"),
            "il messaggio segnala il disallineamento: {m}"
        ),
        other => panic!("prev_hash disallineato deve fallire la fold: {other:?}"),
    }

    // il log BUONO è immune da entrambe le rotture e la fold ci passa
    let good = Log::from_jsonl(&jsonl).unwrap();
    good.verify().expect("la genesi + append puliti formano una catena valida");
    assert!(State::fold(&good).is_ok(), "fold del log non corrotto a verde");
}

// ---------------------------------------------------------------------------
// Invariante su LOG REALE (genesi + N cicli + compensazione): ogni evento
// STATE porta `state`, e la fold concorda con l'ultimo
// ---------------------------------------------------------------------------

#[test]
fn test_real_log_every_state_carries_snapshot_and_fold_agrees() {
    use ariadne::cycle;
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    cycle::run_cycles(&mut log, &mut rng, 3).expect("3 cicli legali");

    // l'ultimo evento è uno STATE col suo snapshot, e l'engine concorda
    let ultimo = log.events().last().unwrap();
    let ultimo_snapshot = match ultimo.payload.get("state") {
        Some(Value::Str(s)) => s.clone(),
        other => panic!("ultimo STATE senza snapshot: {other:?}"),
    };
    assert_eq!(rng.state_decimal(), ultimo_snapshot, "snapshot in log ≠ snapshot d'engine: il log reale non è sincrono");

    // compensazione dell'ultimo STATE (opzione A: riparte con lo stesso snapshot)
    let comp = log.compensate(ultimo.seq, "correzione provata su log reale", ultimo.tick)
        .expect("compensazione legale dell'ultimo STATE");
    log.verify().expect("log reale con compensazione è verificabile end-to-end");
    assert_eq!(
        comp.payload.get("state"),
        Some(&Value::Str(ultimo_snapshot.clone())),
        "il compensativo porta lo snapshot dell'ultimo STATE precedente"
    );

    // INVARIANTE: nel log reale OGNI evento STATE (genesi, STATE di ciclo e
    // compensativo) porta `state`
    for e in log.events() {
        if e.r#type == EventType::State {
            assert!(
                matches!(e.payload.get("state"), Some(Value::Str(_))),
                "EVENTO STATE non autodescrittivo alla seq {}: {}", e.seq, e.payload.canonical()
            );
        }
    }

    // e la fold concorda con l'ultimo snapshot portato
    let s = State::fold(&log).unwrap();
    assert_eq!(s.prng, ultimo_snapshot, "fold: prng = snapshot dell'ultimo STATE");
    assert_eq!(s.head, comp.hash, "head = compensativo");
}
