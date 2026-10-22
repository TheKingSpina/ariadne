//! Binario diagnostico `l0_replay` — dispatch B5, SPEC L0 v0.2 §5.
//!
//! # Scopo
//!
//! Esporre il log L0 all'esterno con i suoi byte canonici, perché due
//! esecuzioni INDIPENDENTI possano essere confrontate: `cargo run --bin l0_replay
//! -- --seed <u64> --ticks <u64>` scrive su stdout ESATTAMENTE
//! `Log::to_jsonl()` — niente banner, nessun newline extra, nessun path, nessun
//! tempo. Due processi separati con gli stessi argomenti devono emettere gli
//! stessi byte: questa è la voce §5 "replay bit-perfetto su due esecuzioni
//! indipendenti", oggi verificabile anche FUORI dai test (l'unico test
//! precedente, `tests/l0_cycle.rs:142`, costruisce due log nello stesso
//! processo; `tests/l0_replay_process.rs` ora lancia il binario due volte).
//!
//! # Eccezione documentata al divieto di I/O esterna
//!
//! L0 vieta I/O esterna NEL SUBSTRATO, cioè nella libreria (`lib`): zero
//! wall-clock, zero file, zero reti, zero casualità fuori dal PRNG di stato.
//! DIVIETO CHE VALE PER `lib`, NON PER QUESTO HARNESS: questo binario è uno
//! strumento diagnostico, NON parte del substrato, e il suo unico scopo è
//! leggere stdin-less gli argomenti e scrivere su stdout/stderr — proprio
//! perché due processi possano essere confrontati byte per byte. La libreria
//! non cambia: `Log::genesis`, `cycle::run_cycles` e `to_jsonl` restano senza
//! alcun contatto con l'ambiente, e anche qui l'unico input dichiarato sono i
//! due argomenti `--seed` e `--ticks` (nessun tempo, nessun file, nessuna rete,
//! nessuna casualità al di fuori del PRNG seminato dall'argomento).
//!
//! # Contratto
//!
//! - Argomenti esatti: `--seed <u64> --ticks <u64>` (flag in ordine libero,
//!   nessun duplicato, nient'altro).
//! - Successo: exit code 0, su stdout `Log::genesis(seed)` + `ticks` cicli
//!   serializzati in JSONL canonico, `'\n'` finale già incluso da `to_jsonl`:
//!   scritto byte-per-byte, senza aggiunge né lacune.
//! - Argomento mancante, non numerico, negativo, oltre u64 o superfluo:
//!   messaggio su stderr, exit code 1, stdout VUOTO (nessun output parziale).

use std::process::ExitCode;

use ariadne::prng::Pcg64;
use ariadne::{cycle, Log};

/// `Ok(())` → stdout; `Err(messaggio)` → stderr + exit 1. Nessun flusso terzo.
fn main() -> ExitCode {
    match run() {
        Ok((seed, ticks)) => {
            let mut log = Log::genesis(seed);
            let mut rng = Pcg64::new(seed);
            match cycle::run_cycles(&mut log, &mut rng, ticks) {
                Ok(()) => {
                    // Solo i byte del log: `print!` NON aggiunge il newline
                    // che `to_jsonl` termina già da sé.
                    print!("{}", log.to_jsonl());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("l0_replay: esecuzione del ciclo non riuscita: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(messaggio) => {
            eprintln!("{messaggio}");
            ExitCode::FAILURE
        }
    }
}

/// Parse degli argomenti e costruzione del log: `Ok((seed, ticks))` sul
/// successo, `Err(messaggio)` per ogni rifiuto (che NON tocca stdout).
fn run() -> Result<(u64, u64), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut seed: Option<u64> = None;
    let mut ticks: Option<u64> = None;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" | "--ticks" => {
                let flag = args[i].as_str();
                let valore = args.get(i + 1).ok_or_else(|| {
                    format!("l0_replay: manca il valore dopo '{flag}' (uso: --seed <u64> --ticks <u64>)")
                })?;
                let numero = parse_u64_strict(valore, flag)?;
                let slot = if flag == "--seed" { &mut seed } else { &mut ticks };
                if slot.is_some() {
                    return Err(format!("l0_replay: '{flag}' duplicato (uso: --seed <u64> --ticks <u64>)"));
                }
                *slot = Some(numero);
                i += 2;
            }
            altro => {
                return Err(format!(
                    "l0_replay: argomento inatteso '{altro}' (uso: --seed <u64> --ticks <u64>)"
                ));
            }
        }
    }

    let seed = seed
        .ok_or_else(|| "l0_replay: manca --seed <u64> (uso: --seed <u64> --ticks <u64>)".to_string())?;
    let ticks = ticks
        .ok_or_else(|| "l0_replay: manca --ticks <u64> (uso: --seed <u64> --ticks <u64>)".to_string())?;
    Ok((seed, ticks))
}

/// Parse u64 rigoroso per la CLI: solo cifre decimali (niente segno, niente
/// spazi, niente overflow): la stessa forma "cifre, zero in testa vietato"
/// del payload canonico per non introdurre una seconda convenzione.
fn parse_u64_strict(s: &str, flag: &str) -> Result<u64, String> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!(
            "l0_replay: {flag} richiede un intero u64 decimale, trovato '{s}'"
        ));
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(format!("l0_replay: {flag} senza zeri in testa, trovato '{s}'"));
    }
    s.parse::<u64>()
        .map_err(|_| format!("l0_replay: {flag} oltre u64::MAX, trovato '{s}'"))
}
