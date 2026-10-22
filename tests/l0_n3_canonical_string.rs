//! N3 — parser severo: `from_jsonl` ammette SOLO le forme che il serializzatore
//! emette (`write_escaped` in src/payload.rs):
//!   `\"` `\\` `\b` `\t` `\n` `\f` `\r`; per gli altri controlli <0x20 `\uXXXX`
//!   con QUATTRO cifre esadecimali MINUSCOLE; tutto il resto UTF-8 diretto
//!   (anche >=0x7F e astrali, niente coppie surrogate).
//! Decisione umana (dispatch N3, non negoziabile): «from_jsonl rifiuta
//! qualunque encoding che il serializzatore non emette. Il log esterno o è
//! canonico o non entra.»
//!
//! Proprietà fissata dai test:
//!   parse(serialize(x)) == x          (round-trip, valori invariati)
//!   serialize(parse(s)) == s          (byte-per-byte, per ogni s canonico)
//! e ogni forma fuori dall'insieme è un rifiuto di Schema.

use ariadne::{Error, Log, Payload, Value};

/// segnaposto prev_hash: from_jsonl è schema-only, la catena la verifica `verify`.
const HEX64_ZEROS: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/// segnaposto hash: esadecimale minuscolo di 64 caratteri, schema-valido.
const HEX64_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// riga evento schema-valida con payload arbitrario (la catena non è toccata qui).
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
// Gamma ammessa: le forme che il serializzatore emette, e nient'altro.
// ---------------------------------------------------------------------------

/// L'ATTO di emissione è fissato: `write_escaped` deve emettere ESATTAMENTE le
/// forme documentate (atteso costruito A MANO nel test, senza ri-usare il
/// serializzatore) e l'output deve round-trippare byte-per-byte.
#[test]
fn serializer_emission_set_is_pinned_and_round_trips_byte_exact() {
    // s = quote, backslash, TUTTI i controlli <0x20, DEL, accento, astrale.
    let mut s = String::from("\"\\");
    for code in 0u32..0x20 {
        s.push(char::from_u32(code).unwrap());
    }
    s.push('\u{7f}');
    s.push('é');
    s.push('\u{1f600}');

    let payload = Payload::from_pairs(vec![("k", Value::Str(s.clone()))]);
    let expected_payload = concat!(
        r#"{"k":"\"\\\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007\b\t\n\u000b\f\r\u000e\u000f\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017\u0018\u0019\u001a\u001b\u001c\u001d\u001e\u001f"#,
        "\u{7f}",
        r#"é😀"}"#
    );
    assert_eq!(
        payload.canonical(),
        expected_payload,
        "l'emissione deve coincidere con l'insieme documentato: short escape per \"\\ e i 5 controlli di escape standard, \\uXXXX minuscolo per gli altri <0x20, UTF-8 diretto per tutto il resto"
    );

    // serialize(parse(s)) == s: la riga canonica riserializzata è byte-identica.
    let parsed = parse_line(&payload.canonical())
        .expect("l'output del serializzatore è canonico: ammesso");
    assert_eq!(parsed, event_line(&payload.canonical()));

    // parse(serialize(x)) == x: singolarmente, ogni controllo <0x20 round-trippa.
    for code in 0u32..0x20 {
        let sc = String::from(char::from_u32(code).unwrap());
        let p = Payload::from_pairs(vec![("k", Value::Str(sc.clone()))]);
        let out = parse_line(&p.canonical())
            .unwrap_or_else(|e| panic!("char 0x{code:02x} canonico rifiutato: {e}"));
        assert_eq!(out, event_line(&p.canonical()));
    }
}

/// Controparti canoniche di ogni forma rifiutata: ammesse e byte-stabili.
#[test]
fn canonical_counterpart_forms_are_accepted() {
    let cases = [
        r#"{"k":"a/b"}"#,          // controparte di \/
        r#"{"k":"A"}"#,            // controparte di \u0041
        r#"{"k":"é"}"#,            // controparte di \u00e9
        r#"{"k":"😀"}"#,           // controparte delle surrogate
        r#"{"k":"\b\t\n\f\r"}"#,   // short escape canonici
        r#"{"k":"\u0001\u000b\u001f"}"#, // \uXXXX minuscolo solo dove emesso
        r#"{"/":"ok"}"#,           // '/' grezzo non solo nei valori
    ];
    for case in cases {
        let out = parse_line(case)
            .unwrap_or_else(|e| panic!("forma canonica rifiutata: {case} — {e}"));
        assert_eq!(out, event_line(case));
    }
}

/// Riga genesi seed 1042 canonica (formato B2, state stringa): ammessa e
/// riserializzata byte-per-byte.
#[test]
fn canonical_genesis_1042_line_reserializes_byte_exact() {
    const LINE: &str = concat!(
        r#"{"seq":0,"tick":0,"type":"STATE","payload":{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"},"#,
        r#""prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","#,
        r#""hash":"4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7"}"#,
    );
    let out = Log::from_jsonl(&format!("{LINE}\n")).expect("riga genesi: canonica");
    assert_eq!(out.to_jsonl(), format!("{LINE}\n"));
}

// ---------------------------------------------------------------------------
// Forme fuori dall'insieme: RIFIUTATE (decisione umana N3).
// ---------------------------------------------------------------------------

/// `\/`: il serializzatore emette '/' grezzo.
#[test]
fn rifiuta_escape_solidus() {
    let res = parse_line(r#"{"k":"a\/b"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        r#"\/ non è emesso dal serializzatore: "/​" esce grezzo → rifiuto atteso, trovato {:?}"#,
        res
    );
}

/// `\u0041` per 'A': escape non necessario di un carattere >=0x20.
#[test]
fn rifiuta_u_minuscolo_di_carattere_non_controllo() {
    let res = parse_line(r#"{"k":"a\u0041b"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "escape non necessario di carattere >=0x20 → rifiuto atteso, trovato {:?}",
        res
    );
}

/// `\u00e9` per 'é': anche i non-ASCII escono UTF-8 diretto.
#[test]
fn rifiuta_u_di_non_ascii() {
    let res = parse_line(r#"{"k":"a\u00e9b"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "non-ASCII esce UTF-8 diretto, non come \\uXXXX → rifiuto atteso, trovato {:?}",
        res
    );
}

/// `\u007f` per DEL: 0x7F non è <0x20, esce grezzo.
#[test]
fn rifiuta_u_di_carattere_che_esce_grezzo() {
    let res = parse_line(r#"{"k":"a\u007fb"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "0x7F esce grezzo, non come \\uXXXX → rifiuto atteso, trovato {:?}",
        res
    );
}

/// `\u0008` per BS: i 5 controlli di escape standard escono come `\b`, non `\u`.
#[test]
fn rifiuta_u_di_controllo_emesso_come_short_escape() {
    let res = parse_line(r#"{"k":"a\u0008b"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "0x08 è emesso come \\b, non come \\u0008 → rifiuto atteso, trovato {:?}",
        res
    );
}

/// `\u000A` per LF: esce come `\n`; inoltre l'hex maiuscolo non è mai emesso.
#[test]
fn rifiuta_u_maiuscolo_di_controllo_emesso_come_short_escape() {
    let res = parse_line(r#"{"k":"a\u000Ab"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "0x0A è emesso come \\n, e l'hex maiuscolo non è canonico → rifiuto atteso, trovato {:?}",
        res
    );
}

/// `\u000B` per VT: controlli NON di escape standard escono come `\uxxxx`
/// MINUSCOLO: l'unico case canonico qui è il minuscolo.
#[test]
fn rifiuta_u_maiuscolo_di_controllo_emesso_come_u_minuscolo() {
    let res = parse_line(r#"{"k":"a\u000Bb"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "0x0B è emesso come \\u000b minuscolo → rifiuto atteso, trovato {:?}",
        res
    );
    // la controparte minuscola è ammessa (coperta in canonical_counterpart_forms)
}

/// `\u0022` per '"': la quote esce come `\"`.
#[test]
fn rifiuta_u_di_quote() {
    let res = parse_line(r#"{"k":"a\u0022b"}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "0x22 è emesso come \\\" → rifiuto atteso, trovato {:?}",
        res
    );
}

/// Coppie surrogate per l'astrale '😀': i >=0x10000 escono UTF-8 diretto,
/// aucune surrogate è mai emessa (alta, bassa, in nessun case).
#[test]
fn rifiuta_surrogate_per_carattere_astrale() {
    let low = parse_line(r#"{"k":"a\ud83d\ude00b"}"#);
    let high = parse_line(r#"{"k":"a\uD83D\uDE00b"}"#);
    let lone = parse_line(r#"{"k":"a\ud800b"}"#);
    for (nome, res) in [("coppia minuscola", low), ("coppia maiuscola", high), ("alta isolata", lone)] {
        assert!(
            matches!(res, Err(Error::Schema(_))),
            "{nome}: surrogate mai emesse dal serializzatore → rifiuto atteso, trovato {:?}",
            res
        );
    }
}

/// Anche nelle CHIAVI: l'universo ammesso non cambia.
#[test]
fn rifiuta_escape_non_canonico_in_chiave() {
    let res = parse_line(r#"{"\/k":1}"#);
    assert!(
        matches!(res, Err(Error::Schema(_))),
        "l'escape severo vale per chiavi e valori → rifiuto atteso, trovato {:?}",
        res
    );
}
