//! L6 — valutazione: esperimento appaiato memoria vs controllo (matematica
//! pura).
//!
//! Contratto: [`notes/L6_legge_valutazione.md`] (commit `d12cf07`), firmato
//! per delega umana il 01/11/2026 (triage Q1–Q12 + 120 task 80+40; canale
//! «orchestratore» per l'implementazione, quota free esaurita).
//!
//! - **L6.3**: `generate_pair` — piano offline per seed: tabella della
//!   regola nascosta bilanciata (4+4+4 su 12 cue) + 120 task (80
//!   apprendimento + 40 prove di test, ogni cue coperto ≥ 3 volte). Pura,
//!   deterministica byte per byte.
//! - **L6.4**: le due braccia — controllo memorialess (risponde sempre 0);
//!   memoria = maggioranza delle coppie (cue, esito) già percepite nel
//!   PROPRIO log per replay (tie-break: classe minore; cue mai visto: 0).
//! - **L6.5**: milli-reward interi (1000 corretta / 0 errata); metrica di
//!   run = somma dei milli-reward sulle 40 prove di test.
//! - **L6.7**: Wilcoxon signed-rank appaiato una coda («greater»): zeri
//!   scartati, ranghi dei valori assoluti con tie-break = rango medio,
//!   W = somma dei ranghi positivi, approssimazione normale con correzione
//!   di continuità, `p = 0.5·erfc(z/√2)` con erfc in casa (A&S 7.1.26).
//!   Float SOLO nell'output `WilcoxonResult` — mai nei payload.
//! - **L6.9**: `fold_run` ricalcola i reward da un log ricostruito
//!   (`to_jsonl → from_jsonl → verify → fold`): replay bit-perfetto.
//! - **Frontiera draw**: i run NON consumano draw PCG64 (la generazione
//!   offline usa il PRNG dedicato del seed); il budget L0 §4 resta intatto.
//! - **Zero I/O, zero wall-clock**: l'harness (`src/bin/l6_experiment.rs`)
//!   li detiene; questo modulo è matematica pura.

use crate::event::EventType;
use crate::log::Log;
use crate::payload::{Payload, Value};
use crate::prng::Pcg64;

/// Dimensione del pool di cue per run (L6.3).
pub const CUES: usize = 12;
/// Task di apprendimento per run (L6.3, firma 80+40).
pub const LEARNING_TASKS: usize = 80;
/// Prove di test per run (L6.3, firma 80+40).
pub const TEST_TASKS: usize = 40;
/// Task totali per run (L6.3).
pub const TASKS_PER_RUN: usize = LEARNING_TASKS + TEST_TASKS;
/// Milli-reward per risposta corretta (L6.5).
pub const REWARD_MILLI: i64 = 1000;

/// Errore del valutatore (pattern L0–L5: errore esplicito, mai panic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValuationError {
    /// Wilcoxon senza coppie non-zero: nessuna statistica definita (L6.7).
    NoPairs,
    /// Il log non è un run valido: forme deviate (chiavi extra, campioni
    /// fuori range, stream non alternato) o numero di task ≠ 120 (L6.2,
    /// L6.6, L6.9).
    Protocol,
}

/// Un task del piano: il cue proposto e la classe giusta rivelata (L6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// Etichetta del cue dal pool `cue:00`…`cue:11`.
    pub cue: String,
    /// Classe giusta secondo la regola nascosta (0|1|2).
    pub giusta: i64,
}

/// Il piano offline di un run: regola nascosta + lista di task (L6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskPlan {
    /// Seed del run (1000..=1099 nel corpus).
    pub seed: u64,
    /// Tabella `cue index → classe giusta`, bilanciata 4+4+4.
    pub rule: [i64; CUES],
    /// I 120 task (80 apprendimento + 40 test) nella stessa lista per
    /// entrambe le braccia (appaiamento L6.3).
    pub tasks: Vec<Task>,
}

/// Il braccio di un run (L6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm {
    /// Consulta il proprio log per replay (maggioranza, tie → classe minore).
    Memoria,
    /// Politica memorialess: risponde sempre 0.
    Controllo,
}

/// Esito di un braccio: log JSONL + reward diagnostici (L6.5, L6.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmOutput {
    /// Il log del braccio in forma JSONL canonica.
    pub jsonl: String,
    /// Somma dei milli-reward sugli 80 task di apprendimento (diagnostica).
    pub learning_milli: i64,
    /// Somma dei milli-reward sulle 40 prove di test (criterio L6.5).
    pub test_milli: i64,
}

/// Esito di una coppia appaiata (L6.6, L6.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairResult {
    pub memoria: ArmOutput,
    pub controllo: ArmOutput,
    /// Replay bit-perfetto attestato per entrambe le braccia (L6.9).
    pub replay_ok: bool,
}

/// Risultato del Wilcoxon appaiato una coda (L6.7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WilcoxonResult {
    /// Numero di coppie non-zero.
    pub n: usize,
    /// Somma dei ranghi (rango medio per i tie) delle differenze positive.
    pub w: f64,
    /// p-value una coda «memoria > controllo».
    pub p: f64,
    /// Media delle differenze in unità reward (milli/1000).
    pub delta_bar: f64,
}

// ---------------------------------------------------------------------------
// L6.3 — generazione offline appaiata
// ---------------------------------------------------------------------------

/// Etichetta canonica del cue di indice `idx` (L6.3: `cue:00`…`cue:11`).
fn cue_label(idx: usize) -> String {
    format!("cue:{idx:02}")
}

/// Genera il piano offline di un run dal seed — L6.3.
///
/// Una sola volta per coppia, FUORI dai run (zero draw nei run): la tabella
/// della regola nasce bilanciata (4 cue per classe) e viene mescolata con
/// Fisher–Yates sui draw PCG64 del seed; gli 80 task di apprendimento pescano
/// cue uniformi; le 40 prove di test = 3 passate complete sui 12 cue
/// (ogni cue coperto esattamente 3 volte) + 4 estrazioni extra dal PRNG.
/// Stesso seed → stessa tabella e stessa lista, byte per byte.
pub fn generate_pair(seed: u64) -> TaskPlan {
    let mut rng = Pcg64::new(seed);
    // tabella bilanciata: 4 cue per classe, poi Fisher–Yates
    let mut rule = [0i64; CUES];
    for (i, c) in rule.iter_mut().enumerate() {
        *c = (i / 4) as i64;
    }
    for i in (1..CUES).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        rule.swap(i, j);
    }
    let mut tasks: Vec<Task> = Vec::with_capacity(TASKS_PER_RUN);
    // apprendimento: cue uniformi dal PRNG
    for _ in 0..LEARNING_TASKS {
        let idx = (rng.next_u64() % CUES as u64) as usize;
        tasks.push(Task { cue: cue_label(idx), giusta: rule[idx] });
    }
    // test: 3 passate complete sui 12 cue + 4 estrazioni extra (40 = 36+4)
    for _ in 0..3 {
        for idx in 0..CUES {
            tasks.push(Task { cue: cue_label(idx), giusta: rule[idx] });
        }
    }
    for _ in 0..(TEST_TASKS - 3 * CUES) {
        let idx = (rng.next_u64() % CUES as u64) as usize;
        tasks.push(Task { cue: cue_label(idx), giusta: rule[idx] });
    }
    TaskPlan { seed, rule, tasks }
}

// ---------------------------------------------------------------------------
// L6.4 — le due braccia
// ---------------------------------------------------------------------------

/// Previsione del braccio memoria per replay del proprio log — L6.4.
///
/// Conta le coppie (cue, esito) già percepite nel log (dopo la genesi), per
/// il cue dato; previsione = classe col conteggio massimo, parità → classe
/// minore, cue mai visto → classe 0. Il conteggio parte SEMPRE dal log: la
/// memoria è il log (nessun stato separato).
fn predict_from_log(log: &Log, cue: &str) -> i64 {
    let mut counts = [0i64; 3];
    let mut pending: Option<String> = None;
    for e in log.events().iter().skip(1) {
        match (e.r#type, e.payload.get("cue"), e.payload.get("esito")) {
            (EventType::Sense, Some(Value::Str(c)), _) => pending = Some(c.clone()),
            (EventType::Sense, _, Some(Value::Int(esito))) => {
                if let Some(c) = pending.take() {
                    if c == cue && (*esito as usize) < 3 {
                        counts[*esito as usize] += 1;
                    }
                }
            }
            _ => {}
        }
    }
    // argmax con tie-break = classe minore (scandisce 0,1,2 con > stretto)
    let mut best = 0usize;
    for k in 1..3 {
        if counts[k] > counts[best] {
            best = k;
        }
    }
    best as i64
}

/// Esegue un braccio su un piano e produce il log + i reward — L6.4, L6.5.
///
/// Pura: zero draw, zero I/O, zero wall-clock. Flusso per task: SENSE
/// `{"cue"}` → decisione del braccio → ACTION `{"mossa" 0..2}` → SENSE
/// `{"esito" 0..2}` (la classe giusta rivelata, simmetrica per entrambe le
/// braccia). Tick = indice del task (contatore logico, pattern L0).
pub fn run_arm(plan: &TaskPlan, arm: Arm) -> Result<ArmOutput, ValuationError> {
    // validazione difensiva del piano (L6.6: 120 task; regola 0..2)
    if plan.tasks.len() != TASKS_PER_RUN || plan.rule.iter().any(|c| *c < 0 || *c > 2) {
        return Err(ValuationError::Protocol);
    }
    let mut log = Log::genesis(plan.seed).map_err(|_| ValuationError::Protocol)?;
    let mut learning_milli = 0i64;
    let mut test_milli = 0i64;
    for (t, task) in plan.tasks.iter().enumerate() {
        let tick = t as u64 + 1;
        log.append(
            tick,
            EventType::Sense,
            Payload::from_pairs([("cue", Value::from(task.cue.as_str()))]),
        )
        .map_err(|_| ValuationError::Protocol)?;
        let mossa = match arm {
            Arm::Controllo => 0i64,
            Arm::Memoria => predict_from_log(&log, &task.cue),
        };
        log.append(
            tick,
            EventType::Action,
            Payload::from_pairs([("mossa", Value::from(mossa))]),
        )
        .map_err(|_| ValuationError::Protocol)?;
        log.append(
            tick,
            EventType::Sense,
            Payload::from_pairs([("esito", Value::from(task.giusta))]),
        )
        .map_err(|_| ValuationError::Protocol)?;
        let reward = if mossa == task.giusta { REWARD_MILLI } else { 0 };
        if t < LEARNING_TASKS {
            learning_milli += reward;
        } else {
            test_milli += reward;
        }
    }
    Ok(ArmOutput { jsonl: log.to_jsonl(), learning_milli, test_milli })
}

/// Esegue la coppia appaiata completa di un seed — L6.6, L6.9.
///
/// Le due braccia attraversano la STESSA lista di task del piano (stesso
/// stream percettivo). `replay_ok` attesta il replay bit-perfetto per
/// entrambe le braccia: `from_jsonl → verify() → to_jsonl` identità e
/// `fold_run` con gli stessi reward byte per byte (L6.9).
pub fn run_pair(seed: u64) -> PairResult {
    let plan = generate_pair(seed);
    let memoria = run_arm(&plan, Arm::Memoria).expect("piano valido: braccio memoria");
    let controllo = run_arm(&plan, Arm::Controllo).expect("piano valido: braccio controllo");
    let replay_ok = [&memoria, &controllo].iter().all(|out| {
        let Ok(log) = Log::from_jsonl(&out.jsonl) else { return false };
        if log.verify().is_err() {
            return false;
        }
        match fold_run(&log) {
            Ok((learn, test)) => {
                learn == out.learning_milli && test == out.test_milli && log.to_jsonl() == out.jsonl
            }
            Err(_) => false,
        }
    });
    PairResult { memoria, controllo, replay_ok }
}

// ---------------------------------------------------------------------------
// L6.9 — fold e replay del log di un run
// ---------------------------------------------------------------------------

/// Ricalcola i reward di un run dal suo log — L6.9.
///
/// Il log valido di un run è: genesi + 120 task nella forma ESATTA L6.2
/// (`SENSE {"cue": Str}` → `ACTION {"mossa": Int 0..2}` → `SENSE
/// {"esito": Int 0..2}`, tre eventi per task, stesso tick logico). Chiavi
/// extra, campioni fuori range, sequenza non alternata o numero di task
/// diverso da 120 sono violazioni di protocollo (`ValuationError::Protocol`,
/// L6.2). Restituisce `(learning_milli, test_milli)`.
pub fn fold_run(log: &Log) -> Result<(i64, i64), ValuationError> {
    let mut learning_milli = 0i64;
    let mut test_milli = 0i64;
    let mut n_tasks = 0usize;
    let mut mossa: Option<i64> = None;
    let mut attesa_cue = true;
    for e in log.events().iter().skip(1) {
        // forma chiusa: esattamente UNA chiave tra cue/mossa/esito
        let (cue, act, esito) = (
            e.payload.get("cue"),
            e.payload.get("mossa"),
            e.payload.get("esito"),
        );
        match (e.r#type, e.payload.len(), cue, act, esito) {
            (EventType::Sense, 1, Some(Value::Str(_)), None, None) if attesa_cue => {
                attesa_cue = false;
            }
            (EventType::Action, 1, None, Some(Value::Int(m)), None) if !attesa_cue => {
                if *m < 0 || *m > 2 {
                    return Err(ValuationError::Protocol);
                }
                mossa = Some(*m);
            }
            (EventType::Sense, 1, None, None, Some(Value::Int(es))) if !attesa_cue => {
                let Some(m) = mossa.take() else { return Err(ValuationError::Protocol) };
                if *es < 0 || *es > 2 {
                    return Err(ValuationError::Protocol);
                }
                let reward = if m == *es { REWARD_MILLI } else { 0 };
                if n_tasks < LEARNING_TASKS {
                    learning_milli += reward;
                } else {
                    test_milli += reward;
                }
                n_tasks += 1;
                attesa_cue = true;
            }
            _ => return Err(ValuationError::Protocol),
        }
    }
    if mossa.is_some() || !attesa_cue || n_tasks != TASKS_PER_RUN {
        return Err(ValuationError::Protocol);
    }
    Ok((learning_milli, test_milli))
}

// ---------------------------------------------------------------------------
// L6.7 — Wilcoxon appaiato una coda
// ---------------------------------------------------------------------------

/// `erfc(x)` con la formula 7.1.26 di Abramowitz–Stegun (errore ≤ 1.5e-7).
///
/// Per x ≥ 0: `erfc(x) = (a₁t + a₂t² + a₃t³ + a₄t⁴ + a₅t⁵)·e^(−x²)` con
/// `t = 1/(1+0.3275911·x)`. Per x < 0: `erfc(−x) = 2 − erfc(x)`.
fn erfc(x: f64) -> f64 {
    let ax = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * ax);
    let poly = ((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) * t
        + 0.254829592)
        * t;
    let tail = poly * (-ax * ax).exp();
    if x >= 0.0 { tail } else { 2.0 - tail }
}

/// Wilcoxon signed-rank appaiato, UNA CODA (alternativa «greater») — L6.7.
///
/// Input: le differenze memoria−controllo delle medie di test in
/// milli-reward, una per run. Zeri scartati; ranghi dei valori assoluti con
/// tie-break = rango medio; `W` = somma dei ranghi positivi;
/// approssimazione normale con correzione di continuità:
/// `z = (W − n(n+1)/4 − 0.5) / sqrt(n(n+1)(2n+1)/24)`;
/// `p = 0.5·erfc(z/√2)`; `delta_bar = media delle differenze / 1000` (unità
/// reward). Nessuna coppia non-zero → `ValuationError::NoPairs`.
pub fn wilcoxon_one_sided_greater(diffs: &[i64]) -> Result<WilcoxonResult, ValuationError> {
    let nz: Vec<i64> = diffs.iter().copied().filter(|d| *d != 0).collect();
    let n = nz.len();
    if n == 0 {
        return Err(ValuationError::NoPairs);
    }
    // ranghi dei valori assoluti con tie-break = rango medio
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| nz[i].unsigned_abs());
    let mut ranks = vec![0.0f64; n];
    let mut i = 0usize;
    while i < n {
        let mut j = i;
        while j < n && nz[order[j]].unsigned_abs() == nz[order[i]].unsigned_abs() {
            j += 1;
        }
        let avg = ((i + 1) as f64 + j as f64) / 2.0;
        for &k in &order[i..j] {
            ranks[k] = avg;
        }
        i = j;
    }
    let w: f64 = (0..n).filter(|k| nz[*k] > 0).map(|k| ranks[k]).sum();
    let nf = n as f64;
    let mu = nf * (nf + 1.0) / 4.0;
    let sigma = (nf * (nf + 1.0) * (2.0 * nf + 1.0) / 24.0).sqrt();
    let z = (w - mu - 0.5) / sigma;
    let p = 0.5 * erfc(z / 2.0f64.sqrt());
    let delta_bar = diffs.iter().sum::<i64>() as f64 / (diffs.len().max(1) as f64)
        / REWARD_MILLI as f64;
    Ok(WilcoxonResult { n, w, p, delta_bar })
}
