//! Ariadne L0 — engine PCG64 (PCG XSL RR 128/64), SPEC L0 v0.2 "Schema PRNG".
//!
//! Implementazione conforme alla famiglia PCG di Melissa O'Neill:
//! LCG a 128 bit con moltiplicatore `M = 0x2360ED051FC65DA44385DF649FCCF645`
//! (il default 128 bit del riferimento C++) e incremento dispari; l'uscita è
//! la funzione XSL RR 128/64 applicata allo stato DOPO il passo LCG, come
//! nell'implementazione di riferimento per stati a 128 bit:
//!
//! ```text
//! step:    state = state * M + increment            (aritmetica mod 2^128)
//! output:  rot  = top 6 bit dello stato post-step   (state >> 122, & 63)
//!          xsl  = (state >> 64) ^ state             (low 64 bit dello XOR)
//!          out  = rotr64(xsl, rot)
//! ```
//!
//! Non è una variante inventata: le costanti e la forma dell'output seguono
//! `pcg_random.hpp` (xsl_rr_mixin con xtype=64 bit, itype=128 bit — opbits=6,
//! xshift=64, bottomspare=0) e la semantica `output_previous = false` che il
//! riferimento usa proprio per stati a 128 bit.
//!
//! # Seeding da `seed: u64` — DECISIONE DOCUMENTATA
//!
//! La SPEC L0 non fissa la procedura di espansione del seed; questa è la
//! decisione presa, riproducibile da chiunque con la sola descrizione
//! (decisa in governance `opencode:Ariadne`, tag `core`):
//!
//! 1. Espansione **splitmix64** del seed: due output consecutivi
//!    `gs0`, `gs1` a partire dal contatore `x = seed`, dove uno step
//!    splitmix64 è:
//!    ```text
//!    x = x + 0x9E3779B97F4A7C15            (mod 2^64)
//!    z = (x ^ (x >> 30)) * 0xBF58476D1CE4E5B9   (mod 2^64)
//!    z = (z ^ (z >> 27)) * 0x94D049BB133111EB   (mod 2^64)
//!    z = z ^ (z >> 31)
//!    ```
//! 2. `initstate = gs0`, `initseq = gs1`, `increment = (initseq << 1) | 1`
//!    (dispari per costruzione, come nelle API di seeding PCG coi due
//!    parametri state/seq).
//! 3. Stato iniziale alla `pcg64_srandom_r` del riferimento C:
//!    `state = 0; state = state*M + increment; state = state + initstate;
//!    state = state*M + increment`. Vale a dire due passi LCG con l'inserimento
//!    di `initstate` tra i due, l'identità algebrica del costruttore
//!    `state = bump(initstate + increment)` del riferimento C++.
//!
//! Zero wall-clock, zero I/O, zero float, zero casualità fuori dallo stato:
//! `next_u64` dipende SOLO da (seed, storico dei draw dell'istanza).
//! Nessuna dipendenza: solo aritmetica intera a 128 bit nativa.

/// PCG64: PCG XSL RR 128/64, pronto per il seeding deterministico da `u64`.
///
/// Clonabile per istantanea di stato senza side effect (replay L0 §4).
#[derive(Clone, Debug)]
pub struct Pcg64 {
    /// Stato interno dell'LCG (128 bit).
    state: u128,
    /// Incremento dell'LCG (128 bit, dispari — periodo completo).
    increment: u128,
}

// Costanti splitmix64 (Steele/Lea/Flood), fisse per contratto.
const SM_GOLD: u64 = 0x9E3779B97F4A7C15;
const SM_M1: u64 = 0xBF58476D1CE4E5B9;
const SM_M2: u64 = 0x94D049BB133111EB;

/// Uno step splitmix64: avanza il contatore e produce l'output corrente.
fn splitmix64(counter: &mut u64) -> u64 {
    *counter = counter.wrapping_add(SM_GOLD);
    let mut z = *counter;
    z = (z ^ (z >> 30)).wrapping_mul(SM_M1);
    z = (z ^ (z >> 27)).wrapping_mul(SM_M2);
    z ^ (z >> 31)
}

impl Pcg64 {
    /// Moltiplicatore 128 bit fissato dalla SPEC L0 (default PCG 128 bit).
    pub const MULTIPLIER: u128 = 0x2360ED051FC65DA44385DF649FCCF645;

    /// Istanzia la sequenza PCG64 dal seed `u64`.
    ///
    /// Procedura di seeding (decisione documentata, vedi summary del modulo):
    /// splitmix64 del seed → (initstate, initseq); increment = (initseq << 1) | 1;
    /// due passi LCG con l'inserto di `initstate` in mezzo.
    pub fn new(seed: u64) -> Self {
        let mut counter: u64 = seed;
        let initstate = splitmix64(&mut counter); // gs0
        let initseq = splitmix64(&mut counter); // gs1
        // initseq « stream del seeding PCG a due parametri — sempre dispari.
        let increment = ((initseq as u128) << 1) | 1;

        // Inizializzazione algebrica della reference: bump(initstate + increment)
        // ⇒ identica ai due passi di pcg64_srandom_r:
        //   state = 0*M + inc → state += initstate → state*M + inc
        //   ossia (initstate + inc) * M + inc, aritmetica mod 2^128.
        let state = (initstate as u128)
            .wrapping_add(increment)
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(increment);

        Pcg64 { state, increment }
    }

    /// Un draw a 64 bit: fa avanzare l'LCG di un passo e restituisce l'output
    /// XSL RR 128/64 applicato allo stato post-avanzamento — semantica di
    /// riferimento per stati a 128 bit PCG.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(self.increment);

        let s = self.state;
        let rot = (s >> 122) as u32; // top 6 bit, già ≤ 63
        let xsl = ((s >> 64) ^ s) as u64; // low 64 bit dello XOR delle metà
        xsl.rotate_right(rot)
    }

    /// Stato interno in decimale (formato congelato per il campo `state`
    /// dell'evento genesi SPEC L0): stringa decimale, senza segno, senza
    /// zeri in testa (`"0"` se lo stato è nullo). Round-trippabile su u128.
    pub fn state_decimal(&self) -> String {
        self.state.to_string()
    }

    /// Incremento dell'LCG (accessore di sola lettura: necessario ai test di
    /// proprietà della SPEC — incremento sempre dispari). Dispari per
    /// costruzione in [`Pcg64::new`].
    pub fn increment(&self) -> u128 {
        self.increment
    }
}
