//! Harness L6 — esperimento appaiato memoria vs controllo (legge
//! `notes/L6_legge_valutazione.md`, commit d12cf07, regole L6.6–L6.11).
//!
//! # Scopo
//!
//! Eseguire il corpus completo di 100 coppie appaiate (seed 1000..=1099),
//! scrivere i log JSONL dei run come evidenza, calcolare la statistica
//! Wilcoxon appaiata una coda sul criterio §4 e redigere il report
//! aggregato con l'esito empirico ONESTO (L6.8: qualunque sia il p-value,
//! viene registrato; mai manomesso).
//!
//! # Eccezione documentata al divieto di I/O e wall-clock
//!
//! L6.11: wall-clock e I/O vivono SOLO in questo harness, mai nella
//! libreria (`valuation.rs` è pura, `run_pair` non tocca né file né
//! tempo). Tetto: 5,0 secondi per coppia di run (§3); un run che sfora è
//! una VIOLAZIONE di protocollo e fallisce l'esperimento intero (L6.8,
//! nessuna esclusione selettiva di seed).
//!
//! # Contratto
//!
//! - `cargo run --bin l6_experiment` — nessun argomento: corpus fisso
//!   1000..=1099 (L6.6), niente parametri variabili.
//! - Artefatti: `runs/l6/seed_<s>_memoria.jsonl` e
//!   `runs/l6/seed_<s>_controllo.jsonl` (200 log, directory NON committata)
//!   e `reports/l6_esperimento.md` (report aggregato, committato).
//! - Criterio di successo (§4): `p < 0.01` E `Δ̄ ≥ 0.30` (unità reward) E
//!   zero violazioni su tutti i 100 run. Benché la media differenza sia
//!   dominante, il p-value è calcolato dal Wilcoxon (L6.7), non dalla
//!   media.
//! - Benché la SPEC §4 chiami il random uniforme "0,50", con 3 classi il
//!   benchmark onesto è 1/3 ≈ 0,333 (anomalia SPEC, triage Q12d,
//!   registrata nel report; l'oracolo resta 1,00).

use std::fs;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ariadne::valuation::{run_pair, wilcoxon_one_sided_greater, TEST_TASKS};

/// Corpus completo: 100 coppie appaiate, seed 1000..=1099 (L6.6).
const SEEDS: std::ops::RangeInclusive<u64> = 1000..=1099;
/// Tetto wall-clock per coppia di run (§3).
const TIMEOUT_SECS: f64 = 5.0;

fn main() {
    fs::create_dir_all("runs/l6").expect("creazione runs/l6");
    let mut righe = Vec::new();
    let mut diffs: Vec<i64> = Vec::new();
    let mut violazioni: Vec<String> = Vec::new();
    let mut replay_ok_count = 0usize;

    for seed in SEEDS {
        let t0 = Instant::now();
        let r = run_pair(seed);
        let secs = t0.elapsed().as_secs_f64();
        if secs > TIMEOUT_SECS {
            violazioni.push(format!("seed {seed}: timeout {secs:.3}s > {TIMEOUT_SECS}s"));
        }
        if !r.replay_ok {
            violazioni.push(format!("seed {seed}: replay non bit-perfetto"));
        } else {
            // replay_ok attesta ENTRAMBI i log della coppia (L6.9)
            replay_ok_count += 2;
        }
        fs::write(format!("runs/l6/seed_{seed}_memoria.jsonl"), &r.memoria.jsonl)
            .expect("scrittura log memoria");
        fs::write(
            format!("runs/l6/seed_{seed}_controllo.jsonl"),
            &r.controllo.jsonl,
        )
        .expect("scrittura log controllo");
        // media di test per braccio in milli-reward (somma / 40: esatta,
        // ogni contributo è 0 o 1000 milli)
        let media_m = r.memoria.test_milli / TEST_TASKS as i64;
        let media_c = r.controllo.test_milli / TEST_TASKS as i64;
        diffs.push(media_m - media_c);
        righe.push((seed, media_m, media_c, secs, r.replay_ok));
    }

    let stat =
        wilcoxon_one_sided_greater(&diffs).expect("Wilcoxon su 100 differenze (mai tutte zero)");
    let successo = stat.p < 0.01 && stat.delta_bar >= 0.30 && violazioni.is_empty();
    let verdict = if successo {
        "SUCCESSO"
    } else {
        "ESITO NEGATIVO (onesto: criterio non raggiunto)"
    };

    let secs_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut report = String::new();
    report.push_str("# L6 — esperimento appaiato memoria vs controllo\n\n");
    report.push_str("- Legge: `notes/L6_legge_valutazione.md` (commit d12cf07), L6.6–L6.11.\n");
    report.push_str(&format!("- Esecuzione: epoch {secs_epoch} (harness `l6_experiment`).\n"));
    report.push_str(&format!(
        "- Corpus: {} coppie, seed {}..={} — nessuna esclusione (L6.6).\n",
        righe.len(),
        SEEDS.start(),
        SEEDS.end()
    ));
    report.push_str(&format!(
        "- Parametri firmati: 120 task per run ({} apprendimento + {} prove di test); \
         tetto wall-clock {TIMEOUT_SECS}s per coppia; Wilcoxon appaiato una coda, \
         α = 0.01 (§4).\n",
        80,
        TEST_TASKS
    ));
    report.push_str(&format!(
        "- Benchmark passivi (§4): oracolo = 1.000 milli-reward (reward 1,00); \
         random uniforme = 333 milli (reward 0,333 con 3 classi — anomalia \
         SPEC §4 «0,50» registrata onesta, triage Q12d).\n"
    ));
    report.push_str(&format!(
        "- Replay bit-perfetto: {replay_ok_count}/{} log integra (verify() + fold_run \
         identico byte per byte, L6.9).\n",
        2 * righe.len()
    ));
    report.push_str(&format!(
        "- Violazioni di protocollo: {} (timeout, replay, forme L6.2).\n",
        violazioni.len()
    ));
    report.push_str(&format!(
        "- Statistica: W = {:.1}, n = {}, p = {:.6}, Δ̄ = {:.4} (unità reward).\n",
        stat.w, stat.n, stat.p, stat.delta_bar
    ));
    report.push_str(&format!("- **VERDICT (§4): {verdict}**\n\n"));
    if !violazioni.is_empty() {
        report.push_str("## Violazioni registrate\n\n");
        for v in &violazioni {
            report.push_str(&format!("- {v}\n"));
        }
        report.push('\n');
    }
    report.push_str("## Per-run (media di test, milli-reward)\n\n");
    report.push_str("| seed | memoria | controllo | diff | tempo (s) | replay |\n");
    report.push_str("|---|---|---|---|---|---|\n");
    let mut somma_diff = 0i64;
    for (seed, m, c, secs, replay) in &righe {
        somma_diff += m - c;
        report.push_str(&format!(
            "| {seed} | {m} | {c} | {} | {secs:.3} | {} |\n",
            m - c,
            if *replay { "ok" } else { "FALLITO" }
        ));
    }
    let media_diff = somma_diff as f64 / righe.len() as f64;
    report.push_str(&format!(
        "\nMedia diff = {media_diff:.1} milli-reward/run = Δ̄ {:.4} (reward), \
         coerente con la statistica.\n",
        media_diff / 1000.0
    ));

    fs::write("reports/l6_esperimento.md", &report).expect("scrittura report");
    println!(
        "L6: {} coppie, {} violazioni, W={:.1} p={:.6} Δ̄={:.4} → {verdict}",
        righe.len(),
        violazioni.len(),
        stat.w,
        stat.p,
        stat.delta_bar
    );
}
