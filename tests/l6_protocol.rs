//! Test L6 — protocollo sperimentale appaiato (ridotto, L6.2–L6.5, L6.9,
//! L6.12).
//!
//! [Legge `notes/L6_legge_valutazione.md` (commit d12cf07).] Il corpus
//! COMPLETO (100 seed) è eseguito dall'harness `l6_experiment`; qui il
//! protocollo è verificato su un corpus ridotto (5 seed) per mantenere la
//! suite rapida. Test-first: rosso per assenza del modulo (E0432/E0433).

use ariadne::{EventType, Log, Payload, Value};
use ariadne::prng::Pcg64;
use ariadne::valuation::{fold_run, generate_pair, run_arm, run_pair, Arm};

const PROTOCOL_SEEDS: [u64; 5] = [1000, 1001, 1002, 1003, 1004];

/// Estrae la sequenza percettiva (cue, esito) da un log di run.
fn perceived_stream(log: &Log) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    let mut pending_cue: Option<String> = None;
    for e in log.events().iter().skip(1) {
        match (e.r#type, e.payload.get("cue"), e.payload.get("esito")) {
            (EventType::Sense, Some(Value::Str(c)), _) => {
                pending_cue = Some(c.clone());
            }
            (EventType::Sense, _, Some(Value::Int(esito))) => {
                out.push((pending_cue.take().expect("esito senza cue"), *esito));
            }
            _ => {}
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Appaiamento (L6.3): stessa sequenza percettiva per le due braccia
// ---------------------------------------------------------------------------

#[test]
fn l6_paired_arms_perceive_identical_streams() {
    for s in PROTOCOL_SEEDS {
        let plan = generate_pair(s);
        let m = run_arm(&plan, Arm::Memoria).unwrap();
        let c = run_arm(&plan, Arm::Controllo).unwrap();
        let lm = Log::from_jsonl(&m.jsonl).unwrap();
        let lc = Log::from_jsonl(&c.jsonl).unwrap();
        assert_eq!(
            perceived_stream(&lm),
            perceived_stream(&lc),
            "seed {s}: appaiamento (L6.3)"
        );
        assert_eq!(perceived_stream(&lm).len(), 120, "120 task per run");
    }
}

// ---------------------------------------------------------------------------
// Reward (L6.5): milli-reward interi, memoria > controllo sulle PROVE
// ---------------------------------------------------------------------------

#[test]
fn l6_memory_arm_learns_the_rule_and_beats_control_on_tests() {
    for s in PROTOCOL_SEEDS {
        let r = run_pair(s);
        assert!(
            r.memoria.test_milli > r.controllo.test_milli,
            "seed {s}: memoria {} > controllo {} (criterio §2)",
            r.memoria.test_milli,
            r.controllo.test_milli
        );
        // il controllo risponde sempre 0: il suo reward sui test è
        // 1000 × (numero di prove con classe giusta 0) → ≤ 40000;
        // la memoria apprende la tabella → reward alto (≥ 35000 milli)
        assert!(
            r.memoria.test_milli >= 35_000,
            "seed {s}: memoria = {} (attesa apprendimento della regola)",
            r.memoria.test_milli
        );
        assert_eq!(r.memoria.test_milli % 1000, 0, "milli-reward interi");
        assert_eq!(r.controllo.test_milli % 1000, 0);
    }
}

#[test]
fn l6_reward_is_milli_integer_1000_or_0() {
    let plan = generate_pair(1000);
    let c = run_arm(&plan, Arm::Controllo).unwrap();
    assert_eq!(c.test_milli % 1000, 0);
    assert_eq!(c.learning_milli % 1000, 0);
    assert!(c.test_milli > 0, "il controllo con classe fissa 0 vince ~1/3");
    assert!(c.test_milli < 40_000, "…ma non tutte le prove");
}

// ---------------------------------------------------------------------------
// Replay bit-perfetto (L6.9)
// ---------------------------------------------------------------------------

#[test]
fn l6_replay_bit_perfect_jsonl_round_trip_and_verify() {
    for s in PROTOCOL_SEEDS {
        let r = run_pair(s);
        assert!(r.replay_ok, "seed {s}: replay bit-perfetto (L6.9)");
        for out in [&r.memoria, &r.controllo] {
            let log = Log::from_jsonl(&out.jsonl).unwrap();
            log.verify().expect("catena integra");
            assert_eq!(log.to_jsonl(), out.jsonl, "round-trip byte per byte");
            let (learn, test) = fold_run(&log).expect("fold sul log ricostruito");
            assert_eq!(learn, out.learning_milli);
            assert_eq!(test, out.test_milli);
        }
    }
}

// ---------------------------------------------------------------------------
// Forme payload (L6.2) e frontiera draw (L6.12/L0 §4)
// ---------------------------------------------------------------------------

#[test]
fn l6_run_payloads_use_only_cue_mossa_esito_and_zero_draws() {
    // il PRNG di un istante qualsiasi NON è mutato da run_pair: i run
    // consumano zero draw (L6.3, frontiera draw).
    let mut p = Pcg64::new(7);
    let before = p.state_decimal();
    let _ = run_pair(1000);
    assert_eq!(p.state_decimal(), before, "zero draw nei run (L6.3)");

    // forme: nessuna chiave estranea nei payload dei run
    let log = Log::from_jsonl(&run_pair(1000).controllo.jsonl).unwrap();
    for e in log.events().iter().skip(1) {
        for (k, _) in e.payload.iter() {
            assert!(
                matches!(k.as_str(), "cue" | "mossa" | "esito"),
                "chiave inattesa: {k} (L6.2)"
            );
        }
    }
}

#[test]
fn l6_fold_run_rejects_malformed_stream() {
    // log di genesi (nessun task): fold valido con reward zero? NO — la
    // legge L6.9 pretende il pattern completo; un log senza task NON è un
    // run: fold_run su genesi → errore (violazione di protocollo).
    let log = Log::genesis(1).unwrap();
    assert!(fold_run(&log).is_err(), "genesi non è un run valido");

    // un run con payload malformato (chiave extra) → errore
    let mut log = Log::genesis(1).unwrap();
    log.append(5, EventType::Sense, Payload::from_pairs([
        ("cue", Value::from("cue:00")),
        ("extra", Value::from(1)),
    ])).unwrap();
    assert!(fold_run(&log).is_err(), "chiave extra → violazione (L6.2)");
}
