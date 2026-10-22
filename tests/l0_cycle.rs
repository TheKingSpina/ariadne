//! Test del ciclo senso→azione→stato — dispatch B4, SPEC L0 v0.2
//! (sezione "Ciclo senso→azione→stato (un tick)" + riga 44 dello Schema PRNG).
//!
//! Contratto sotto test:
//! - un ciclo emette ESATTAMENTE 3 eventi (SENSE, ACTION, STATE) con lo stesso
//!   tick = ultimo tick + 1, seq consecutivi;
//! - il tick avanza solo a ciclo chiuso: mai cicli parziali;
//! - budget draw: 1 draw per tick (SENSE), 0 dalla policy, 0 dalla genesi —
//!   snapshot `state` dell'ultimo STATE = stato interno di `Pcg64::new(seed)`
//!   dopo ESATTAMENTE N draw (riga 44: "prima i SENSE, poi eventuali draw della
//!   policy, budget draw per tick esplicito — niente dipendenze nascoste");
//! - forme esatte dei payload: SENSE {stimolo: stringa decimale del draw u64
//!   COMPLETO (nessuna mascheratura: il payload canonico ammette solo i64, la
//!   policy fa il parse)}, ACTION {mossa: parse(stimolo) % 3}, STATE
//!   {action: n° di ACTION accumulati, sense: n° di SENSE accumulati,
//!   state: snapshot PCG64 decimale, tick} — la chiave dello snapshot è
//!   `state` (NON `prng`: la fold legge `state`, e nella genesi `prng` è il
//!   nome dell'algoritmo); l'evento STATE NON contiene `head` (è già il
//!   prev_hash dell'evento stesso: un `head` nel payload sarebbe circolare
//!   rispetto al proprio hash);
//! - determinismo forte: due log indipendenti, stesso seed, stessi cicli →
//!   `to_jsonl()` byte-identici;
//! - out-of-range: stimolo che NON entra nella scala i64 del payload canonico
//!   (> i64::MAX) è gestito come stringa a entropia completa (parse u64 senza
//!   mascheratura); seme diverso → log diverso.
//!
//! Il log L0 resta intatto: append-only, catena verificabile, replay del JSONL.

use ariadne::prng::Pcg64;
use ariadne::{cycle, EventType, Log, State};

/// N di cicli per i test di storico.
const N: u64 = 7;

/// Sequenza 0,1,1,1,2,2,2,… attesa: genesi (tick 0) + 3 eventi per tick.
fn tick_attesi(n_cicli: u64) -> Vec<u64> {
    let mut ticks = vec![0u64];
    for tick in 1..=n_cicli {
        for _ in 0..3 {
            ticks.push(tick);
        }
    }
    ticks
}

// ---------------------------------------------------------------------------
// Struttura del ciclo: 3 eventi, stesso tick, seq consecutivi
// ---------------------------------------------------------------------------

/// Un ciclo emette esattamente 3 eventi, tipi in ordine SENSE→ACTION→STATE,
/// stesso tick (= 1, primo ciclo dopo la genesi), seq consecutivi.
#[test]
fn un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi() {
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);

    cycle::run_cycle(&mut log, &mut rng).expect("il ciclo è legale su un log verificato");

    let events = log.events();
    assert_eq!(events.len(), 4, "genesi + 3 eventi del ciclo");
    assert_eq!(events[0].r#type, EventType::State, "la genesi è STATE");
    assert_eq!(events[1].r#type, EventType::Sense);
    assert_eq!(events[2].r#type, EventType::Action);
    assert_eq!(events[3].r#type, EventType::State);

    // seq consecutivi: la genesi è seq 0, il ciclo occupa 1..=3.
    for (i, e) in events.iter().enumerate() {
        assert_eq!(e.seq, i as u64, "seq consecutivi da 0, nessun gap");
    }

    // stesso tick per i 3 eventi del ciclo = ultimo tick (0, genesi) + 1.
    let tick_ciclo = events[1].tick;
    assert_eq!(tick_ciclo, 1, "tick = ultimo tick + 1, avanzato a ciclo chiuso");
    assert_eq!(events[2].tick, tick_ciclo);
    assert_eq!(events[3].tick, tick_ciclo);
}

/// Mai due SENSE consecutivi senza ACTION, mai cicli parziali: il log dopo N
/// cicli è sempre genesi + N triple consecutive SENSE → ACTION → STATE.
#[test]
fn mai_cicli_parziali_ritmo_sense_action_state() {
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    cycle::run_cycles(&mut log, &mut rng, N).expect("N cicli legali");

    let events = log.events();
    // Genesi + 3 eventi per ciclo: conteggio esatto; ogni tripla è
    // SENSE, ACTION, STATE nel medesimo tick.
    assert_eq!(events.len(), 1 + 3 * N as usize);
    for tripla in events[1..].chunks_exact(3) {
        assert_eq!(tripla[0].r#type, EventType::Sense);
        assert_eq!(tripla[1].r#type, EventType::Action);
        assert_eq!(tripla[2].r#type, EventType::State);
        assert_eq!(tripla[0].tick, tripla[1].tick);
        assert_eq!(tripla[1].tick, tripla[2].tick);
    }
}

/// Nessun tick salta: la sequenza dei tick nel log è 0,1,1,1,2,2,2,…
/// (genesi + 3 eventi per tick).
#[test]
fn nessun_tick_salta_sequenza_0_1_1_1_2_2_2() {
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    cycle::run_cycles(&mut log, &mut rng, N).expect("N cicli legali");

    let ticks: Vec<u64> = log.events().iter().map(|e| e.tick).collect();
    let attesi = tick_attesi(N);
    assert_eq!(ticks, attesi, "tick = contatore logico di ciclo, mai un timestamp");

    // doppio riscontro dopo la serializzazione: il replay del JSONL dà la stessa
    // sequenza di tick, byte per byte.
    let rigiocato = Log::from_jsonl(&log.to_jsonl()).expect("JSONL canonico: replay legale");
    let ticks_replay: Vec<u64> = rigiocato.events().iter().map(|e| e.tick).collect();
    assert_eq!(ticks_replay, attesi);
}

/// Dopo N cicli il tick è N (azione dell'ultimo evento = STATE del ciclo N) e
/// la catena è verificabile end-to-end con `verify()`.
#[test]
fn dopo_n_cicli_tick_n_e_catena_verificabile() {
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    cycle::run_cycles(&mut log, &mut rng, N).expect("N cicli legali");

    assert!(log.verify().is_ok(), "catena di hash verificabile end-to-end");
    let stato = State::fold(&log).expect("fold su log verificato");
    assert_eq!(stato.tick, N as i64, "il tick avanza solo a ciclo chiuso: N cicli → tick N");
}

// ---------------------------------------------------------------------------
// Determinismo forte e differenza per seme
// ---------------------------------------------------------------------------

/// Determinismo forte: due log indipendenti, stesso seed, stesso numero di
/// cicli → `to_jsonl()` byte-identici.
#[test]
fn determinismo_forte_due_log_indipendenti_byte_identici() {
    let seed = 1042u64;
    let n_cicli = 5u64;

    let mut log_a = Log::genesis(seed);
    let mut rng_a = Pcg64::new(seed);
    cycle::run_cycles(&mut log_a, &mut rng_a, n_cicli).expect("cicli legali A");

    let mut log_b = Log::genesis(seed);
    let mut rng_b = Pcg64::new(seed);
    cycle::run_cycles(&mut log_b, &mut rng_b, n_cicli).expect("cicli legali B");

    let a = log_a.to_jsonl();
    let b = log_b.to_jsonl();
    assert_eq!(a, b, "stesso log → stessa storia: byte per byte");
    assert_eq!(a.len(), b.len());
    // il confronto è sui byte delle stringhe: il JSONL non è vuoto e la
    // verifica non passa per nessuna normalizzazione.
    assert!(!a.is_empty());
    // il replay del JSONL rigenera un log che serializza identico (replay bit-perfetto).
    let rigiocato = Log::from_jsonl(&a).expect("JSONL canonico");
    assert_eq!(rigiocato.to_jsonl(), a);
}

/// Seme diverso → log diverso (byte diversi nel JSONL).
#[test]
fn seme_diverso_produce_log_diverso() {
    let n_cicli = 3u64;
    let mut log_a = Log::genesis(1042);
    let mut rng_a = Pcg64::new(1042);
    cycle::run_cycles(&mut log_a, &mut rng_a, n_cicli).expect("cicli legali A");

    let mut log_b = Log::genesis(1043);
    let mut rng_b = Pcg64::new(1043);
    cycle::run_cycles(&mut log_b, &mut rng_b, n_cicli).expect("cicli legali B");

    let a = log_a.to_jsonl();
    let b = log_b.to_jsonl();
    assert_ne!(a, b, "semi diversi → storie diverse");
}

// ---------------------------------------------------------------------------
// Budget draw: 1 draw per tick (SENSE), 0 dalla policy, 0 dalla genesi
// ---------------------------------------------------------------------------

/// Lo snapshot `state` dell'ultimo evento STATE = stato interno di
/// `Pcg64::new(seed)` dopo esattamente N draw: dimostra 1 draw per tick e
/// 0 dalla policy (nessuna dipendenza di draw nascosta).
#[test]
fn budget_draw_1_per_tick_0_per_la_policy() {
    let seed = 1042u64;
    let n_cicli = 5u64;

    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    cycle::run_cycles(&mut log, &mut rng, n_cicli).expect("cicli legali");

    // snapshot dell'ultimo STATE.
    let ultimo_state = log
        .events()
        .iter()
        .rev()
        .find(|e| e.r#type == EventType::State)
        .expect("dopo almeno un ciclo esiste un STATE");
    let snapshot = match ultimo_state.payload.get("state") {
        Some(ariadne::Value::Str(s)) => s.clone(),
        altro => panic!("l'evento STATE porta `state` stringa, trovato: {altro:?}"),
    };

    // riferimento indipendente: il medesimo seed, esattamente N draw.
    let mut riferimento = Pcg64::new(seed);
    for _ in 0..n_cicli {
        riferimento.next_u64();
    }
    assert_eq!(snapshot, riferimento.state_decimal(), "N cicli = N draw, nessun draw extra");

    // la genesi consuma 0 draw: il primo ciclo parte dallo stato iniziale e il
    // suo SENSE è ESATTAMENTE il PRIMO draw della sequenza.
    let primo_sense = log
        .events()
        .iter()
        .find(|e| e.r#type == EventType::Sense)
        .expect("il primo SENSE è nel primo ciclo");
    let stimolo_1 = match primo_sense.payload.get("stimolo") {
        Some(ariadne::Value::Str(s)) => s.clone(),
        altro => panic!("il SENSE porta `stimolo` stringa, trovato: {altro:?}"),
    };
    let mut primo_draw = Pcg64::new(seed);
    let d1 = primo_draw.next_u64();
    // riparte dal medesimo seme: il secondo draw è il medesimo della sequenza
    // referenziata (stessa istanza, secondo output).
    let mut ripetuto = Pcg64::new(seed);
    let _ = ripetuto.next_u64();
    assert_ne!(d1, ripetuto.next_u64(), "i draw consecutivi differiscono (sconto il caso degenero)");
    assert_eq!(
        stimolo_1,
        d1.to_string(),
        "il primo SENSE è il primo draw: la genesi non ha consumato nulla"
    );
}

// ---------------------------------------------------------------------------
// Policy: mossa = parse(stimolo) % 3, zero draw
// ---------------------------------------------------------------------------

/// ACTION.mossa == parse(SENSE.stimolo) % 3, su più tick.
#[test]
fn mossa_e_parse_stimolo_mod_3_su_piu_tick() {
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    let n_cicli = 6u64;
    cycle::run_cycles(&mut log, &mut rng, n_cicli).expect("cicli legali");

    let events = log.events();
    for tripla in events[1..].chunks_exact(3) {
        let (s, a) = (&tripla[0], &tripla[1]);
        assert_eq!(s.r#type, EventType::Sense);
        assert_eq!(a.r#type, EventType::Action);
        let stimolo = match s.payload.get("stimolo") {
            Some(ariadne::Value::Str(x)) => x.clone(),
            altro => panic!("il SENSE porta `stimolo` stringa, trovato: {altro:?}"),
        };
        let mossa = match a.payload.get("mossa") {
            Some(ariadne::Value::Int(m)) => *m,
            altro => panic!("l'ACTION porta `mossa` intero, trovato: {altro:?}"),
        };
        // parse INDEPENDENTE del test: u64 completo, nessuna mascheratura.
        let valore: u64 = stimolo.parse().expect("stimolo solo cifre, parse u64");
        assert_eq!(
            mossa,
            (valore % 3) as i64,
            "mossa = parse(stimolo) % 3 al tick {}",
            s.tick
        );
        assert!(
            (0..=2).contains(&mossa),
            "la mossa è in range 0..2 al tick {}",
            s.tick
        );
    }

    // la policy copre tutte e tre le mosse nel corpus usato (il draw è uniforme:
    // in 6 tick tutte le classi mod 3 compaiono quasi certamente; se non
    // compaiono il test resta comunque valido, è solo debole — per questo un
    // secondo riscontro usa la funzione di policy diretta su valori estremi).
    let estremi = [0u64, 1, 2, 3, u64::MAX, u64::MAX - 1];
    for v in estremi {
        assert_eq!(
            cycle::mossa_from_stimolo(&v.to_string()).expect("stimolo legale"),
            (v % 3) as i64,
            "policy = parse % 3 anche sui valori estremi"
        );
    }
}

/// `stimolo` è stringa di sole cifre, senza segno né zeri in testa: la forma
/// canonica del draw u64 completo (mascherarlo perderebbe entropia).
#[test]
fn stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa() {
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    let n_cicli = 10u64;
    cycle::run_cycles(&mut log, &mut rng, n_cicli).expect("cicli legali");

    for e in log
        .events()
        .iter()
        .filter(|e| e.r#type == EventType::Sense)
    {
        let stimolo = match e.payload.get("stimolo") {
            Some(ariadne::Value::Str(x)) => x,
            altro => panic!("il SENSE porta `stimolo` stringa, trovato: {altro:?}"),
        };
        assert!(!stimolo.is_empty(), "stimolo non vuoto");
        assert!(
            stimolo.bytes().all(|b| b.is_ascii_digit()),
            "stimolo '{}': sole cifre ASCII",
            stimolo
        );
        assert!(!stimolo.starts_with('-'), "stimolo senza segno");
        assert!(!stimolo.starts_with('+'), "stimolo senza segno");
        assert!(
            stimolo == "0" || !stimolo.starts_with('0'),
            "stimolo senza zeri in testa (il solo valore nullo è \"0\")"
        );
        // entropia COMPLETA: la stringa round-trippa su u64 senza perdita.
        let valore: u64 = stimolo.parse().expect("solo cifre");
        assert_eq!(stimolo, &valore.to_string(), "round-trip decimale esatto");
    }
}

// ---------------------------------------------------------------------------
// Out-of-range e differenza per input
// ---------------------------------------------------------------------------

/// Se il draw sfora la scala i64 del payload canonico (> i64::MAX, esattamente
/// ciò per cui `stimolo` è UNA stringa), il ciclo resta legale: parse u64 senza
/// mascheratura, mossa calcolata sul valore COMPLETO e catena verificata.
#[test]
fn stimolo_oltre_i64_max_parse_completo_senza_mascheratura() {
    // seme deterministico del corpus (1000–1099 di L6) che produce un primo
    // draw oltre i64::MAX: la ricerca è nel test, la scelta non è casuale.
    let seed = (1000u64..=1099)
        .find(|&s| Pcg64::new(s).next_u64() > i64::MAX as u64)
        .expect("nel corpus 1000–1099 esiste un primo draw oltre i64::MAX");
    assert!(seed >= 1000 && seed <= 1099);

    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    cycle::run_cycle(&mut log, &mut rng).expect("ciclo legale con stimolo oltre i64");

    let sense = &log.events()[1];
    let action = &log.events()[2];
    let stimolo = match sense.payload.get("stimolo") {
        Some(ariadne::Value::Str(x)) => x.clone(),
        altro => panic!("il SENSE porta `stimolo` stringa, trovato: {altro:?}"),
    };
    let mossa = match action.payload.get("mossa") {
        Some(ariadne::Value::Int(m)) => *m,
        altro => panic!("l'ACTION porta `mossa` intero, trovato: {altro:?}"),
    };

    // entropia completa: il valore RESTA oltre la scala i64 del payload.
    let valore: u64 = stimolo.parse().expect("solo cifre");
    assert!(valore > i64::MAX as u64, "il stimolo conserva l'intero range u64");

    // la mossa è calcolata sul valore COMPLETO (u64), non sul residuo mascherato:
    // parse(stimolo) % 3 con il parse a entropia intera.
    assert_eq!(mossa, (valore % 3) as i64);
    // e coincide con la policy diretta.
    assert_eq!(
        cycle::mossa_from_stimolo(&stimolo).expect("stimolo legale"),
        (valore % 3) as i64
    );

    // catena e payload canonici intatti.
    assert!(log.verify().is_ok());
    let rigiocato = Log::from_jsonl(&log.to_jsonl()).expect("JSONL canonico anche qui");
    assert!(rigiocato.verify().is_ok());
}

// ---------------------------------------------------------------------------
// Fold dello stato sull'intero log
// ---------------------------------------------------------------------------

/// La fold di `state.rs` sull'intero log (genesi + N cicli) dà:
/// tick == N, sense == N, action == N, prng == snapshot dell'ultimo STATE.
#[test]
fn fold_su_log_intero_con_ultima_snapshot_di_state() {
    let seed = 1042u64;
    let mut log = Log::genesis(seed);
    let mut rng = Pcg64::new(seed);
    cycle::run_cycles(&mut log, &mut rng, N).expect("N cicli legali");
    log.verify().expect("log verificato prima della fold");

    let stato = State::fold(&log).expect("fold su log verificato");
    assert_eq!(stato.tick, N as i64);
    assert_eq!(stato.sense, N as i64, "un SENSE per ciclo, nessuno extra");
    assert_eq!(stato.action, N as i64, "un ACTION per ciclo, nessuno extra");

    // prng == snapshot dell'ultimo STATE letto verbatim (la fold non ricalcola).
    let ultimo_state = log
        .events()
        .iter()
        .rev()
        .find(|e| e.r#type == EventType::State)
        .expect("dopo almeno un ciclo esiste un STATE");
    let snapshot = match ultimo_state.payload.get("state") {
        Some(ariadne::Value::Str(s)) => s.clone(),
        altro => panic!("l'evento STATE porta `state` stringa, trovato: {altro:?}"),
    };
    assert_eq!(stato.prng, snapshot);

    // riscontro indipendente del snapshot: N draw dal seme.
    let mut riferimento = Pcg64::new(seed);
    for _ in 0..N {
        riferimento.next_u64();
    }
    assert_eq!(stato.prng, riferimento.state_decimal());

    // L'evento STATE non contiene `head` (sarebbe circolare rispetto al proprio
    // hash: l'hash di testa è già prev_hash dell'evento stesso).
    assert!(
        ultimo_state.payload.get("head").is_none(),
        "il payload di STATE non contiene `head`"
    );
}
