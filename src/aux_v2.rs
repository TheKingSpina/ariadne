//! Versioned AUX implementation, separate from the signed historical v1 API.
//!
//! `V2Log` implements the coordinated profile documented in
//! `AUX_V2_PROFILO_IMPLEMENTATO.md`, including semantic replay. `AuxSuffix`
//! below is retained as the earlier, limited prototype; it is not a protocol
//! reader or a replacement for `V2Log`. No v1 event type or reader is changed.

use sha2::{Digest, Sha256};

use crate::{EventType, Log, Payload, Value};

mod json;
mod protocol;
pub use json::Json;
pub use protocol::{chain_break, Error, V2Log, Record, Node, MemoryStatus, SocialModel, Fixture};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuxError {
    InvalidPrefix,
    InvalidConfig,
    WindowState,
    SequenceOverflow,
}

/// A sealed AUX record. Construction is restricted to the operations below.
#[derive(Debug, Clone, PartialEq)]
pub struct AuxRecord {
    pub seq: u64,
    pub tick: u64,
    pub payload: Payload,
    pub prev_hash: String,
    pub hash: String,
}

/// Experimental suffix attached to an immutable, verified v1 prefix.
/// This is not a replacement for the v1 `Log` and cannot append L0 cycles.
#[derive(Debug, Clone)]
pub struct AuxSuffix {
    prefix: Log,
    records: Vec<AuxRecord>,
    configured: bool,
    window_open: bool,
}

impl AuxSuffix {
    /// Accept a genesis followed only by complete SENSE/ACTION/STATE cycles.
    /// Chain validation is performed, but payload and PRNG semantics are not.
    pub fn new(prefix: Log) -> Result<Self, AuxError> {
        prefix.verify().map_err(|_| AuxError::InvalidPrefix)?;
        let events = prefix.events();
        let genesis = events.first().ok_or(AuxError::InvalidPrefix)?;
        if genesis.seq != 0 || genesis.tick != 0 || genesis.r#type != EventType::State {
            return Err(AuxError::InvalidPrefix);
        }
        let tail = &events[1..];
        if tail.len() % 3 != 0 {
            return Err(AuxError::InvalidPrefix);
        }
        for (index, cycle) in tail.chunks_exact(3).enumerate() {
            let tick = u64::try_from(index).map_err(|_| AuxError::InvalidPrefix)?
                .checked_add(1).ok_or(AuxError::InvalidPrefix)?;
            if cycle[0].r#type != EventType::Sense
                || cycle[1].r#type != EventType::Action
                || cycle[2].r#type != EventType::State
                || cycle.iter().any(|event| event.tick != tick)
            {
                return Err(AuxError::InvalidPrefix);
            }
        }
        Ok(Self { prefix, records: Vec::new(), configured: false, window_open: false })
    }

    pub fn records(&self) -> &[AuxRecord] {
        &self.records
    }

    /// Append one L2 config when no window is open.
    pub fn append_config(&mut self, k: i64, tau: i64) -> Result<(), AuxError> {
        if self.window_open {
            return Err(AuxError::WindowState);
        }
        if k < 0 || tau < 1 {
            return Err(AuxError::InvalidConfig);
        }
        let data = format!("{{\"k\":{k},\"tau\":{tau}}}");
        let event = self.prepare("config", &data, 0, None)?;
        self.records.push(event);
        self.configured = true;
        Ok(())
    }

    pub fn open_window(&mut self) -> Result<(), AuxError> {
        if !self.configured || self.window_open {
            return Err(AuxError::WindowState);
        }
        let event = self.prepare("barrier", "{\"phase\":\"open\"}", 0, None)?;
        self.records.push(event);
        self.window_open = true;
        Ok(())
    }

    /// Close a window that cannot contain selects in this restricted prototype.
    /// Both records are prepared before the suffix is mutated.
    pub fn close_empty_window(&mut self) -> Result<(), AuxError> {
        if !self.window_open {
            return Err(AuxError::WindowState);
        }
        let close = self.prepare("barrier", "{\"phase\":\"close\"}", 0, None)?;
        let summary = self.prepare("summary", "{\"expulsions\":[],\"text\":\"\"}", 1, Some(&close.hash))?;
        self.records.extend([close, summary]);
        self.window_open = false;
        Ok(())
    }

    fn prepare(&self, kind: &str, data: &str, offset: u64, previous: Option<&str>) -> Result<AuxRecord, AuxError> {
        let last = self.prefix.events().last().ok_or(AuxError::InvalidPrefix)?;
        let base_seq = self.records.last().map_or(last.seq, |record| record.seq);
        let seq = base_seq.checked_add(1).and_then(|n| n.checked_add(offset)).ok_or(AuxError::SequenceOverflow)?;
        let prev_hash = previous.unwrap_or_else(|| self.records.last().map_or(last.hash.as_str(), |r| r.hash.as_str())).to_owned();
        let payload = Payload::from_pairs([
            ("data", Value::Str(data.to_owned())),
            ("kind", Value::Str(kind.to_owned())),
            ("level", Value::Str("L2".to_owned())),
            ("protocol_version", Value::Str("2".to_owned())),
            ("provenance", Value::Str("{\"members\":[],\"refs\":[]}".to_owned())),
            ("sources", Value::Str("[]".to_owned())),
        ]);
        let preimage = format!("{seq}\x1f{}\x1fAUX\x1f{}\x1f{prev_hash}", last.tick, payload.canonical());
        let hash = Sha256::digest(preimage.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect();
        Ok(AuxRecord { seq, tick: last.tick, payload, prev_hash, hash })
    }
}
