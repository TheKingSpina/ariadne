//! L0 — SPEC v0.2 §2 (formato evento), §3 (append-only e correzioni), genesi §4.
//!
//! Questo file è il commit TEST-ONLY: se resta solo qui, la suite è ROSSA
//! perché il codice di produzione non esiste ancora.
//!
//! Contratto testato:
//! - evento {seq, tick, type, payload, prev_hash, hash};
//! - hash = SHA-256 del preimage separato da 0x1F:
//!     seq_dec ‖ 0x1F ‖ tick_dec ‖ 0x1F ‖ type ‖ 0x1F ‖ payload_json ‖ 0x1F ‖ prev_hash
//!   (esadecimale minuscolo; encoding fissato perché la SPEC §2 non specifica
//!   l'encoding della concatenazione: 0x1F non compare in nessun campo quindi
//!   il preimage è privo di ambiguità: un tampering di seq/tick non produce lo stesso hash).
//! - payload: JSON canonico, chiavi ordinate alfabeticamente, valori solo interi i64 e stringhe;
//! - genesì: seq 0, STATE, prev_hash = 64 zeri, payload {seed, prng:"PCG64", state};
//! - log append-only: nessuna mutazione retroattiva; errore → evento compensativo
//!   {compensates, reason, state} (opzione A del dispatch C1: il compensativo è
//!   AUTODESCRITTIVO, lo snapshot `state` è copia verbatim dell'ultimo STATE).

use ariadne::{EventType, Log, Payload, Value};

const ZERO64: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Hash di riferimento della genesi seed 1042, precomputato a mano
/// (SHA-256 del preimage sopra documentato, calcolato indipendentemente dal
/// crate con `shasum -a 256` su byte scritti a mano — nessuna funzione del crate).
///
/// Formato B2 (dispatch, chiude D1): `state` è la STRINGA decimale dello stato
/// interno PCG64 a 128 bit (`Pcg64::new(1042).state_decimal()`), non più un intero.
const GENESIS_1042_HASH: &str = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";

/// Stato interno PCG64 a 128 bit per seed 1042, ottenuto da `Pcg64::new(1042)
/// .state_decimal()` del crate esistente (formato congelato: decimale, senza
/// segno, senza zeri in testa).
const GENESIS_1042_STATE_DEC: &str = "43149974864253806286336044685702662359";

const GENESIS_1042_LINE: &str = concat!(
    r#"{"seq":0,"tick":0,"type":"STATE","payload":{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"},"#,
    r#""prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","#,
    r#""hash":"4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7"}"#,
);

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Preimage della SPEC §2: seq ‖ tick ‖ type ‖ payload ‖ prev_hash, separati da 0x1F.
fn preimage(seq: u64, tick: u64, event_type: &str, payload_json: &str, prev_hash: &str) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(seq.to_string().as_bytes());
    v.push(0x1F);
    v.extend_from_slice(tick.to_string().as_bytes());
    v.push(0x1F);
    v.extend_from_slice(event_type.as_bytes());
    v.push(0x1F);
    v.extend_from_slice(payload_json.as_bytes());
    v.push(0x1F);
    v.extend_from_slice(prev_hash.as_bytes());
    v
}

/// Log di prova: genesi + tre eventi con payload {"gain": n} (nessuna collusione
/// con le sottostringhe "seq" o "hash" usate dai test di tampering).
fn log_di_prova() -> Log {
    let mut log = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)");
    for (tick, gain) in [(1, 1i64), (1, 2), (1, 3)] {
        log.append(tick, EventType::State, Payload::from_pairs(vec![("gain", Value::Int(gain))]))
            .unwrap();
    }
    log
}

// ---------------------------------------------------------------------------
// Genesi (§4 + schema PRNG)
// ---------------------------------------------------------------------------

#[test]
fn test_genesis_prev_hash_zeroes_and_shape() {
    let log = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)");
    let events = log.events();
    assert_eq!(events.len(), 1, "la genesi è il solo evento iniziale");
    let g = &events[0];
    assert_eq!(g.seq, 0, "la genesi è seq 0");
    assert_eq!(g.tick, 0, "tick contatore logico, parte da 0");
    assert_eq!(g.r#type.as_str(), "STATE", "la genesi è type STATE");
    assert_eq!(g.prev_hash, ZERO64, "prev_hash della genesi = 64 zeri");
    assert_eq!(g.payload.canonical(),
        r#"{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"}"#,
        "payload genesi: seed intero, prng PCG64, state STRINGA decimale");
    assert_eq!(g.hash.len(), 64);
    assert!(g.hash.bytes().all(|b| b.is_ascii_hexdigit() && b == b.to_ascii_lowercase()),
        "hash esadecimale minuscolo");
    assert_eq!(log.verify().map_err(|e| format!("{e:?}")), Ok(()), "la genesi da sola è valida");
}

#[test]
fn test_genesis_golden_hash_constant() {
    // Il byte-format del preimage è fissato: l'hash della genesi con seed 1042
    // deve agganciarsi a questo valore precomputato dall'esterno (shasum -a 256
    // su byte scritti a mano, separatore 0x1F — nessuna funzione del crate).
    let log = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)");
    assert_eq!(log.events()[0].hash, GENESIS_1042_HASH);
    // e la serializzazione canonica della riga genesi è byte per byte quella attesa
    assert_eq!(log.to_jsonl(), format!("{GENESIS_1042_LINE}\n"));
}

// ---------------------------------------------------------------------------
// Formato B2 — `state` stringa decimale, coerenza engine ↔ log
// ---------------------------------------------------------------------------

#[test]
fn test_genesis_state_is_string_not_integer() {
    // chiude D1: `state` nel payload canonico è una stringa JSON ("..."),
    // non un intero; `seed` resta intero.
    let payload = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)").events()[0].payload.canonical();
    assert!(payload.contains(r#""state":"43149974864253806286336044685702662359""#),
        "state è stringa: {payload}");
    assert!(payload.contains(r#""seed":1042,"#),
        "seed resta intero: {payload}");
    assert!(!payload.contains(r#""state":431499"#),
        "state NON è più un intero grezzo: {payload}");
}

#[test]
fn test_genesis_state_matches_engine_state_decimal() {
    // coerenza engine ↔ log: lo stato scritto nel payload genesi è ESATTAMENTE
    // quello che Pcg64::new(seed).state_decimal() rende dall'engine esistente.
    use ariadne::prng::Pcg64;
    let atteso = Pcg64::new(1042).state_decimal();
    assert_eq!(atteso, GENESIS_1042_STATE_DEC, "lo stato d'engine non è cambiato");
    let log = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)");
    let payload = log.events()[0].payload.clone();
    let state = match payload.get("state") {
        Some(ariadne::Value::Str(s)) => s.clone(),
        other => panic!("state del payload genesi non è una stringa: {other:?}"),
    };
    assert_eq!(state, atteso);
}

#[test]
fn test_genesis_state_string_format() {
    // formato congelato del campo `state`: solo cifre decimali, nessun segno,
    // nessuno zero in testa, al più 39 caratteri (128 bit).
    let state = match Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)").events()[0].payload.get("state") {
        Some(ariadne::Value::Str(s)) => s.clone(),
        other => panic!("state non è una stringa: {other:?}"),
    };
    assert!(!state.is_empty(), "state non vuoto");
    assert!(state.bytes().all(|b| b.is_ascii_digit()), "state: solo cifre");
    assert!(!state.starts_with('+') && !state.starts_with('-'), "state: nessun segno");
    assert!(!state.starts_with('0'), "state: nessuno zero in testa");
    assert!(state.len() <= 39, "state oltre 128 bit: {} cifre", state.len());
}

// ---------------------------------------------------------------------------
// §2 — catena di hash
// ---------------------------------------------------------------------------

#[test]
fn test_hash_chain_end_to_end() {
    let log = log_di_prova();
    log.verify().expect("catena valida end-to-end");

    // verifica indipendente, evento per evento, del preimage e dell'aggancio
    let mut prev = ZERO64.to_string();
    for (i, e) in log.events().iter().enumerate() {
        assert_eq!(e.seq, i as u64, "seq monotono da 0 senza salti, evento {i}");
        assert_eq!(e.prev_hash, prev, "prev_hash agganciato all'hash precedente");
        let expected = sha256_hex(&preimage(
            e.seq, e.tick, e.r#type.as_str(), &e.payload.canonical(), &e.prev_hash,
        ));
        assert_eq!(e.hash, expected, "hash ricalcolato");
        assert!(e.hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        prev = e.hash.clone();
    }
}

#[test]
fn test_event_types_constrained_to_sense_action_state() {
    let mut log = Log::genesis(1).expect("seed in range: adattamento meccanico autorizzato (C2)");
    log.append(1, EventType::Sense, Payload::empty()).unwrap();
    log.append(1, EventType::Action, Payload::empty()).unwrap();
    let types: Vec<&str> = log.events().iter().map(|e| e.r#type.as_str()).collect();
    assert_eq!(types, vec!["STATE", "SENSE", "ACTION"]);
    // il parser di log rifiuta un type fuori dal dominio
    let jsonl = log.to_jsonl();
    let bad = jsonl.replace("\"SENSE\"", "\"SEE\"");
    assert!(Log::from_jsonl(&bad).is_err(), "type fuori da SENSE|ACTION|STATE rifiutato");
}

#[test]
fn test_unknown_top_level_member_rejected() {
    // zero wall-clock: la schema evento non ha spazio per un timestamp di nessun tipo
    let poisoned = GENESIS_1042_LINE
        .replacen(r#""type":"STATE""#, r#""ts":"","type":"STATE""#, 1);
    assert!(Log::from_jsonl(&poisoned).is_err(), "membro top-level non previsto rifiutato");
}

// ---------------------------------------------------------------------------
// §2 — payload JSON canonico
// ---------------------------------------------------------------------------

#[test]
fn test_payload_canonical_sorted_keys_minimal() {
    // chiavi inserite in ordine che NON è alfabetico: l'output deve ordinare
    let p = Payload::from_pairs(vec![
        ("zeta", Value::Int(-3)),
        ("alpha", Value::Str("s".to_string())),
        ("mik", Value::Int(7)),
        ("compensates", Value::Int(1)),
        ("reason", Value::Str("lo scoglio".to_string())),
    ]);
    assert_eq!(
        p.canonical(),
        r#"{"alpha":"s","compensates":1,"mik":7,"reason":"lo scoglio","zeta":-3}"#
    );
    // payload vuoto
    assert_eq!(Payload::empty().canonical(), "{}");
}

#[test]
fn test_parser_rejects_non_canonical_payload_values() {
    // la SPEC §2: payload solo interi e stringhe. Float, bool, null, array, object sono vietati.
    for (payload, label) in [
        (r#"{"gain":1.5}"#, "float vietato"),
        (r#"{"gain":true}"#, "bool vietato"),
        (r#"{"gain":null}"#, "null vietato"),
        (r#"{"gain":[1,2]}"#, "array vietato"),
        (r#"{"gain":{"a":1}}"#, "object nested vietato"),
        (r#"{"gain":5,"gain":5}"#, "chiave duplicata vietata"),
    ] {
        // sostituisci il blocco payload nella riga genesi con un payload viziato
        let line = poison_payload(GENESIS_1042_LINE, payload);
        assert!(Log::from_jsonl(&line).is_err(), "{label} deve essere rifiutato");
    }
}

/// rimpiazza il contenuto del membro "payload"
/// nella riga evento (dal `{` alla virgola che chiude il blocco payload)
fn poison_payload(line: &str, poisoned: &str) -> String {
    let marker = r#""payload":"#;
    let content = line.find(marker).unwrap() + marker.len();
    let inner = &line[content..];
    let comma = inner.find(',').unwrap();
    format!("{}{}", &line[..content], poisoned) + &line[content + comma..]
}

// ---------------------------------------------------------------------------
// §2/§3 — schema cablato: nessun evento senza seq; seq monotono da 0 senza gap
// ---------------------------------------------------------------------------

#[test]
fn test_event_without_seq_is_rejected() {
    // la riga della genesi senza il membro "seq": vietata dalla schema
    let headless = GENESIS_1042_LINE.replacen(r#""seq":0,"#, "", 1);
    assert!(Log::from_jsonl(&headless).is_err(), "evento senza seq di origine vietato");
    // anche il membro mancante in qualunque posizione: la schema lo vieta
    let headless2 = GENESIS_1042_LINE.replacen(r#""seq":0,"#, r#""seq":,"#, 1);
    assert!(Log::from_jsonl(&headless2).is_err());
}

#[test]
fn test_seq_monotonic_from_zero_no_gaps() {
    let log = log_di_prova();
    // catena valida: verify passa
    log.verify().expect("log lineare valido");
    // gap introdotto manipolando lo seq di un evento, la chain deve fallire
    let jsonl = log.to_jsonl();
    let checked_gap = jsonl.replacen(r#""seq":2,"#, r#""seq":7,"#, 1);
    let gap_log = Log::from_jsonl(&checked_gap).expect("il parse accetta seq 7 come uint64");
    assert!(gap_log.verify().is_err(), "gap nello seq vietato: verify deve fallire");
    // cancel non da 0
    let headstart = jsonl.replacen(r#""seq":0,"#, r#""seq":9,"#, 1);
    assert!(Log::from_jsonl(&headstart).unwrap().verify().is_err(), "persistenza da seq non 0 vietata");
}

// ---------------------------------------------------------------------------
// §3 — append-only, immutabilità dimostrata
// ---------------------------------------------------------------------------

#[test]
fn test_log_is_append_only_no_mutation_truncation_or_reorder() {
    // API: solo genesis/append/compensate/verify/to_jsonl/from_jsonl/events —
    // nessuna funzione eliminazione, riscrittura o cambio di ordine. Qui dimostriamo
    // che qualsiasi mutazione retroattiva dei byte del log rende la catena non verificabile.
    let log = log_di_prova();
    let jsonl = log.to_jsonl();

    // 1) cambio di un payload vecchio (seq 2): hash non più coerente
    let tampered_payload = jsonl.replacen(r#""gain":2"#, r#""gain":99"#, 1);
    assert!(Log::from_jsonl(&tampered_payload).unwrap().verify().is_err(),
        "un evento vecchio cambiato deve rompere la verifica");

    // 2) cambio dell'hash memorizzato di un evento vecchio (seq 1): la catena lo scopre
    let target = format!("\"hash\":\"{}\"", log.events()[1].hash);
    let replacement = format!("\"hash\":\"{}\"", "f".repeat(64));
    let tampered_hash = jsonl.replacen(&target, &replacement, 1);
    assert!(Log::from_jsonl(&tampered_hash).unwrap().verify().is_err());

    // 3) cambio del prev_hash memorizzato di un evento vecchio (seq 2): la catena lo scopre
    let target = format!("\"prev_hash\":\"{}\"", log.events()[2].prev_hash);
    let replacement = format!("\"prev_hash\":\"{}\"", "a".repeat(64));
    let tampered_prev = jsonl.replacen(&target, &replacement, 1);
    assert!(Log::from_jsonl(&tampered_prev).unwrap().verify().is_err());

    // 4) l'originale fuori da tampering resta valido: nessuna riscrittura "conforme" al volo
    Log::from_jsonl(&jsonl).unwrap().verify().expect("il log non toccato resta valido");
}

#[test]
fn test_replay_prefix_byte_per_byte() {
    // riplay del prefisso storico: costruito due volte indipendentemente → stesse byte
    let build = || {
        let mut log = Log::genesis(1000).expect("seed in range: adattamento meccanico autorizzato (C2)");
        log.append(1, EventType::Sense, Payload::from_pairs(vec![("stimolo", Value::Int(42))])).unwrap();
        log.append(1, EventType::Action, Payload::from_pairs(vec![("mossa", Value::Str("V".to_string()))])).unwrap();
        log.append(2, EventType::State, Payload::from_pairs(vec![("gain", Value::Int(6))])).unwrap();
        log
    };
    let jsonl_a = build().to_jsonl();
    let jsonl_b = build().to_jsonl();
    assert_eq!(jsonl_a.as_bytes(), jsonl_b.as_bytes(), "stesso log → stessi byte");

    // riplay: il log riparte dai byte e riproduce, byte per byte, la storia
    let replayed = Log::from_jsonl(&jsonl_a).expect("riplay dal prefisso riuscito");
    assert_eq!(replayed.to_jsonl().as_bytes(), jsonl_a.as_bytes(), "riplay byte per byte");
    replayed.verify().expect("prefisso ri-riprodotto verify a verde");

    // determinismo zero wall-clock: nessun timestamp nella rappresentazione
    assert!(!jsonl_a.contains("time"));
    assert!(!jsonl_a.contains("1970"));
}

// ---------------------------------------------------------------------------
// §3 — correzione compensativa
// ---------------------------------------------------------------------------

#[test]
fn test_compensating_event_payload_and_history_intact() {
    let mut log = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)");
    log.append(1, EventType::Action, Payload::from_pairs(vec![("gain", Value::Int(5))])).unwrap();
    let history_before = log.to_jsonl();

    let comp = log.compensate(1, "gain errato: il valore era 6", 2).unwrap();
    assert_eq!(comp.seq, 2);
    assert_eq!(comp.r#type.as_str(), "STATE", "l'evento compensativo è STATE");
    // l'unico STATE precedente è la genesi: il compensativo ne copia lo snapshot
    assert_eq!(
        comp.payload.canonical(),
        r#"{"compensates":1,"reason":"gain errato: il valore era 6","state":"43149974864253806286336044685702662359"}"#,
        "payload compensativo: compensates seq, reason, E lo snapshot dell'ultimo STATE (opzione A)"
    );
    // la storia resta intatta: ogni byte scritto prima del compensativo coincide
    let after = log.to_jsonl();
    let (prefix_before, _) = history_before.split_at(history_before.len() - 1); // meno il \n finale
    assert!(after.starts_with(prefix_before), "nessun byte del passato mutato dal compensativo");
    // e l'evento compensato esiste ancora, identico
    assert_eq!(comp.payload.canonical(), r#"{"compensates":1,"reason":"gain errato: il valore era 6","state":"43149974864253806286336044685702662359"}"#);
    assert_eq!(log.events()[1].payload.canonical(), r#"{"gain":5}"#, "l'evento errato rimane nel log");
    // catena intera valida
    log.verify().expect("log con correzione verificabile end-to-end");

    // compensazione verso un seq inesistente: vietata
    assert!(log.compensate(999, "fuori dal log", 3).is_err());
}

/// Payload compensativo ESATTO in tre chiavi (opzione A, C1): `{compensates,
/// reason, state}` con `state` = snapshot dell'ultimo STATE PRECEDENTE, non
/// della genesi se in mezzo ce n'è un altro.
#[test]
fn test_compensating_payload_carries_most_recent_state_snapshot_exact() {
    const SNAP_SECOND: &str = "918273645040506070809101112131415161718";
    let mut log = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)");
    log.append(1, EventType::Sense, Payload::empty()).unwrap();
    log.append(1, EventType::State, Payload::from_pairs(vec![
        ("state", Value::Str(SNAP_SECOND.to_string()))
    ])).unwrap();
    let history_before = log.to_jsonl();

    let comp = log.compensate(1, "stimolo errato: era 42", 2).unwrap();
    assert_eq!(comp.seq, 2);
    assert_eq!(comp.r#type.as_str(), "STATE", "l'evento compensativo è STATE");
    assert_eq!(
        comp.payload.canonical(),
        r#"{"compensates":1,"reason":"stimolo errato: era 42","state":"918273645040506070809101112131415161718"}"#,
        "payload compensativo in tre chiavi: STATE più RECENTE, non genesi"
    );
    assert_ne!(SNAP_SECOND, GENESIS_1042_STATE_DEC,
        "controllo servente: l'ultimo snapshot e quello di genesi sono distinguibili");
    // la storia resta intatta (invariante §3 non allentata dal nuovo payload)
    let after = log.to_jsonl();
    let (prefix_before, _) = history_before.split_at(history_before.len() - 1);
    assert!(after.starts_with(prefix_before), "nessun byte del passato mutato dal compensativo");
    assert_eq!(log.events()[2].payload.canonical(), r#"{"state":"918273645040506070809101112131415161718"}"#,
        "l'evento compensato resta nel log, intatto");
    log.verify().expect("log con correzione compensativa verificabile end-to-end");
}

/// Su un log di sola genesi: il compensativo copia lo snapshot di GENESI.
#[test]
fn test_compensating_state_on_genesis_only_log_copies_genesis_snapshot() {
    let mut log = Log::genesis(1042).expect("seed in range: adattamento meccanico autorizzato (C2)");
    let history_before = log.to_jsonl();
    let comp = log.compensate(0, "genesi corretta senza riscritture", 0).unwrap();
    assert_eq!(comp.seq, 1, "il compensativo è il secondo evento");
    assert_eq!(
        comp.payload.get("state"),
        Some(&Value::Str(GENESIS_1042_STATE_DEC.to_string())),
        "l'ultimo STATE è la genesi: lo snapshot copiato è il suo"
    );
    let after = log.to_jsonl();
    let (prefix_before, _) = history_before.split_at(history_before.len() - 1);
    assert!(after.starts_with(prefix_before), "nessun byte del passato mutato dal compensativo");
    log.verify().expect("genesi + compensativo è verificabile");
}

#[test]
fn test_immutable_events_slice_borrows() {
    // gli eventi in lettura sono immutabili: nessun mutatore; un clone dei byte è indipendente
    let mut log = log_di_prova();
    let len = log.events().len();
    // append dopo la copia del prefisso non cambia il prefisso
    let prefix = log.to_jsonl();
    log.compensate(1, "segno", 2).unwrap();
    let after = log.to_jsonl();
    assert!(after.starts_with(&prefix[..prefix.len() - 1]), "prefisso intatto");
    assert_eq!(log.events().len(), len + 1);
    assert_eq!(
        log.events()[len].payload.canonical(),
        r#"{"compensates":1,"reason":"segno","state":"43149974864253806286336044685702662359"}"#,
        "payload compensativo in tre chiavi: compensates, reason, state (opzione A)"
    );
}
