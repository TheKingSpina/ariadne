//! Ariadne L0 — substrato: log append-only aciclico.
//!
//! Contenuto di questo crate (scope del dispatch: SPEC L0 v0.2 §2 e §3 + genesi §4):
//! - schema evento {seq, tick, type, payload, prev_hash, hash} ([`Event`], [`EventType`]);
//! - catena di hash: `hash = SHA-256(seq ‖ tick ‖ type ‖ payload ‖ prev_hash)`
//!   con preimage separato da 0x1F (encoding documentato in [`event`] e fissato dai test);
//! - genesi con seed: [`Log::genesis`], payload {seed, prng: "PCG64", state esplicito};
//! - append-only e correzioni compensative: [`Log::append`], [`Log::compensate`], [`Log::verify`];
//! - log canonico JSONL: [`Log::to_jsonl`], [`Log::from_jsonl`];
//! - ciclo senso→azione→stato (dispatch B4, sezione "Ciclo" della SPEC + riga 44
//!   dello Schema PRNG): [`cycle::run_cycle`] / [`cycle::run_cycles`], 3 eventi
//!   per tick, budget draw esplicito (1 dal SENSE, 0 dalla policy, 0 dalla genesi).
//!
//! NON è in scope qui (dispatch esplicito): policy oltre la stub `parse % 3`,
//! livelli L1–L6.
//!
//! Determinismo: zero wall-clock, zero I/O esterna, zero reti, zero float.
//! `tick` è un contatore logico di ciclo, mai un timestamp.

pub mod cycle;
mod event;
pub mod identity;
mod log;
pub mod memory;
mod payload;
pub mod prng;
pub mod selection;
pub mod social;
pub mod state;
pub mod valuation;
pub mod view;

pub use event::{compute_hash, Event, EventType, GENESIS_PREV_HASH};
pub use log::{Error, Log};
pub use payload::{Payload, Value};
pub use state::{State, StateError};
