//! Ciclo senso→azione→stato — SPEC L0 v0.2, sezione "Ciclo senso→azione→stato
//! (un tick)" + riga 44 dello Schema PRNG.
//!
//! # Semantica del tick (forme esatte, decise in governance `opencode:Ariadne`)
//!
//! - La genesi è `tick 0`. Un ciclo emette ESATTAMENTE 3 eventi — SENSE, ACTION,
//!   STATE — tutti con lo stesso `tick` = ultimo tick del log + 1.
//! - Il tick avanza SOLO a ciclo chiuso: dopo l'append dello STATE. Nessuna
//!   variabile di avanzamento separata dal log: il tick di apertura del
//!   PROSSIMO ciclo è calcolato leggendo la coda (ultimo `tick` + 1), quindi
//!   l'avanzamento avviene materialmente quando lo STATE è in coda. Mai cicli
//!   parziali, mai due SENSE consecutivi senza ACTION: il ciclo è atomico per
//!   costruzione — tre append consecutivi in ordine fisso, su un log che è
//!   append-only (nessun riordino possibile).
//! - L'evento STATE di tick NON contiene `head`: l'hash di testa è già il
//!   `prev_hash` dell'evento stesso e un `head` nel payload sarebbe circolare
//!   rispetto al proprio hash.
//!
//! # Budget draw (riga 44: "prima i SENSE, poi eventuali draw della policy,
//! budget draw per tick esplicito — niente dipendenze nascoste")
//!
//! - **1 draw per tick**, consumato dal SENSE: il "mondo" di L0 è uno stub
//!   deterministico, il senso è estratto dal PRNG e scritto come evento.
//! - La policy consume **0 draw**: è funzione deterministica dello stimolo.
//! - La genesi consuma **0 draw** (il suo `state` è lo stato iniziale).
//!
//! # Forme esatte dei payload (decise, non interpretabili)
//!
//! ```text
//! SENSE  → {"stimolo": "<draw u64 in decimale, stringa>"}
//! ACTION → {"mossa": <int 0..2>}                    = parse(stimolo) % 3
//! STATE  → {"action": <int>, "sense": <int>, "state": "<snapshot PCG64 128
//!           bit decimale>", "tick": <int>}
//! ```
//!
//! - `stimolo` è la stringa decimale del draw COMPLETO: un draw PCG64 è `u64`,
//!   il payload canonico ammette solo `i64`, e mascherare perderebbe entropia.
//!   La policy fa il parse (su `u64`, mai su `i64` mascherato).
//! - La chiave dello snapshot è **`state`**, NON `prng`: `prng` nella genesi è
//!   il nome dell'algoritmo, e la fold di [`crate::state`] legge `state` —
//!   nessuna seconda convenzione.
//! - I contatori `sense` / `action` nell'evento STATE sono il numero di eventi
//!   SENSE / ACTION accumulati nel log al chiuso del ciclo (dopo N cicli: N ed
//!   N): valori interi in scala `i64` della forma canonica. NON sono il valore
//!   dello stimolo (un draw `u64` può sforare `i64::MAX`, per questo `stimolo`
//!   è una stringa).
//!
//! # Policy
//!
//! `mossa = parse(stimolo) % 3` ([`mossa_from_stimolo`]): zero rete, zero draw,
//! zero stato — unica funzione deterministica dello stimolo. Un ACTION per
//! ciclo, mai zero mai due: è il secondo evento del ciclo.
//!
//! # Isolamento
//!
//! Input dichiarati e usati: SOLO [`Log`] (append e lettura della coda) e
//! [`Pcg64`] (draw e snapshot). Zero wall-clock, zero I/O, zero float, zero
//! casualità fuori dal PRNG di stato, zero dipendenze nuove: la forma canonica
//! del payload è `Payload`/`Value` esistenti e l'errore riusa [`Error`] del log
//! (le uniche fallibilità reali sono quelle di append e cast espliciti).

use crate::event::EventType;
use crate::log::{Error, Log};
use crate::payload::{Payload, Value};
use crate::prng::Pcg64;

/// Policy del ciclo (deterministica, zero draw): `parse(stimolo) % 3`.
///
/// Il parse è su `u64` a entropia COMPLETA: lo stimolo è la stringa decimale
/// del draw intero e un parse su `i64` mascherebbe i draw oltre `i64::MAX`
/// (all'origine stessa della scelta della stringa). La funzione accetta la
/// stringa e la interpreta: fuori formato → `None` (la policy non inventa
/// valori).
pub fn mossa_from_stimolo(stimolo_dec: &str) -> Option<i64> {
    stimolo_dec
        .parse::<u64>()
        .ok()
        .map(|valore| (valore % 3) as i64)
}

/// Un ciclo completo: SENSE → ACTION → STATE, con lo stesso tick = ultimo
/// tick del log + 1. Atomico per costruzione: o i tre eventi escono in ordine
/// o non esce nulla.
///
/// Budget draw: questo tick consuma ESATTAMENTE 1 draw, saldato al SENSE
/// (policy: 0; genesi: 0). Vedi mod-doc.
pub fn run_cycle(log: &mut Log, rng: &mut Pcg64) -> Result<(), Error> {
    // Il tick di apertura del ciclo: ultimo tick del log + 1. La lettura della
    // coda è l'UNICO input dopo il seed: nessun wall-clock, nessun contatore
    // esterno che possa divergere dal log.
    let ultimo_tick = log
        .events()
        .last()
        .ok_or_else(|| Error::Chain("ciclo su log vuoto: manca la genesi".to_string()))?
        .tick;
    let tick = ultimo_tick
        .checked_add(1)
        .ok_or_else(|| Error::Chain("tick oltre u64::MAX".to_string()))?;

    // 1) SENSE — 1 draw per tick, stringa decimale del draw intero.
    let draw = rng.next_u64();
    log.append(
        tick,
        EventType::Sense,
        Payload::from_pairs(vec![("stimolo", Value::Str(draw.to_string()))]),
    )?;

    // 2) ACTION — policy: parse(stimolo) % 3, ZERO draw. Il parse rilegge il
    //    payload emesso dal SENSE: lo stimolo è l'input ufficiale della policy.
    let stimolo = ultimo_sense_stimolo(log)?;
    let mossa = mossa_from_stimolo(&stimolo).ok_or_else(|| {
        // Irraggiungibile per costruzione: il SENSE emette solo decimali u64.
        Error::Chain(format!("stimolo non interpretabile dalla policy: {stimolo:?}"))
    })?;
    log.append(
        tick,
        EventType::Action,
        Payload::from_pairs(vec![("mossa", Value::Int(mossa))]),
    )?;

    // 3) STATE — snapshot PCG64 (chiave `state`), contatori accumulati, tick.
    //    Zero draw: lo snapshot è la LETTURA dello stato interno corrente.
    let (sense, action) = conteggi_al_chiuso(log)?;
    log.append(
        tick,
        EventType::State,
        Payload::from_pairs(vec![
            ("action", Value::Int(action)),
            ("sense", Value::Int(sense)),
            ("state", Value::Str(rng.state_decimal())),
            ("tick", Value::Int(i64_tick(tick)?)),
        ]),
    )?;

    // Ciclo chiuso: il tick avanza per costruzione — il PROSSIMO ciclo legge
    // QUESTO STATE in coda e apre a tick + 1. Mai cicli parziali.
    Ok(())
}

/// N cicli completi e consecutivi: mai cicli parziali.
pub fn run_cycles(log: &mut Log, rng: &mut Pcg64, n: u64) -> Result<(), Error> {
    for _ in 0..n {
        run_cycle(log, rng)?;
    }
    Ok(())
}

/// stimolo del SENSE in coda (l'input ufficiale della policy): lettura dal
/// log, mai uno stato parallelo che possa divergere.
fn ultimo_sense_stimolo(log: &Log) -> Result<String, Error> {
    let last = log
        .events()
        .last()
        .ok_or_else(|| Error::Chain("log vuoto a metà ciclo".to_string()))?;
    match (last.r#type, last.payload.get("stimolo")) {
        (EventType::Sense, Some(Value::Str(s))) => Ok(s.clone()),
        _ => Err(Error::Chain(
            "l'evento in coda non è un SENSE con `stimolo` stringa: mai cicli parziali"
                .to_string(),
        )),
    }
}

/// Conteggi di SENSE/ACTION accumulati al chiuso del ciclo: la stessa semantica
/// che la fold di `state.rs` produce contando gli eventi — una sola verità,
/// calcolata in unica passata sul log al chiuso del ciclo.
fn conteggi_al_chiuso(log: &Log) -> Result<(i64, i64), Error> {
    let mut sense: u64 = 0;
    let mut action: u64 = 0;
    for e in log.events() {
        match e.r#type {
            EventType::Sense => sense = sense.saturating_add(1),
            EventType::Action => action = action.saturating_add(1),
            EventType::State => {}
        }
    }
    let sense = i64::try_from(sense)
        .map_err(|_| Error::Chain("conteggio SENSE oltre i64".to_string()))?;
    let action = i64::try_from(action)
        .map_err(|_| Error::Chain("conteggio ACTION oltre i64".to_string()))?;
    Ok((sense, action))
}

/// tick dell'evento in scala `i64` della forma canonica: cast unico, esplicito,
/// mai un `as` silenzioso.
fn i64_tick(tick: u64) -> Result<i64, Error> {
    i64::try_from(tick).map_err(|_| Error::Chain("tick oltre la scala i64 del payload".to_string()))
}
