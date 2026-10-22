//! Test del binario diagnostico `l0_replay` — dispatch B5, SPEC L0 v0.2 §5.
//!
//! Rendere verificabile la voce §5 "replay bit-perfetto su due esecuzioni
//! indipendenti". Il test più vicino in `tests/l0_cycle.rs:142` costruisce due
//! log NELLO STESSO PROCESSO: prova che una funzione è deterministica, non che
//! il sistema lo sia FRA PROCESSI. Qui il binario viene lanciato DUE VOLTE
//! come processi separati (due `std::process::Command` distinti, ciascuno con
//! i propri argomenti) e i corpi stdout sono confrontati byte per byte.
//!
//! Contratto sotto test:
//! - due esecuzioni separate con gli stessi argomenti → stdout byte-identico;
//! - l'output non è vuoto e finisce con UN SOLO '\n' (nessun banner, nessun
//!   newline extra: è esattamente `Log::to_jsonl()`);
//! - l'output è ricaricabile: `Log::from_jsonl` + `verify()` e riscrittura
//!   byte-identica (replay bit-perfetto sull'intero stdout);
//! - il numero di eventi STATE non-genesi è pari a `--ticks` e l'ultimo
//!   `tick` è `--ticks` (3 eventi per ciclo, tick avanza a ciclo chiuso);
//! - semi diversi → byte diversi; tick diversi → byte diversi;
//! - argomenti invalidi → exit code ≠ 0, stdout vuoto, messaggio su stderr.

use std::process::Command;

use ariadne::{EventType, Log};

/// Percorso del binario diagnostico, fornito da Cargo all'atto del test.
/// In primo commit questa costante NON compila: `CARGO_BIN_EXE_l0_replay`
/// non esiste perché il target `l0_replay` non c'è ancora — è il rosso
/// dichiarato dal dispatch.
const BIN: &str = env!("CARGO_BIN_EXE_l0_replay");

/// Lancio in un processo NUOVO e raccolgo l'output completo. Due chiamate a
/// questa funzione sono due processi distinti: nessuna variabile condivisa,
/// nessun stato in-process che possa mascherare differenze.
fn lancia(args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .args(args)
        .output()
        .expect("il binario diagnostico l0_replay parte")
}

// ---------------------------------------------------------------------------
// Determinismo fra processi
// ---------------------------------------------------------------------------

/// Due esecuzioni in due processi separati, stessi argomenti → i corpi stdout
/// sono byte-identici (SPEC §5: replay bit-perfetto su due esecuzioni
/// indipendenti).
#[test]
fn due_processi_stessi_argomenti_stdout_byte_identico() {
    let a = lancia(&["--seed", "1042", "--ticks", "5"]);
    let b = lancia(&["--seed", "1042", "--ticks", "5"]);
    assert!(
        a.status.success() && b.status.success(),
        "entrambi i processi escono con successo ({:?} / {:?})",
        a.status.code(),
        b.status.code()
    );
    assert!(!a.stdout.is_empty(), "il primo stdout non è vuoto");
    assert_eq!(
        a.stdout, b.stdout,
        "due processi indipendenti, stesso seed, stessi tick → stessi byte"
    );
}

/// L'output non è vuoto e finisce con UN SOLO newline: è esattamente il JSONL
/// canonico di `Log::to_jsonl()`, senza banner né newline extra.
#[test]
fn output_non_vuoto_finisce_con_un_solo_newline() {
    let out = lancia(&["--seed", "1042", "--ticks", "3"]);
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout.clone()).expect("stdout UTF-8");
    assert!(!s.is_empty(), "l'output non è vuoto");
    assert!(s.ends_with('\n'), "newline finale canonico");
    assert!(!s.ends_with("\n\n"), "un SOLO newline finale, nessun extra");
    assert!(!s.contains('\r'), "solo LF: nessun CR nel corpo");
}

/// L'output è ricaricabile: `Log::from_jsonl` + `verify()` sull'intero stdout,
/// e il log ricaricato riscrive ESATTAMENTE gli stessi byte (replay
/// bit-perfetto anche attraversando la serializzazione).
#[test]
fn output_ricaricabile_da_from_jsonl_e_verificato() {
    let out = lancia(&["--seed", "1042", "--ticks", "7"]);
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout.clone()).expect("stdout UTF-8");
    let replay = Log::from_jsonl(&s).expect("l'intero stdout è JSONL canonico");
    replay
        .verify()
        .expect("il log ricaricato dal stdout verifica la catena end-to-end");
    assert_eq!(
        replay.to_jsonl(),
        s,
        "il ricaricato riscrive gli stessi byte: replay bit-perfetto"
    );
}

// ---------------------------------------------------------------------------
// Struttura: 3 eventi per tick, STATE non-genesi = ticks, ultimo tick = ticks
// ---------------------------------------------------------------------------

/// Il numero di eventi STATE NON-genesi è pari a `--ticks` (3 eventi/ciclo:
/// SENSE, ACTION, STATE) e l'ultimo `tick` del log è `--ticks`.
#[test]
fn state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks() {
    let ticks: u64 = 5;
    let out = lancia(&["--seed", "1042", "--ticks", &ticks.to_string()]);
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout.clone()).expect("stdout UTF-8");
    let replay = Log::from_jsonl(&s).expect("JSONL canonico");

    let state_nongenesi = replay
        .events()
        .iter()
        .filter(|e| e.r#type == EventType::State && e.seq != 0)
        .count();
    assert_eq!(
        state_nongenesi as u64, ticks,
        "un evento STATE di ciclo per tick, la genesi non conta"
    );
    let ultimo = replay.events().last().expect("log non vuoto");
    assert_eq!(ultimo.tick, ticks, "l'ultimo tick è --ticks");
}

/// Semi diversi → byte diversi; tick diversi → byte diversi. E il log con
/// più tick ha i byte del log più corto come PREFISSO estratto-invariante
/// (stessa storia in prolongamento, nessuna riorganizzazione).
#[test]
fn argomenti_diversi_producono_byte_diversi() {
    let seme_a = lancia(&["--seed", "1042", "--ticks", "3"]);
    assert!(seme_a.status.success());
    let seme_b = lancia(&["--seed", "1043", "--ticks", "3"]);
    assert!(seme_b.status.success());
    assert_ne!(
        seme_a.stdout, seme_b.stdout,
        "semi diversi → storie diverse, byte per byte"
    );

    let tick_a = lancia(&["--seed", "1042", "--ticks", "3"]);
    assert!(tick_a.status.success());
    let tick_b = lancia(&["--seed", "1042", "--ticks", "4"]);
    assert!(tick_b.status.success());
    assert_ne!(tick_a.stdout, tick_b.stdout, "tick diversi → byte diversi");
    assert!(
        tick_b.stdout.starts_with(&tick_a.stdout),
        "il log più lungo prolunga quello più corto: stesso prefisso, niente riorganizzazione"
    );
}

// ---------------------------------------------------------------------------
// Argomenti invalidi
// ---------------------------------------------------------------------------

/// Caso di rifiuto: exit code ≠ 0 (anche per segnale), stdout VUOTO
/// (niente output parziale o spurio), messaggio diagnostico su stderr.
fn rifiuta(args: &[&str], caso: &str) {
    let out = lancia(args);
    let code = out.status.code();
    assert!(
        !out.status.success(),
        "{caso}: l'exit code non deve essere 0 (trovato {code:?})"
    );
    assert!(
        out.stdout.is_empty(),
        "{caso}: stdout vuoto su rifiuto, trovati {} byte",
        out.stdout.len()
    );
    assert!(!out.stderr.is_empty(), "{caso}: messaggio diagnostico su stderr");
}

#[test]
fn argomenti_invalidi_exit_non_zero_stdout_vuoto() {
    rifiuta(&[], "nessun argomento");
    rifiuta(&["--seed", "1042"], "manca --ticks");
    rifiuta(&["--ticks", "3"], "manca --seed");
    rifiuta(&["--seed", "abc", "--ticks", "3"], "seed non numerico");
    rifiuta(&["--seed", "1042", "--ticks", "abc"], "ticks non numerico");
    rifiuta(&["--seed", "-1", "--ticks", "3"], "seed negativo");
    rifiuta(&["--seed", "1042", "--ticks", "-1"], "ticks negativo");
    rifiuta(
        &["--seed", "18446744073709551616", "--ticks", "3"],
        "seed oltre u64",
    );
    rifiuta(&["--seed", "1042", "--ticks", "3", "extra"], "argomento extra");
    rifiuta(&["--seed", "1042", "--ticks"], "flag senza valore");
}
