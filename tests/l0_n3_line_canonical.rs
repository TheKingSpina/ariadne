//! N3-bis — canonicità di RIGA: `serialize(parse(s)) == s` per ogni riga `s`
//! che `from_jsonl` accetta, non solo per ogni token stringa.
//!
//! N3 (tests/l0_n3_canonical_string.rs) ha reso canonico il token stringa;
//! resta non canonico ciò che il parser accetta ma il serializzatore non
//! emette mai:
//!   1. l'ordine delle chiavi del payload (parse accumula le coppie
//!      nell'ordine sorgente, il BTreeMap le riordina);
//!   2. l'ordine dei membri top-level (parse dispaccia per nome, il
//!      serializzatore emette l'ordine fisso {seq, tick, type, payload,
//!      prev_hash, hash});
//!   3. `-0` (parse lo ammette, il serializzatore emette `0`).
//!
//! Decisione umana (già firmata, dispatch N3): «from_jsonl rifiuta qualunque
//! encoding che il serializzatore non emette. Il log esterno o è canonico o
//! non entra.» L'ordine delle chiavi e la forma dei numeri sono encoding.
//! Ogni riga fuori canonicità è un rifiuto di `Error::Schema`.

use ariadne::{Error, Log, Payload, Value};

/// segnaposto prev_hash: from_jsonl è schema-only, la catena la verifica `verify`.
const HEX64_ZEROS: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/// segnaposto hash: esadecimale minuscolo di 64 caratteri, schema-valido.
const HEX64_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// riga genesi seed 1042 (formato B2, state stringa), senza '\n' finale.
const GENESIS_1042_LINE: &str = concat!(
    r#"{"seq":0,"tick":0,"type":"STATE","payload":{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"},"#,
    r#""prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","#,
    r#""hash":"4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7"}"#,
);

/// riga evento schema-valida con payload arbitrario, in forma CANONICA
/// (membri nell'ordine della SPEC §2), '\n' finale incluso.
fn event_line(payload: &str) -> String {
    format!(
        "{{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{payload},\"prev_hash\":\"{HEX64_ZEROS}\",\"hash\":\"{HEX64_A}\"}}\n"
    )
}

fn parse_line(payload: &str) -> Result<String, Error> {
    let line = event_line(payload);
    let log = Log::from_jsonl(&line)?;
    Ok(log.to_jsonl())
}

// ---------------------------------------------------------------------------
// 1. Ordine delle chiavi del payload: il serializzatore emette SOLO l'ordine
//    alfabetico (BTreeMap); qualunque altro ordine è encoding non emesso.
// ---------------------------------------------------------------------------

#[test]
fn rifiuta_permutazione_chiavi_payload() {
    let non_canonici = [
        // due chiavi, ordine invertito rispetto all'alfabetico
        r#"{"stimolo":42,"gain":6}"#,
        // tre chiavi, ordine ruotato
        r#"{"c":3,"a":1,"b":2}"#,
        // due chiavi con stringa e intero, invertite
        r#"{"zeta":-3,"alpha":"s"}"#,
    ];
    for payload in non_canonici {
        let res = parse_line(payload);
        assert!(
            matches!(res, Err(Error::Schema(_))),
            "ordine chiavi payload non canonico ({payload}) → rifiuto atteso, trovato {:?}",
            res
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Ordine dei membri top-level: il serializzatore emette SOLO l'ordine
//    {seq, tick, type, payload, prev_hash, hash}; qualunque altra
//    disposizione è encoding non emesso.
// ---------------------------------------------------------------------------

#[test]
fn rifiuta_permutazione_membri_top_level() {
    let seq = r#""seq":1"#;
    let tick = r#""tick":7"#;
    let r#type = r#""type":"STATE""#;
    let payload = r#""payload":{"gain":6}"#;
    let prev = format!(r#""prev_hash":"{HEX64_ZEROS}""#);
    let hash = format!(r#""hash":"{HEX64_A}""#);

    let non_canonici = [
        // tick prima di seq
        format!("{{{tick},{seq},{type},{payload},{prev},{hash}}}"),
        // payload prima di type
        format!("{{{seq},{tick},{payload},{type},{prev},{hash}}}"),
        // hash prima di prev_hash
        format!("{{{seq},{tick},{type},{payload},{hash},{prev}}}"),
        // prev_hash prima di payload
        format!("{{{seq},{tick},{type},{prev},{payload},{hash}}}"),
    ];
    for line in non_canonici {
        let input = format!("{line}\n");
        let res = Log::from_jsonl(&input).map(|l| l.to_jsonl());
        assert!(
            matches!(res, Err(Error::Schema(_))),
            "ordine membri top-level non canonico ({line}) → rifiuto atteso, trovato {:?}",
            res
        );
    }
}

// ---------------------------------------------------------------------------
// 3. `-0`: il serializzatore emette `0`; `-0` è encoding non emesso.
//    (Per zeri in testa, +, esponenziale, spazi, null/true: già rifiuti del
//    parse strutturale oggi — vedasi parse_uint/parse_i64/parse_payload —
//    quindi non sono forme "accettate e non emesse" e non mutorosirsi qui.)
// ---------------------------------------------------------------------------

#[test]
fn rifiuta_meno_zero_su_seq() {
    let line = format!(
        r#"{{"seq":-0,"tick":7,"type":"STATE","payload":{{}},"prev_hash":"{HEX64_ZEROS}","hash":"{HEX64_A}"}}"#
    );
    let res = Log::from_jsonl(&format!("{line}\n")).map(|l| l.to_jsonl());
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "-0 su seq → rifiuto atteso, trovato {:?}",
        res
    );
}

#[test]
fn rifiuta_meno_zero_su_tick() {
    let line = format!(
        r#"{{"seq":1,"tick":-0,"type":"STATE","payload":{{}},"prev_hash":"{HEX64_ZEROS}","hash":"{HEX64_A}"}}"#
    );
    let res = Log::from_jsonl(&format!("{line}\n")).map(|l| l.to_jsonl());
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "-0 su tick → rifiuto atteso, trovato {:?}",
        res
    );
}

#[test]
fn rifiuta_meno_zero_su_valore_payload() {
    let res = parse_line(r#"{"gain":-0}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "-0 nel payload → rifiuto atteso, trovato {:?}",
        res
    );
}

// ---------------------------------------------------------------------------
// 4. Le righe canoniche restano accettate per ogni tipo di evento, e
//    serialize(parse(s)) == s byte-per-byte sull'intera riga ('\n' finale).
// ---------------------------------------------------------------------------

#[test]
fn riga_canonica_con_payload_byte_esatta() {
    let out = parse_line(r#"{"gain":6,"stimolo":42}"#)
        .expect("riga canonical: ammessa");
    assert_eq!(out, event_line(r#"{"gain":6,"stimolo":42}"#));
}

#[test]
fn riga_canonica_payload_vuoto_byte_esatta() {
    for r#type in ["SENSE", "ACTION"] {
        let line = format!(
            r#"{{"seq":1,"tick":7,"type":"{type}","payload":{{}},"prev_hash":"{HEX64_ZEROS}","hash":"{HEX64_A}"}}"#
        );
        let out = Log::from_jsonl(&format!("{line}\n")).expect("riga canonica con payload vuoto");
        let out = out.to_jsonl();
        assert_eq!(out, format!("{line}\n"), "{type} con payload vuoto: byte-per-byte");
    }
}

#[test]
fn riga_genesi_1042_accetta_e_byte_esatta() {
    let log = Log::from_jsonl(&format!("{GENESIS_1042_LINE}\n")).expect("riga genesi canonica");
    assert_eq!(log.to_jsonl(), format!("{GENESIS_1042_LINE}\n"));
    log.verify().expect("catena della genesi valida");
}

/// il contratto del '\n' finale non cambia: oggi `from_jsonl` accetta l'input
/// anche SENZA newline finale (l'ultima riga è l'evento); l'emissione canonica
/// (`to_jsonl`) lo ri-aggiunge.
#[test]
fn contratto_newline_finale_invariato() {
    let log = Log::from_jsonl(GENESIS_1042_LINE).expect("senza '\n' finale: ancora accetta");
    assert_eq!(log.to_jsonl(), format!("{GENESIS_1042_LINE}\n"));
}

/// la proprietà end-to-end su un log costruito via API: ogni riga che il
/// serializzatore emette è accetta, e il riplay è byte-per-byte.
#[test]
fn flight_log_via_api_round_trip_byte_per_byte() {
    let mut log = Log::genesis(1042).expect("seed in range");
    log.append(1, ariadne::EventType::Sense, Payload::empty()).unwrap();
    log.append(
        1,
        ariadne::EventType::Action,
        Payload::from_pairs(vec![("mossa", Value::Str("V".to_string()))]),
    )
    .unwrap();
    log.append(
        2,
        ariadne::EventType::State,
        Payload::from_pairs(vec![("gain", Value::Int(6))]),
    )
    .unwrap();
    let s = log.to_jsonl();
    let replay = Log::from_jsonl(&s).expect("l'output del serializzatore è canonico");
    assert_eq!(replay.to_jsonl(), s, "serialize(parse(s)) == s, log intero");
    replay.verify().expect("la catena regge dopo il riplay");
}
