//! Test L6 — matematica del valutatore (`valuation.rs`).
//!
//! [Legge `notes/L6_legge_valutazione.md` (commit d12cf07), L6.3, L6.7.]
//! Test-first: questo file è nel PRIMO commit ed è ROSSO per assenza del
//! modulo (E0432/E0433). Autorizzazioni: firma umana 01/11/2026 (triage
//! Q1–Q12 + 120 task 80+40; canale «orchestratore» per l'implementazione).

use ariadne::valuation::{
    generate_pair, wilcoxon_one_sided_greater, CUES, LEARNING_TASKS, TEST_TASKS,
};

// ---------------------------------------------------------------------------
// generate_pair — determinismo, bilanciamento, struttura (L6.3)
// ---------------------------------------------------------------------------

#[test]
fn l6_pair_generation_is_deterministic_byte_per_byte() {
    let a = generate_pair(1000);
    let b = generate_pair(1000);
    assert_eq!(a.tasks.len(), b.tasks.len());
    for (ta, tb) in a.tasks.iter().zip(b.tasks.iter()) {
        assert_eq!(ta.cue, tb.cue);
        assert_eq!(ta.giusta, tb.giusta);
    }
    assert_eq!(a.rule, b.rule, "stesso seed → stessa tabella");
    assert_eq!(a.tasks[0].cue, a.tasks[0].cue);
}

#[test]
fn l6_rule_is_balanced_four_cues_per_class() {
    let plan = generate_pair(1000);
    assert_eq!(plan.rule.len(), CUES);
    let mut counts = [0i64; 3];
    for c in &plan.rule {
        assert!(*c >= 0 && *c <= 2, "classe 0..2");
        counts[*c as usize] += 1;
    }
    assert_eq!(counts, [4, 4, 4], "tabella bilanciata (L6.3)");
}

#[test]
fn l6_tasks_80_learning_then_40_test() {
    let plan = generate_pair(1042);
    assert_eq!(plan.tasks.len(), LEARNING_TASKS + TEST_TASKS);
    assert_eq!(LEARNING_TASKS, 80);
    assert_eq!(TEST_TASKS, 40);
}

#[test]
fn l6_different_seeds_give_different_plans() {
    let a = generate_pair(1000);
    let b = generate_pair(1001);
    // non iniettività universale: basta che le due tabelle NON coincidano
    // per il seed scelto (divergenza scoped, pattern L4/L5)
    assert_ne!(a.rule, b.rule, "seed diversi → regole diverse (1000 vs 1001)");
}

// ---------------------------------------------------------------------------
// Wilcoxon appaiato una coda (L6.7)
// ---------------------------------------------------------------------------

#[test]
fn l6_wilcoxon_w_and_p_on_known_small_case() {
    // diffs [100,200,300,400]: ranghi 1,2,3,4 tutti positivi → W=10;
    // mu = 4*5/4 = 5; sigma = sqrt(4*5*9/24) = sqrt(7.5) ≈ 2.738613;
    // z = (10 − 5 − 0.5)/2.738613 ≈ 1.646207;
    // p = 0.5*erfc(z/sqrt(2)) ≈ 0.0498 (calcolo indipendente a mano).
    let r = wilcoxon_one_sided_greater(&[100, 200, 300, 400]).unwrap();
    assert_eq!(r.n, 4);
    assert!((r.w - 10.0).abs() < 1e-9, "W = 10 (rango pieno)");
    assert!(r.p > 0.04 && r.p < 0.06, "p ≈ 0.05, ottenuto {}", r.p);
    assert!((r.delta_bar - 0.25).abs() < 1e-9, "Δ̄ = 250/1000 = 0.25");
}

#[test]
fn l6_wilcoxon_ties_use_average_rank() {
    // [100, 100, 200]: ranghi |d| = 1.5, 1.5, 3, tutti positivi → W = 6.
    let r = wilcoxon_one_sided_greater(&[100, 100, 200]).unwrap();
    assert_eq!(r.n, 3);
    assert!((r.w - 6.0).abs() < 1e-9, "tie-break = rango medio: W = 6");
}

#[test]
fn l6_wilcoxon_discards_zeros_and_negative_diffs_lower_w() {
    // [0, 100, 200] → n=2 (zero scartato), ranghi 1,2 → W=3.
    let r = wilcoxon_one_sided_greater(&[0, 100, 200]).unwrap();
    assert_eq!(r.n, 2);
    assert!((r.w - 3.0).abs() < 1e-9);
    // [−100, 100, 200]: |−100| = |+100| legano → rango medio 1.5 per
    // entrambe (L6.7); solo le differenze positive entrano in W → 1.5+3.
    let r2 = wilcoxon_one_sided_greater(&[-100, 100, 200]).unwrap();
    assert!((r2.w - 4.5).abs() < 1e-9, "tie rango medio: W = 4.5");
    assert!(r2.p > r.p, "evidenza minore → p maggiore (una coda >)");
}

#[test]
fn l6_wilcoxon_all_zero_is_no_pairs() {
    assert!(wilcoxon_one_sided_greater(&[0, 0, 0]).is_err());
    assert!(wilcoxon_one_sided_greater(&[]).is_err());
}

#[test]
fn l6_wilcoxon_strong_evidence_gives_small_p() {
    // 100 differenze tutte positive e grandi: p deve essere << 0.01.
    let diffs: Vec<i64> = (1..=100).map(|i| 400 + i as i64).collect();
    let r = wilcoxon_one_sided_greater(&diffs).unwrap();
    assert!(r.p < 0.01, "p = {} deve essere < 0.01", r.p);
    assert!(r.delta_bar > 0.4, "Δ̄ ≈ 0.45");
}

#[test]
fn l6_wilcoxon_delta_bar_in_reward_units() {
    let r = wilcoxon_one_sided_greater(&[500, 700]).unwrap();
    assert!((r.delta_bar - 0.6).abs() < 1e-9, "Δ̄ = 600 milli / 1000");
}
