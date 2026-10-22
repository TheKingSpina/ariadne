//! Fold dello State — SPEC L0 v0.2 §4 ("Stato = fold del log applicato in
//! ordine di seq", "stesso log → stesso stato, byte per byte") + riga 43 dello
//! Schema PRNG ("ogni STATE snapshotta il suo stato, così il replay verifica
//! per confronto invece di fidarsi della riesecuzione").
//!
//! # Contratto
//!
//! - [`State::fold`] applica gli eventi in ordine di `seq` e non ricalcola MAI
//!   il PRNG: lo stato PCG64 viene **letto** dallo snapshot (chiave `state`,
//!   stringa decimale) dell'ultimo STATE che ne porta uno. Nessuna chiamata a
//!   `Pcg64` qui dentro: se la fold rieseguisse i draw, il replay sarebbe un
//!   confronto fra due esecuzioni identiche per costruzione, cioè inutile.
//! - `tick` = ultimo tick visto (tick dell'ultimo evento in ordine di `seq`);
//!   `sense` / `action` = numero di eventi SENSE / ACTION applicati.
//! - `head` = hash dell'ultimo evento applicato (dopo la sola genesi è
//!   l'hash della genesi, poi avanza con essa).
//! - **Precondizione: la fold assume un log VERIFICATO** (`Log::verify()` a
//!   `Ok` prima della chiamata). Il codice NON ricalcola nessun hash di
//!   evento — è ciò che la logica di replay di §4 esige — quindi un tampering
//!   dei payload è INVISIBILE alla fold da sola: solo `verify()` lo coglie.
//!   Come guardia d'integrità (uso improprio reso rumoroso, non sostituzione
//!   della verifica) la fold rifiuta comunque una catena che non si aggancia
//!   strutturalmente: seq non da 0 senza gap (posizione = seq) o `prev_hash`
//!   memorizzato diverso dall'hash memorizzato dell'evento precedente, con
//!   [`StateError::Chain`].
//! - Evento compensativo (§3 + opzione A, dispatch C1): è un STATE come gli
//!   altri — AUTODESCRITTIVO, porta lo snapshot `state` dell'ultimo STATE
//!   precedente (copia verbatim) e NON avanza il PRNG. Ogni STATE del log
//!   porta `state`: la fold non ricorda più un «ultimo visto», lo snapshot è
//!   aggiornato da ciò che l'evento PORTE — un STATE senza `state` stringa è
//!   un errore rumoroso ([`StateError::Snapshot`]); nessuna conciliazione
//!   silenziosa.
//! - La fold è PURA: nessuna mutazione del log, nessun I/O, nessun
//!   wall-clock; stesso log → stesso stato, byte per byte.
//!
//! # Schema (proposta del dispatch B3, con una correzione dichiarata)
//!
//! `State { tick, sense, action, prng, head }` come richiesto; i contatori
//! sono `i64` invece di `u64`: la forma canonica riusa `Payload`, i cui valori
//! interi sono `i64`, e il cast da `u64` avviene UNA volta, esplicito e
//! sicuro, in [`State::fold`] (`i64::try_from`, mai un `as`); dimensioni
//! superiori a `i64::MAX` sono impossibili per contatori tratti da un `Vec`
//! in memoria, ma il cast non è fiducioso: fuori scala → [`StateError::Overflow`].

use std::fmt;

use crate::event::{Event, EventType, GENESIS_PREV_HASH};
use crate::log::Log;
use crate::payload::{Payload, Value};

/// Errore della fold. Il caso `Chain` è la GUARDIA STRUTTURALE documentata nel
/// modulo: non ricalcola hash (quella verifica completa è solo `Log::verify`).
#[derive(Clone, Debug, PartialEq)]
pub enum StateError {
    /// Catena non agganciata strutturalmente: seq non da 0 senza gap, o
    /// `prev_hash` memorizzato diverso dall'hash memorizzato dell'evento
    /// precedente. Un `Log::verify()` preventivo l'avrebbe comunque rifiutata.
    Chain(String),
    /// Genesi (seq 0, STATE) senza `state` stringa nel payload: nessuno stato
    /// iniziale del PRNG è leggibile (la SPEC §4 fissa lo snapshot nello
    /// payload della genesi).
    Genesis(String),
    /// Evento STATE (non genesi, o comunque in coda dopo la genesi) senza
    /// `state` stringa nel payload: viola l'invariante «ogni STATE è
    /// autodescrittivo» (opzione A, dispatch C1). Errore rumoroso, nessuna
    /// conciliazione con un «ultimo visto»: niente stato è derivabile da uno
    /// snapshot assente.
    Snapshot(String),
    /// Contatore fuori dalla scala `i64` della forma canonica: cast esplicito
    /// rifiutato, mai un `as` silenzioso. Irrealistico (contatori tratti da un
    /// `Vec` in memoria), tuttavia non fiducioso.
    Overflow(String),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::Chain(m) => write!(f, "fold: catena non agganciata: {m}"),
            StateError::Genesis(m) => write!(f, "fold: genesi senza snapshot: {m}"),
            StateError::Snapshot(m) => write!(f, "fold: STATE senza snapshot: {m}"),
            StateError::Overflow(m) => write!(f, "fold: contatore oltre i64: {m}"),
        }
    }
}

impl std::error::Error for StateError {}

/// Stato del mondo a riposo, data la storia: la fold del log verificato.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Ultimo tick visto nel log (tick dell'ultimo evento in ordine di `seq`).
    pub tick: i64,
    /// Numero di eventi SENSE applicati.
    pub sense: i64,
    /// Numero di eventi ACTION applicati.
    pub action: i64,
    /// Stato interno PCG64 a 128 bit, stringa decimale, LETTO dallo snapshot
    /// dell'ultimo STATE che ne porta uno — mai ricalcolato dai draw.
    pub prng: String,
    /// Hash dell'ultimo evento applicato.
    pub head: String,
}

impl State {
    /// Fold pura del log (SPEC §4). Precondizione: log verificato (vedi
    /// mod-doc); la guardia strutturale rende l'uso improprio rumoroso.
    pub fn fold(log: &Log) -> Result<State, StateError> {
        let events = log.events();

        // Guardia strutturale: log mai vuoto, seq = posizione (da 0, senza
        // gap), prev_hash agganciato all'hash memorizzato precedente. Nessun
        // ricalcolo di hash: se i riferimenti memorizzati non si agganciano
        // fra loro, la fold rifiuta — nessuno stato è derivabile da una
        // catena non agganciata.
        let mut prev_hash: &str = GENESIS_PREV_HASH;
        for (i, e) in events.iter().enumerate() {
            if e.seq != i as u64 {
                return Err(StateError::Chain(format!(
                    "seq non da 0 senza gap: posizione {i}, seq {}",
                    e.seq
                )));
            }
            if e.prev_hash != prev_hash {
                return Err(StateError::Chain(format!(
                    "prev_hash disallineato a seq {}: atteso {prev_hash}, registrato {}",
                    e.seq, e.prev_hash
                )));
            }
            prev_hash = &e.hash;
        }

        // La genesi è l'evento di partenza e porta lo snapshot iniziale (§4).
        let genesis = &events[0];
        let mut prng = match genesis.payload.get("state") {
            Some(Value::Str(s)) => s.clone(),
            _ => {
                return Err(StateError::Genesis(format!(
                    "genesi seq 0 senza `state` stringa: payload {}",
                    genesis.payload.canonical()
                )))
            }
        };
        let mut head = genesis.hash.clone();
        let mut tick = tick_of(genesis)?;
        let mut sense: u64 = 0;
        let mut action: u64 = 0;

        // APPEND-ONLY, in ordine di seq: il primo passo è la genesi stessa,
        // poi ogni evento porta il suo contributo allo stato.
        for e in events {
            match e.r#type {
                EventType::Sense => {
                    sense = sense
                        .checked_add(1)
                        .ok_or_else(|| StateError::Overflow("sense oltre u64".to_string()))?;
                }
                EventType::Action => {
                    action = action
                        .checked_add(1)
                        .ok_or_else(|| StateError::Overflow("action oltre u64".to_string()))?;
                }
                EventType::State => {
                    // Ogni STATE è autodescrittivo (opzione A, dispatch C1):
                    // PORTE lo snapshot e da esso `prng` si aggiorna — VERBATIM
                    // (la fold non valida né reinterpretà la stringa). Niente
                    // «ultimo visto»: uno STATE senza `state` stringa è un
                    // errore rumoroso, non un silenzio da colmare col ricordo.
                    prng = match e.payload.get("state") {
                        Some(Value::Str(s)) => s.clone(),
                        _ => {
                            return Err(StateError::Snapshot(format!(
                                "evento STATE a seq {} senza `state` stringa nel payload: {}",
                                e.seq,
                                e.payload.canonical()
                            )))
                        }
                    };
                }
            }
            head = e.hash.clone();
            tick = tick_of(e)?;
        }

        Ok(State {
            tick,
            sense: counter(sense, "sense")?,
            action: counter(action, "action")?,
            prng,
            head,
        })
    }

    /// Forma canonica: stesso schema dei payload di evento (JSON canonico via
    /// `Payload`), chiavi in ordine alfabetico — action, head, prng, sense, tick;
    /// le due stringhe come `Value::Str`, i contatori come `Value::Int`.
    pub fn to_payload(&self) -> Payload {
        Payload::from_pairs(vec![
            ("action", Value::Int(self.action)),
            ("head", Value::Str(self.head.clone())),
            ("prng", Value::Str(self.prng.clone())),
            ("sense", Value::Int(self.sense)),
            ("tick", Value::Int(self.tick)),
        ])
    }

    /// Stato canonico come stringa JSON: stessa stringa, byte per byte, a ogni
    /// chiamata e a ogni run (la fold è pura e il formato è fissato).
    pub fn canonical(&self) -> String {
        self.to_payload().canonical()
    }
}

/// tick dell'evento in scala `i64` della forma canonica: cast unico, esplicito.
fn tick_of(e: &Event) -> Result<i64, StateError> {
    i64::try_from(e.tick)
        .map_err(|_| StateError::Overflow(format!("tick {} oltre i64 a seq {}", e.tick, e.seq)))
}

fn counter(n: u64, name: &str) -> Result<i64, StateError> {
    i64::try_from(n).map_err(|_| StateError::Overflow(format!("{name} oltre i64")))
}
