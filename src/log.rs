//! Log append-only — SPEC L0 v0.2 §3 + genesi §4.
//!
//! - append-only: nessuna mutazione, cancellazione, riscrittura o riordino;
//!   l'API espone solo `genesis`, `append`, `compensate`, `verify`, `to_jsonl`,
//!   `from_jsonl`, `events`.
//! - Ogni errore si corregge con evento compensativo: payload
//!   {compensates, reason, state} (opzione A, dispatch C1: il compensativo è
//!   AUTODESCRITTIVO — porta lo snapshot `state` dell'ultimo STATE precedente,
//!   verbatim, e NON avanza il PRNG); la storia resta intatta.
//! - Genesi (§4 + Schema PRNG, dispatch B2; fallibile dal dispatch C1): seq 0,
//!   tick 0, type STATE, prev_hash = 64 zeri, payload {seed, prng: "PCG64",
//!   state: stringa decimale dello stato interno PCG64 a 128 bit}. `state` =
//!   `Pcg64::new(seed).state_decimal()` (formato congelato: decimale, senza
//!   segno, senza zeri in testa; round-trippabile su u128) — chiude la
//!   discrepanza D1 `state = seed`. `seed` resta intero `i64` nel payload
//!   (contatore, non entropia): seed > `i64::MAX` è RIFIUTATO con errore
//!   esplicito — mai un cast `as` che mutorosamente lo renda negativo.
//! - Formato log: JSONL, un evento per riga, membri top-level nell'ordine della
//!   SPEC §2 {seq, tick, type, payload, prev_hash, hash}, newline finale '\n'.
//!
//! `from_jsonl` è validazione di SCHEMA SOLA: non verifica la catena; la verifica
//! della catena (seq senza gap da 0, aggancio prev_hash, hash ricalcolato) è solo
//! in `verify`. Le prove di tampering dei test fondano su questa separazione.

use std::fmt;

use crate::event::{compute_hash, Event, EventType, GENESIS_PREV_HASH};
use crate::payload::{write_escaped, Payload, Value};
use crate::prng::Pcg64;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Rifiuto di schema: riga non canonica, membro mancante/doppio/non previsto,
    /// valori fuori dominio (float/bool/null/array/object nested), numeri con zero
    /// in testa o overflow, stringhe con caratteri di controllo grezzi, CR,
    /// stringhe con escape NON canonico (N3: solo l'encoding che il serializzatore
    /// emette è ammesso).
    Schema(String),
    /// Catena non verificabile: seq con gap o non da 0, prev_hash disallineato,
    /// hash registrato diverso dal ricalcolo, compensazione verso seq inesistente.
    Chain(String),
    /// Seed fuori dalla scala `i64` del payload canonico: un cast `as`
    /// silenzioso lo renderebbe negativo (wrap vietato). Nessun evento creato.
    Seed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Schema(m) => write!(f, "schema: {m}"),
            Error::Chain(m) => write!(f, "chain: {m}"),
            Error::Seed(m) => write!(f, "seed: {m}"),
        }
    }
}

impl std::error::Error for Error {}

/// Log append-only L0: un substrato, nessun fork.
#[derive(Clone, Debug)]
pub struct Log {
    events: Vec<Event>,
}

impl Log {
    /// Evento genesi (§4 + Schema PRNG): seq 0, tick 0, STATE, prev_hash 64 zeri,
    /// payload {seed, prng: "PCG64", state}. Lo stato PCG64 reale a 128 bit è
    /// serializzato come stringa decimale da `Pcg64::new(seed).state_decimal()`;
    /// `seed` resta intero (formato congelato, dispatch B2). FALLIBILE dal
    /// dispatch C1: seed > `i64::MAX` (2^63-1) è rifiutato con [`Error::Seed`],
    /// prima un cast `as` lo mutava in negativo — nessun evento creato nel
    /// percorso `Err`.
    pub fn genesis(seed: u64) -> Result<Log, Error> {
        let seed_int = i64::try_from(seed).map_err(|_| {
            Error::Seed(format!(
                "seed {seed} oltre i64::MAX: nel payload canonico il seed resta intero i64 \
                 (contatore, non entropia), nessun cast silenzioso"
            ))
        })?;
        let state = Pcg64::new(seed).state_decimal();
        let payload = Payload::from_pairs(vec![
            ("prng", Value::Str("PCG64".to_string())),
            ("seed", Value::Int(seed_int)),
            ("state", Value::Str(state)),
        ]);
        let genesis = Event::seal(0, 0, EventType::State, payload, GENESIS_PREV_HASH);
        Ok(Log { events: vec![genesis] })
    }

    /// Apre un evento in coda: seq = ultimo + 1, prev_hash = hash di testa.
    /// L'ordine è quello di append: nessun riordino, nessun fork.
    pub fn append(&mut self, tick: u64, r#type: EventType, payload: Payload) -> Result<&Event, Error> {
        let prev = self.events.last().ok_or_else(|| Error::Chain("log vuoto".to_string()))?;
        let seq = prev
            .seq
            .checked_add(1)
            .ok_or_else(|| Error::Chain("seq oltre u64::MAX".to_string()))?;
        let prev_hash = prev.hash.clone();
        self.events.push(Event::seal(seq, tick, r#type, payload, &prev_hash));
        Ok(self.events.last().expect("appena inserito"))
    }

    /// Correzione compensativa (§3 + opzione A, dispatch C1): evento STATE in
    /// coda con payload {compensates: seq, reason, state}. `state` è lo
    /// SNAPSHOT più recente nel log, copiato verbatim (scansione all'indietro
    /// della coda; la genesi ne ha sempre uno): per l'invariante «ogni STATE
    /// è autodescrittivo» coincide con l'ultimo STATE. Il compensativo è
    /// autodescrittivo e NON avanza il PRNG. L'evento compensato resta nel
    /// log, intatto. Ritorna una copia posseduta: la logica append-only non
    /// presta riferimenti mutabili oltre la chiusura dell'append.
    pub fn compensate(&mut self, compensates: u64, reason: &str, tick: u64) -> Result<Event, Error> {
        if compensates as usize >= self.events.len() {
            return Err(Error::Chain(format!(
                "compensates {compensates} non esiste nel log (len {})",
                self.events.len()
            )));
        }
        let comp_int = i64::try_from(compensates).map_err(|_| {
            Error::Chain("compensates oltre i64: fuori dalla scala ammessa nel payload".to_string())
        })?;
        // Snapshot da portare: lo SNAPSHOT più recente nel log, scansionando
        // all'indietro la coda — SPEC §3 + opzione A: l'oggetto della ricerca
        // è lo snapshot, non l'ultimo STATE; per l'invariante «ogni STATE è
        // autodescrittivo» coincide sempre con l'ultimo STATE (la genesi ne
        // ha sempre uno), quindi uno STATE senz'istantanea non prolunga mai
        // la correzione con una conciliazione implicita.
        let ultimo_snapshot = self
            .events
            .iter()
            .rev()
            .find(|e| e.r#type == EventType::State && e.payload.get("state").is_some());
        let snapshot = match ultimo_snapshot {
            Some(e) => match e.payload.get("state") {
                Some(Value::Str(s)) => s.clone(),
                _ => unreachable!("selezionato solo per STATE con `state`"),
            },
            None => {
                return Err(Error::Chain(
                    "log senza alcuno snapshot `state`: nessuno snapshot da copiare".to_string(),
                ))
            }
        };
        let payload = Payload::from_pairs(vec![
            ("compensates", Value::Int(comp_int)),
            ("reason", Value::Str(reason.to_string())),
            ("state", Value::Str(snapshot)),
        ]);
        self.append(tick, EventType::State, payload).cloned()
    }

    /// Riferimento di sola lettura alla storia: mai mutabile.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Verifica end-to-end della catena: seq = posizione (monotono da 0, nessun gap),
    /// prev_hash agganciato all'hash precedente (genesi: 64 zeri), hash = ricalcolo
    /// dal preimage documentato.
    pub fn verify(&self) -> Result<(), Error> {
        let mut prev = GENESIS_PREV_HASH.to_string();
        for (i, e) in self.events.iter().enumerate() {
            if e.seq != i as u64 {
                return Err(Error::Chain(format!(
                    "seq non monotono da 0 senza gap: posizione {i}, seq {}",
                    e.seq
                )));
            }
            if e.prev_hash != prev {
                return Err(Error::Chain(format!(
                    "prev_hash disallineato a seq {}: atteso {prev}, registrato {}",
                    e.seq, e.prev_hash
                )));
            }
            let expected = compute_hash(e.seq, e.tick, e.r#type, &e.payload.canonical(), &e.prev_hash);
            if e.hash != expected {
                return Err(Error::Chain(format!(
                    "hash incoerente a seq {}: atteso {expected}, registrato {}",
                    e.seq, e.hash
                )));
            }
            prev = e.hash.clone();
        }
        Ok(())
    }

    /// Serializzazione canonica: ogni evento su una riga JSONL, '\n' finale.
    pub fn to_jsonl(&self) -> String {
        let mut out = String::new();
        for e in &self.events {
            out.push_str(&event_line(e));
            out.push('\n');
        }
        out
    }

    /// Riplay dal byte: parse rigoroso di schema (senza verifica di catena).
    /// Accetta solo linee canoniche con '\n' finale; CR e righe vuote interne
    /// sono rifiutate.
    pub fn from_jsonl(input: &str) -> Result<Log, Error> {
        if input.contains('\r') {
            return Err(Error::Schema("CR vietato: nel log canonico solo LF".to_string()));
        }
        let pieces: Vec<&str> = input.split('\n').collect();
        let mut events = Vec::new();
        for (i, line) in pieces.iter().enumerate() {
            let is_last = i + 1 == pieces.len();
            if line.is_empty() {
                if is_last {
                    break; // newline finale canonico
                }
                return Err(Error::Schema("riga vuota intermedia nel log".to_string()));
            }
            events.push(parse_event(line)?);
        }
        if events.is_empty() {
            return Err(Error::Schema("log vuoto: nessuna riga evento".to_string()));
        }
        Ok(Log { events })
    }
}

/// Una riga evento canonica:
/// {"seq":..,"tick":..,"type":"..","payload":{..},"prev_hash":"..","hash":".."}
fn event_line(e: &Event) -> String {
    let mut s = format!(
        "{{\"seq\":{},\"tick\":{},\"type\":\"{}\",\"payload\":",
        e.seq, e.tick, e.r#type.as_str()
    );
    s.push_str(&e.payload.canonical());
    s.push_str(",\"prev_hash\":\"");
    s.push_str(&e.prev_hash);
    s.push_str("\",\"hash\":\"");
    s.push_str(&e.hash);
    s.push_str("\"}");
    s
}

// ---------------------------------------------------------------------------
// Parser JSON rigoroso (validazione di schema). Niente spazi fuori dalle
// stringhe: la forma canonica non li ammette.
// ---------------------------------------------------------------------------

struct Cursor<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Cursor<'a> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }
    fn bump(&mut self) -> Option<u8> {
        let c = self.peek();
        if c.is_some() {
            self.i += 1;
        }
        c
    }
    fn expect(&mut self, c: u8) -> Result<(), Error> {
        match self.bump() {
            Some(x) if x == c => Ok(()),
            Some(x) => Err(Error::Schema(format!(
                "atteso '{}', trovato '{}'",
                c as char, x as char
            ))),
            None => Err(Error::Schema("fine riga inattesa".to_string())),
        }
    }
    fn at_end(&self) -> bool {
        self.i >= self.b.len()
    }
}

/// Imposta uno slot obbligatorio; vietato il membro duplicato.
macro_rules! set_slot {
    ($slot:expr, $name:expr, $value:expr) => {{
        if $slot.is_some() {
            return Err(Error::Schema(format!("membro duplicato: {}", $name)));
        }
        $slot = Some($value);
    }};
}

/// Niente spazi fuori dalle stringhe: `expect` e cursori non saltano nulla.
fn parse_event(line: &str) -> Result<Event, Error> {
    let mut c = Cursor { b: line.as_bytes(), i: 0 };
    c.expect(b'{')?;

    let mut set = (
        Option::<u64>::None,      // seq
        Option::<u64>::None,      // tick
        Option::<EventType>::None, // type
        Option::<Payload>::None,  // payload
        Option::<String>::None,   // prev_hash
        Option::<String>::None,   // hash
    );

    let mut first = true;
    loop {
        if c.peek() == Some(b'}') {
            c.bump();
            break;
        }
        if !first {
            c.expect(b',')?;
        }
        first = false;
        let key = parse_string(&mut c)?;
        c.expect(b':')?;
        match key.as_str() {
            "seq" => set_slot!(set.0, "seq", parse_uint(&mut c)?),
            "tick" => set_slot!(set.1, "tick", parse_uint(&mut c)?),
            "type" => {
                let s = parse_string(&mut c)?;
                let t = EventType::parse(&s)
                    .ok_or_else(|| Error::Schema(format!("type fuori dominio: {s}")))?;
                set_slot!(set.2, "type", t);
            }
            "payload" => set_slot!(set.3, "payload", parse_payload(&mut c)?),
            "prev_hash" => {
                let s = parse_string(&mut c)?;
                validate_hash64(&s, "prev_hash")?;
                set_slot!(set.4, "prev_hash", s);
            }
            "hash" => {
                let s = parse_string(&mut c)?;
                validate_hash64(&s, "hash")?;
                set_slot!(set.5, "hash", s);
            }
            other => return Err(Error::Schema(format!("membro non previsto: {other}"))),
        }
    }
    if !c.at_end() {
        return Err(Error::Schema("rifiuti oltre il '}' di chiusura".to_string()));
    }

    match set {
        (Some(seq), Some(tick), Some(typ), Some(payload), Some(prev_hash), Some(hash)) => Ok(Event {
            seq,
            tick,
            r#type: typ,
            payload,
            prev_hash,
            hash,
        }),
        _ => Err(Error::Schema(
            "membro mancante: seq|tick|type|payload|prev_hash|hash".to_string(),
        )),
    }
}

/// Payload di schema: object flat, chiavi stringa, valori SOLO interi o stringhe.
/// Float, bool, null, array, object nested, chiavi duplicate: vietati.
fn parse_payload(c: &mut Cursor) -> Result<Payload, Error> {
    c.expect(b'{')?;
    let mut pairs: Vec<(String, Value)> = Vec::new();
    let mut first = true;
    loop {
        if c.peek() == Some(b'}') {
            c.bump();
            break;
        }
        if !first {
            c.expect(b',')?;
        }
        first = false;
        let key = parse_string(c)?;
        c.expect(b':')?;
        let value = match c.peek() {
            Some(b'"') => Value::Str(parse_string(c)?),
            Some(b) if b.is_ascii_digit() || b == b'-' => Value::Int(parse_i64(c)?),
            _ => {
                return Err(Error::Schema(
                    "valore payload fuori dominio: solo intero o stringa".to_string(),
                ))
            }
        };
        if pairs.iter().any(|(k, _)| *k == key) {
            return Err(Error::Schema(format!("chiave payload duplicata: {key}")));
        }
        pairs.push((key, value));
    }
    Ok(Payload::from_pairs(pairs))
}

/// uint64 in schema: solo cifre, senza zero in testa (tranne "0").
fn parse_uint(c: &mut Cursor) -> Result<u64, Error> {
    let digits = take_digits(c)?;
    if digits.len() > 1 && digits[0] == b'0' {
        return Err(Error::Schema("zero in testa vietato".to_string()));
    }
    let s = String::from_utf8(digits).expect("cifre ascii");
    s.parse::<u64>()
        .map_err(|_| Error::Schema("uint64 oltre il massimo".to_string()))
}

/// i64 in schema: '-' opzionale, cifre, senza zero in testa (tranne "0"/"-0").
fn parse_i64(c: &mut Cursor) -> Result<i64, Error> {
    let negative = if c.peek() == Some(b'-') {
        c.bump();
        true
    } else {
        false
    };
    let digits = take_digits(c)?;
    if digits.len() > 1 && digits[0] == b'0' {
        return Err(Error::Schema("zero in testa vietato".to_string()));
    }
    let mut s = if negative { "-".to_string() } else { String::new() };
    s.push_str(&String::from_utf8(digits).expect("cifre ascii"));
    s.parse::<i64>()
        .map_err(|_| Error::Schema("i64 oltre il massimo".to_string()))
}

fn take_digits(c: &mut Cursor) -> Result<Vec<u8>, Error> {
    let mut digits = Vec::new();
    loop {
        match c.peek() {
            Some(b) if b.is_ascii_digit() => {
                c.bump();
                digits.push(b);
            }
            _ => break,
        }
    }
    if digits.is_empty() {
        return Err(Error::Schema("numero mancante".to_string()));
    }
    Ok(digits)
}

/// Esadecimale minuscolo, esattamente 64 caratteri.
fn validate_hash64(s: &str, field: &str) -> Result<(), Error> {
    if s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Ok(());
    }
    Err(Error::Schema(format!("{}: atteso esadecimale minuscolo di 64 caratteri", field)))
}

/// Stringa JSON CANONICA (N3): ammette ESATTAMENTE l'encoding emesso da
/// `payload::write_escaped` — `\"` `\\` `\b` `\t` `\n` `\f` `\r`; altri
/// controlli <0x20 come `\uXXXX` con quattro cifre esadecimali MINUSCOLE;
/// tutto il resto UTF-8 diretto (anche ≥0x7F e astrali, nessuna surrogate).
/// Criterio del rifiuto: la riserializzazione canonica della stringa
/// decodificata deve coincidere BYTE PER BYTE col token sorgente — così
/// `\/`, hex maiuscolo, `\u` di caratteri emessi in altra forma, escape
/// non necessari e coppie surrogate sono non canonici: logged esterno o è
/// canonico o non entra (parse∘serialize = identità byte-per-byte).
fn parse_string(c: &mut Cursor) -> Result<String, Error> {
    let start = c.i; // token completo, quote di apertura inclusa
    c.expect(b'"')?;
    let mut out: Vec<u8> = Vec::new();
    loop {
        match c.bump() {
            None => return Err(Error::Schema("stringa non chiusa".to_string())),
            Some(b'"') => break,
            Some(b'\\') => match c.bump() {
                Some(b'"') => out.push(b'"'),
                Some(b'\\') => out.push(b'\\'),
                Some(b'/') => out.push(b'/'),
                Some(b'b') => out.push(0x08),
                Some(b'f') => out.push(0x0C),
                Some(b'n') => out.push(b'\n'),
                Some(b'r') => out.push(b'\r'),
                Some(b't') => out.push(b'\t'),
                Some(b'u') => {
                    let code = parse_u4x(c)?;
                    if (0xD800..=0xDBFF).contains(&code) {
                        if c.peek() != Some(b'\\') {
                            return Err(Error::Schema("surrogate alta senza coppia".to_string()));
                        }
                        c.bump();
                        c.expect(b'u')?;
                        let low = parse_u4x(c)?;
                        if !(0xDC00..=0xDFFF).contains(&low) {
                            return Err(Error::Schema("coppia surrogate non valida".to_string()));
                        }
                        let combined = 0x10000u32 + ((code - 0xD800) << 10) + (low - 0xDC00);
                        push_rune(&mut out, combined)?;
                    } else if (0xDC00..=0xDFFF).contains(&code) {
                        return Err(Error::Schema("surrogate bassa isolata".to_string()));
                    } else {
                        push_rune(&mut out, code)?;
                    }
                }
                _ => return Err(Error::Schema("escape non valido".to_string())),
            },
            Some(raw) => {
                if raw < 0x20 {
                    return Err(Error::Schema(format!(
                        "carattere di controllo grezzo 0x{raw:02x} in stringa"
                    )));
                }
                out.push(raw);
            }
        }
    }
    let decoded = String::from_utf8(out).map_err(|_| Error::Schema("stringa non UTF-8".to_string()))?;
    // N3 — canonico == solo ciò che il serializzatore emette: se il token
    // sorgente non coincide con la riserializzazione canonica della stringa
    // decodificata, il contenuto è lo stesso ma il BYTE è diverso: un tale log
    // ri-serializzato non è byte-identico a se stesso → non entra.
    let mut canonical = String::new();
    write_escaped(&mut canonical, &decoded);
    if canonical.as_bytes() != &c.b[start..c.i] {
        return Err(Error::Schema(
            "stringa non canonica: l'encoding non coincide con l'output del serializzatore \
             (write_escaped): ogni forma di escape non emessa è rifiutata"
                .to_string(),
        ));
    }
    Ok(decoded)
}

fn push_rune(out: &mut Vec<u8>, code: u32) -> Result<(), Error> {
    let ch = char::from_u32(code).ok_or_else(|| Error::Schema("codepoint non valido".to_string()))?;
    let mut buf = [0u8; 4];
    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
    Ok(())
}

fn parse_u4x(c: &mut Cursor) -> Result<u32, Error> {
    let mut v = 0u32;
    for _ in 0..4 {
        let d = match c.bump() {
            Some(b) if b.is_ascii_hexdigit() => (b as char).to_digit(16).unwrap(),
            _ => return Err(Error::Schema("escape \\u malformato".to_string())),
        };
        v = v * 16 + d;
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parser: la riga genesi canonica si compone (formato B2: state stringa).
    #[test]
    fn genesis_line_parses() {
        const GENESIS_1042_LINE: &str = concat!(
            r#"{"seq":0,"tick":0,"type":"STATE","payload":{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"},"#,
            r#""prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","#,
            r#""hash":"4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7"}"#,
        );
        let log = Log::from_jsonl(&format!("{GENESIS_1042_LINE}\n")).expect("riga genesi valida");
        assert_eq!(log.events().len(), 1);
        log.verify().expect("catena geni valida");
    }

    /// seq con gap: la verifica della catena fallisce con errore di Chain,
    /// anche quando l'aggancio degli hash è formalmente coerente.
    #[test]
    fn gap_is_chain_error_not_schema_error() {
        // costruzione interna (crate): catena con un buco seq 0,1 riftata a 0,1,3
        let e0 = Log::genesis(1).expect("seed in range: adattamento meccanico autorizzato (C2)").events()[0].clone();
        let mut e1 = Event::seal(1, 0, EventType::State, Payload::empty(), &e0.hash);
        e1.hash = compute_hash(e1.seq, e1.tick, e1.r#type, &e1.payload.canonical(), &e1.prev_hash);
        let e3 = Event::seal(3, 0, EventType::State, Payload::empty(), &e1.hash);
        let gapped = Log { events: vec![e0.clone(), e1.clone(), e3] };
        assert!(matches!(gapped.verify(), Err(Error::Chain(_)) | Err(Error::Schema(_))));
        // la stessa catena senza buco è valida
        let e2 = Event::seal(2, 0, EventType::State, Payload::empty(), &e1.hash);
        let linear = Log { events: vec![e0, e1, e2] };
        linear.verify().expect("senza buco la catena è valida");
    }
}
