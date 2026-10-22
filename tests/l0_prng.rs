//! L0 — engine PCG64 (SPEC L0 v0.2, sezione "Schema PRNG"; coerenza con L6 v0.2 §3:
//! corpus su seed pre-fissati 1000–1099 su PCG64).
//!
//! Questo file è il commit TEST-ONLY: se resta solo qui, la suite è ROSSA
//! perché `src/prng.rs` non esiste ancora.
//!
//! Contratto testato (proprietà, NON golden numerici esterni — non esiste un
//! riferimento agganciato in repo e un valore inventato sarebbe peggio di
//! nessun valore):
//! - determinismo: stesso seed → sequenza identica da istanze indipendenti;
//! - seed diversi → sequenze diverse (verificato su tutto il corpus 1000–1099);
//! - istantanea di stato: `Clone` copia lo stato, i draw del clone sono senza
//!   side effect sull'originale, e lo "stato dopo N draw" è verifica
//!   indirettamente dal clone che ha fatto gli stessi N draw;
//! - `state_decimal()`: formato congelato per il campo `state` della genesi —
//!   decimale, senza segno, senza zeri in testa, round-trippabile su u128;
//! - l'incremento dell'LCG è sempre dispari (condizione di periodo completo
//!   della famiglia PCG: incremento dispari, stato arbitrario).
//!
//! Determinismo strutturale: nessuna sorgente esterna (wall-clock, I/O, rete,
//! float, casualità) entra nei draw; i draw dipendono SOLO da (seed, storico
//! dei draw dell'istanza). Lo prova il test di interleaving.

use ariadne::prng::Pcg64;

const SEED_RIF: u64 = 1042; // seme canonico dei goldens L0
const CORPUS: std::ops::Range<u64> = 1000..1100; // semi L6: 1000..=1099
const DRAWS: usize = 256;

fn draws(seed: u64, n: usize) -> Vec<u64> {
    let mut rng = Pcg64::new(seed);
    (0..n).map(|_| rng.next_u64()).collect()
}

// ---------------------------------------------------------------------------
// Proprietà 1 — determinismo: stesso seed, due istanze indipendenti, sequenza
// identica. Include interleaving: osservare lo stato tra un draw e l'altro non
// altera la sequenza (nessuna sorgente esterna nei draw).
// ---------------------------------------------------------------------------

#[test]
fn stesso_seed_produce_sequenza_identica_da_istanze_indipendenti() {
    let a = draws(SEED_RIF, DRAWS);
    let b = draws(SEED_RIF, DRAWS);
    assert_eq!(a.len(), DRAWS);
    assert_eq!(a, b, "stesso seed → sequenza identica");
}

#[test]
fn interleaving_di_osservazioni_non_altera_la_sequenza() {
    // Istanza osservata (state_decimal ad ogni draw) vs istanza "pulita":
    // le sequenze di draw devono coincidere, perché i draw non leggono
    // wall-clock, I/O o casualità fuori dallo stato proprio.
    let mut oss = Pcg64::new(SEED_RIF);
    let mut pulita = Pcg64::new(SEED_RIF);
    for _ in 0..DRAWS {
        let o = oss.next_u64();
        let p = pulita.next_u64();
        let _osservazione_senza_side_effect = pulita.state_decimal();
        assert_eq!(o, p, "i draw non dipendono da nessuna fonte esterna");
    }
    assert_eq!(oss.state_decimal(), pulita.state_decimal());
}

// ---------------------------------------------------------------------------
// Proprietà 2 — istantanea/replay: copia lo stato senza side effect e possa
// riprodurre il futuro dell'originale. Il riscontro indiretto dello "stato
// dopo N draw" è il confronto col clone che ha fatto gli stessi N draw.
// ---------------------------------------------------------------------------

#[test]
fn clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato() {
    // Un'istanza avanzata di N draw e un clone della medesima istanza fresca
    // che fa gli stessi N draw: alla fine i due stati coincidono.
    let mut a = Pcg64::new(SEED_RIF);
    let base = Pcg64::new(SEED_RIF);
    for _ in 0..DRAWS {
        a.next_u64();
    }
    let stato_dopo_n = a.state_decimal();

    let mut c = base.clone(); // istantanea PRIMA dei draw
    for _ in 0..DRAWS {
        c.next_u64();
    }
    assert_eq!(stato_dopo_n, c.state_decimal());

    // stesso stato ⇒ stessa sequenza futura.
    let fut_a = a.next_u64();
    let fut_c = c.next_u64();
    assert_eq!(fut_a, fut_c);
}

#[test]
fn clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect() {
    // Istantanea a metà percorso. Il futuro dell'originale deve essere
    // riprodotto da qualsiasi clone, e ogni clone dev'essere senza side
    // effect sull'originale.
    let mut orig = Pcg64::new(SEED_RIF);
    for _ in 0..DRAWS {
        orig.next_u64();
    }
    let snapshot = orig.clone();

    //
    let atteso: Vec<u64> = {
        let mut pr = Pcg64::new(SEED_RIF);
        for _ in 0..DRAWS {
            pr.next_u64();
        }
        (0..DRAWS).map(|_| pr.next_u64()).collect()
    };

    // Ogni clone riproduce il futuro atteso.
    for _ in 0..8 {
        let mut clone = snapshot.clone();
        assert_eq!(clone.next_u64(), atteso[0]);
        for k in 1..DRAWS/8 {
            assert_eq!(clone.next_u64(), atteso[k], "futuro del clone deviato");
        }
    }

    // La sequenza continua dell'originale è intatta: nessun side effect
    let fut_orig: Vec<u64> = (0..8).map(|_| orig.next_u64()).collect();
    assert_eq!(fut_orig, atteso[..8].to_vec());
}

// ---------------------------------------------------------------------------
// Proprietà 3 — seed diversi → sequenze diverse: tutta la matrice del corpus
// L6 (1000–1099), prime 64 uscite.
// ---------------------------------------------------------------------------

#[test]
fn il_corpus_l6_produce_sequenze_distinte_per_seme() {
    let n = 64;
    let mut seqs: Vec<Vec<u64>> = Vec::new();
    for seed in CORPUS.clone() {
        seqs.push(draws(seed, n));
    }
    for i in 0..seqs.len() {
        for j in (i + 1)..seqs.len() {
            assert_ne!(
                seqs[i], seqs[j],
                "seed {} e seed {} hanno prime {} uscite identiche",
                1000u64 + i as u64,
                1000u64 + j as u64,
                n
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Proprietà 4 — `state_decimal`: formato congelato (decimale, senza segno,
// senza zeri in testa, round-trippabile su u128).
// ---------------------------------------------------------------------------

#[test]
fn state_decimal_e_round_trippabile_e_senza_zero_in_testa() {
    let semi: Vec<u64> = vec![0, 1, 2, 999, 1000, 1042, 6543269, u64::MAX, 1099];
    for seed in semi {
        let mut rng = Pcg64::new(seed);
        for _ in 0..DRAWS {
            rng.next_u64();
        }
        let s = rng.state_decimal();
        // decimale puro
        assert!(
            !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()),
            "seed {}: stato non decimale",
            seed
        );
        // round-trip
        let v: u128 = s
            .parse()
            .unwrap_or_else(|e| panic!("seed {}: non parsabile ({:?})", seed, e));
        assert_eq!(v.to_string(), s, "seed {}: round-trip fallito", seed);
        // senza zeri in testa (per lo stato nullo è tollerato solo "0")
        if s != "0" {
            assert!(
                !s.starts_with('0'),
                "seed {}: stato con zeri in testa",
                seed
            );
        }
        // stato a 128 bit → realisticamente non superiore a 39 cifre
        assert!(s.len() <= 39, "seed {}: stato oltre 128 bit", seed);
    }
}

// ---------------------------------------------------------------------------
// Proprietà 5 — l'incremento dell'LCG è sempre dispari (condizione di periodo
// completo della famiglia PCG: incremento dispari, stato arbitrario).
// ---------------------------------------------------------------------------

#[test]
fn l_incremento_non_e_mai_pari() {
    let semi: Vec<u64> = vec![0, 1, 999, 1000, 1042, 1099, u64::MAX];
    for seed in semi {
        let rng = Pcg64::new(seed);
        assert_eq!(
            rng.increment() % 2,
            1,
            "seed {}: incremento pari",
            seed
        );
    }
}
