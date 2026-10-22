//! Evento e catena di hash — SPEC L0 v0.2 §2.
//!
//! Formato evento: {seq, tick, type, payload, prev_hash, hash}.
//! - seq: uint64 monotono da 0, gap vietati (imposto in `Log::verify`).
//! - tick: contatore logico di ciclo — mai un timestamp (nessun wall-clock nel crate).
//! - type: SENSE | ACTION | STATE.
//! - payload: JSON canonico (vedi `payload.rs`).
//! - prev_hash: hash dell'evento precedente, "0"×64 per la genesi.
//! - hash = SHA-256 del preimage, esadecimale minuscolo.
//!
//! Preimage (encoding fissato, registrato in governance `opencode:Ariadne`):
//!   seq_dec ‖ 0x1F ‖ tick_dec ‖ 0x1F ‖ type ‖ 0x1F ‖ payload_json_canonico ‖ 0x1F ‖ prev_hash
//! La SPEC §2 scrive "SHA-256(seq ‖ tick ‖ type ‖ payload ‖ prev_hash)" senza specificare
//! l'encoding della concatenazione; 0x1F non compare in nessun campo (decimali, A-Z,
//! payload canonico che escapa i controlli, esadecimale minuscolo), quindi il preimage
//! è univoco e un tampering di seq/tick non produce lo stesso hash.

use sha2::{Digest, Sha256};
use std::fmt;

use crate::payload::Payload;

/// prev_hash dell'evento genesi: 64 zeri.
pub const GENESIS_PREV_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// byte separatore del preimage (ASCII Unit Separator).
const SEP: u8 = 0x1F;

/// Domain dei tipi evento: SENSE | ACTION | STATE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventType {
    Sense,
    Action,
    State,
}

impl EventType {
    /// Rappresentazione testuale canonica: sempre maiuscolo, come dalla SPEC.
    pub fn as_str(self) -> &'static str {
        match self {
            EventType::Sense => "SENSE",
            EventType::Action => "ACTION",
            EventType::State => "STATE",
        }
    }

    /// Parse rigoroso: solo "SENSE"|"ACTION"|"STATE".
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "SENSE" => Some(EventType::Sense),
            "ACTION" => Some(EventType::Action),
            "STATE" => Some(EventType::State),
            _ => None,
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Evento L0. I campi sono pubblici in lettura; la costruzione avviene
/// solo attraverso `Event::seal` (crate-internal) — append-only.
#[derive(Clone, Debug)]
pub struct Event {
    pub seq: u64,
    pub tick: u64,
    pub r#type: EventType,
    pub payload: Payload,
    pub prev_hash: String,
    pub hash: String,
}

impl Event {
    /// Sigilla l'evento: calcola l'hash dal preimage canonico.
    /// `prev_hash` deve essere l'esadecimale a 64 caratteri dell'evento precedente
    /// (`GENESIS_PREV_HASH` per la genesi); non viene duplicato il controllo qui —
    /// l'aggancio è verificato da `Log::verify`.
    pub(crate) fn seal(seq: u64, tick: u64, r#type: EventType, payload: Payload, prev_hash: &str) -> Event {
        let hash = compute_hash(seq, tick, r#type, &payload.canonical(), prev_hash);
        Event {
            seq,
            tick,
            r#type,
            payload,
            prev_hash: prev_hash.to_string(),
            hash,
        }
    }
}

/// SHA-256 del preimage dell'evento, esadecimale minuscolo.
pub fn compute_hash(
    seq: u64,
    tick: u64,
    event_type: EventType,
    payload_canonical: &str,
    prev_hash: &str,
) -> String {
    let mut preimage = Vec::new();
    preimage.extend_from_slice(seq.to_string().as_bytes());
    preimage.push(SEP);
    preimage.extend_from_slice(tick.to_string().as_bytes());
    preimage.push(SEP);
    preimage.extend_from_slice(event_type.as_str().as_bytes());
    preimage.push(SEP);
    preimage.extend_from_slice(payload_canonical.as_bytes());
    preimage.push(SEP);
    preimage.extend_from_slice(prev_hash.as_bytes());
    to_hex(&Sha256::digest(&preimage))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_matches_documented_preimage() {
        // preimage indipendente, costruito dal test, contro il golden ricalcolato
        // esternamente con `shasum -a 256` (comando e output verbatim nel
        // messaggio del commit N4); mai ricalcolato con compute_hash o to_jsonl.
        // Formato canonico attuale (B2): `state` è la STRINGA decimale dello
        // stato interno PCG64, non l'intero del payload (forma pre-B2 eliminata).
        let payload = r#"{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"}"#;
        let mut expected_pre = Vec::new();
        expected_pre.extend_from_slice(b"0");
        expected_pre.push(0x1F);
        expected_pre.extend_from_slice(b"0");
        expected_pre.push(0x1F);
        expected_pre.extend_from_slice(b"STATE");
        expected_pre.push(0x1F);
        expected_pre.extend_from_slice(payload.as_bytes());
        expected_pre.push(0x1F);
        expected_pre.extend_from_slice(GENESIS_PREV_HASH.as_bytes());
        let golden = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";
        assert_eq!(to_hex(&Sha256::digest(&expected_pre)), golden);
        assert_eq!(
            compute_hash(0, 0, EventType::State, payload, GENESIS_PREV_HASH),
            golden
        );
    }

    #[test]
    fn event_type_parse_is_total_and_strict() {
        assert_eq!(EventType::parse("SENSE"), Some(EventType::Sense));
        assert_eq!(EventType::parse("ACTION"), Some(EventType::Action));
        assert_eq!(EventType::parse("STATE"), Some(EventType::State));
        assert_eq!(EventType::parse("state"), None);
        assert_eq!(EventType::parse("SEE"), None);
        assert_eq!(EventType::State.as_str(), "STATE");
    }
}
