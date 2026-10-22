//! L0 C1 — percorso `Err` di `Log::genesis` fallibile (SCOPE 2 del dispatch C1).
//!
//! Questo file è il commit TEST-ONLY: se resta solo qui la suite è ROSSA
//! perché `Log::genesis` è ancora infallibile e fa `seed as i64` silenzioso.
//! Nessuna funzione del crate esiste ancora che ritorni un `Result` sulla
//! genesi: questo file NON COMPARA (`.expect`/pattern `Err` su un `Log`) —
//! rosso provato per compile-fail, come precedentemente in 6d6eadc.
//!
//! Contratto testato (una volta implementato):
//! - `Log::genesis(i64::MAX as u64)` → `Ok`: il seed di confine resta
//!   rappresentabile come intero `i64` nel payload (contatore, non entropia);
//! - `Log::genesis(i64::MAX as u64 + 1)` → `Err`, NESSUN wrap: prima era
//!   `seed as i64` silenzioso (riproduzione nell'archivio memoria del progetto:
//!   "seed":-9223372036854775808 per --seed 9223372036854775808), ora rifiuto
//!   esplicito con messaggio che manda il fault al seed e al limite `i64`;
//!   dal percorso `Err` non esce alcun `Log`: nessun evento creato;
//! - il golden della genesi (seed 1042, payload invariato, hash
//!   `4649d1f4…1973f28e593526b7`) NON cambia: coperto già da
//!   `tests/l0_event_log.rs::test_genesis_golden_hash_constant`, che questa
//!   task NON tocca.

use ariadne::{Log, Value};

const SEED_I64_MAX: u64 = i64::MAX as u64;

/// Seed al confine `i64::MAX`: rappresentabile senza perdita.
#[test]
fn test_genesis_accepts_seed_i64_max() {
    let log = Log::genesis(SEED_I64_MAX).expect("seed == i64::MAX è rappresentabile");
    assert_eq!(log.events().len(), 1, "la genesi è il solo evento");
    log.verify().expect("la genesi al confine è verificabile");
    // round-trip del campo seed: i64 nel payload, nessun wrap
    assert_eq!(
        log.events()[0].payload.get("seed"),
        Some(&Value::Int(i64::MAX)),
        "il seed al confine entra nel payload ESATTAMENTE com'è"
    );
    let payload = log.events()[0].payload.canonical();
    assert!(
        payload.contains(r#""seed":9223372036854775807,"#),
        "seed al confine in forma canonica: {payload}"
    );
}

/// Seed oltre `i64::MAX` (2^63): rifiuto ESPPLICITO, non wrap silenzioso.
#[test]
fn test_genesis_rejects_seed_beyond_i64_max_no_event_no_wrap() {
    let oltre = SEED_I64_MAX.checked_add(1).expect("2^63 è rappresentabile in u64");
    let err = match Log::genesis(oltre) {
        Ok(_) => panic!("seed 2^63 accettato: era esattamente il wrap silenzioso da vietare"),
        Err(e) => e,
    };
    // messaggio esplicito: manda il fault al seed, riferisce il limite di dominio
    let msg = format!("{err}");
    assert!(msg.contains("seed"), "il messaggio manda il fault al seed: {msg}");
    assert!(msg.contains("i64"), "il messaggio riferisce il limite i64: {msg}");
    // "nessun evento creato": il percorso Err non produce alcun Log, quindi
    // nessun evento esce da genesis — la prova è qui a monte, nel match sopra.
}
