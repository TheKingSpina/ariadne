//! L5 — legge sociale (modelli degli altri e teoria della mente):
//! test della legge `notes/L5_legge_sociale.md` (commit `23a7104`), checklist 1–8.
//!
//! Scelte dichiarate (report del producer):
//! - eventi `l5` e SENSE con `osservazioni` apposti MANUALMENTE con payload L0
//!   (`Log::append` + `Payload::from_pairs`), NON via `encode5`/`encode5_predict`
//!   (nessuna dipendenza circolare dei test; l'append resta sempre al
//!   chiamante, L5.9/L5.10);
//! - `id_altro` calcolato NEI TEST come SHA-256 del canone `{"altro":"<label>"}`
//!   (L5.3), senza dipendere dal codice (le label di test non usano escape);
//! - le viste si confrontano per uguaglianza strutturale (`PartialEq` come
//!   L4), con la vista INTERA mai assunta iniettiva fra log diversi.

use ariadne::{EventType, Log, Payload, Value};
use ariadne::social::{build_social, evaluate_social, predict_social, Decision5, SocialBase, SocialError};
use ariadne::identity::{build_self, evaluate_self, SelfBase};
use ariadne::memory;
use ariadne::prng::Pcg64;
use ariadne::selection;
use ariadne::cycle;
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// helper di costruzione
// ---------------------------------------------------------------------------

fn sha256_hex(s: &str) -> String {
    Sha256::digest(s.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// id canonico dell'altro (L5.3): SHA-256 minuscolo del canone `{"altro":"<label>"}`.
fn id_altro(label: &str) -> String {
    sha256_hex(&format!("{{\"altro\":\"{label}\"}}"))
}

/// membro osservazione canonico (L5.2): `{"altro":"<label>","comportamento":<n>}`
/// con chiavi in ordine lessicografico.
fn obs_member(label: &str, comportamento: i64) -> String {
    format!("{{\"altro\":\"{label}\",\"comportamento\":{comportamento}}}")
}

/// array F1 canonico di membri (L5.2).
fn osservazioni_json(members: &[String]) -> String {
    let mut out = String::from("[");
    for (i, m) in members.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(m);
    }
    out.push(']');
    out
}

/// SENSE con `osservazioni` (L5.2): UN evento con l'array F1 in log.
fn append_obs(log: &mut Log, tick: u64, members: &[String]) {
    log.append(tick, EventType::Sense, Payload::from_pairs([(
        "osservazioni",
        Value::Str(osservazioni_json(members)),
    )])).unwrap();
}

/// payload `l5="predict"` (L5.10): chiavi esatte `l5, at, id, previsto, sources`.
fn predict_pairs(at: u64, id: &str, previsto: i64, sources: &str) -> Payload {
    Payload::from_pairs([
        ("l5", Value::Str("predict".into())),
        ("at", Value::Int(at as i64)),
        ("id", Value::Str(id.to_string())),
        ("previsto", Value::Int(previsto)),
        ("sources", Value::Str(sources.to_string())),
    ])
}

/// base full-log (coerenza di prefisso, L5.10/L2.22 pattern).
fn base_full(log: &Log) -> SocialBase {
    SocialBase {
        head_hash: log.events().last().unwrap().hash.clone(),
        log_len: log.events().len(),
    }
}

// ---------------------------------------------------------------------------
// checklist 1 — replay byte per byte / genesi
// ---------------------------------------------------------------------------

// [checklist 1 / L5.5] log di sola genesi: `models` vuoto, `head_hash`
// = hash dell'evento genesi; array `osservazioni` vuoto ammesso, senza
// effetto statistico (L5.2).
#[test]
fn l5_check1_genesis_only_models_empty_head_hash_is_genesis() {
    let log = Log::genesis(1042).unwrap();
    let v = build_social(&log).unwrap();
    assert!(v.models.is_empty());
    assert_eq!(v.head_hash, log.events()[0].hash);

    // SENSE con `osservazioni = []` (L5.2): ammesso, nessun modello.
    let mut log2 = Log::genesis(1042).unwrap();
    append_obs(&mut log2, 1, &[]);
    let v2 = build_social(&log2).unwrap();
    assert!(v2.models.is_empty());
    assert_eq!(v2.head_hash, log2.events().last().unwrap().hash);
}

// [checklist 1 / L5.5] due `build_social` sul medesimo log → viste
// identiche byte per byte (replay deterministico).
#[test]
fn l5_check1_replay_is_byte_per_byte() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1), obs_member("alice", 3)]);
    append_obs(&mut log, 2, &[obs_member("bob", 2)]);
    let v1 = build_social(&log).unwrap();
    let v2 = build_social(&log).unwrap();
    assert_eq!(v1, v2);
}

// ---------------------------------------------------------------------------
// checklist 2 — frontiera intersoggettiva (L5.9)
// ---------------------------------------------------------------------------

// [checklist 2 / L5.9] NESSUNA API L5 scrive nel log; l'append è SOLO via
// API L0 dal chiamante; ogni vista riscontra head_hash + `log.verify()`.
#[test]
fn l5_check2_frontiera_no_l5_writes_in_log() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);
    log.append(2, EventType::Action, predict_pairs(1, &id_altro("bob"), 1, "[1]"))
        .unwrap();
    let before_len = log.events().len();
    let jsonl_before = log.to_jsonl();

    // le tre API L5: zero scritture, zero side-effect sulla catena
    let _view = build_social(&log).unwrap();
    let base = base_full(&log);
    let _pred = predict_social(&log, &base, &id_altro("bob")).unwrap();
    let _dec = evaluate_social(&log, &base).unwrap();

    assert_eq!(log.events().len(), before_len);
    assert_eq!(log.to_jsonl(), jsonl_before);
    log.verify().unwrap();

    let view = build_social(&log).unwrap();
    assert_eq!(view.head_hash, log.events().last().unwrap().hash);
}

// ---------------------------------------------------------------------------
// checklist 3 — previsione ancorata (L5.6)
// ---------------------------------------------------------------------------

// [checklist 3 / L5.6] `sources` = seq osservate dell'id nel prefisso,
// dedup crescente; `NoModel` per id assente (zero osservazioni).
#[test]
fn l5_check3_predict_source_seqs_and_no_model() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);                          // seq 1
    append_obs(&mut log, 2, &[obs_member("alice", 5)]);                       // seq 2
    append_obs(&mut log, 3, &[obs_member("bob", 2), obs_member("alice", 5)]); // seq 3
    append_obs(&mut log, 4, &[obs_member("alice", 5)]);                       // seq 4

    let base = base_full(&log);
    let p = predict_social(&log, &base, &id_altro("bob")).unwrap();
    // bibliografia dell'ancora: solo le seq con osservazioni di bob
    assert_eq!(p.sources, vec![1, 3]);
    assert_eq!(p.id, id_altro("bob"));
    // classi di bob {1:1, 2:1} — parità → classe MINORE vince (L5.6)
    assert_eq!(p.previsto, 1);

    // id con zero osservazioni nel prefisso → NoModel (errore PROPRIO)
    assert_eq!(
        predict_social(&log, &base, &id_altro("carol")).unwrap_err(),
        SocialError::NoModel
    );

    // alice: classi {5:3} → previsto 5, sources [2,3,4]
    let pa = predict_social(&log, &base, &id_altro("alice")).unwrap();
    assert_eq!(pa.previsto, 5);
    assert_eq!(pa.sources, vec![2, 3, 4]);
}

// ---------------------------------------------------------------------------
// checklist 4 — divergenza scoped (L5.5, Q10)
// ---------------------------------------------------------------------------

// [checklist 4(i)] log che differiscono SOLO per eventi irrilevanti per il
// modello di bob (osservazioni di un ALTRO id) → il modello di bob resta
// IDENTICO a livello di `models`. NESSUNA assunzione di iniettività
// universale: le viste INTERE restano diverse (alice + head_hash).
#[test]
fn l5_check4_divergenza_irrelevant_events_leave_model_identical() {
    let mut loga = Log::genesis(1042).unwrap();
    append_obs(&mut loga, 1, &[obs_member("bob", 1)]);
    append_obs(&mut loga, 2, &[obs_member("bob", 1)]);
    let mut logb = Log::genesis(1042).unwrap();
    append_obs(&mut logb, 1, &[obs_member("bob", 1)]);
    append_obs(&mut logb, 2, &[obs_member("bob", 1)]);
    append_obs(&mut logb, 3, &[obs_member("alice", 9)]); // irrilevante per bob

    let va = build_social(&loga).unwrap();
    let vb = build_social(&logb).unwrap();

    // sotto-modello bob: IDENTICO
    assert_eq!(va.models.get(&id_altro("bob")), vb.models.get(&id_altro("bob")));

    // le viste INTERE NON coincidono
    assert!(vb.models.contains_key(&id_altro("alice")));
    assert_ne!(va.head_hash, vb.head_hash);
}

// [checklist 4(ii)] un'osservazione rilevante di X cambia modello e
// contatori di X (e la vista intera via head_hash).
#[test]
fn l5_check4_divergenza_relevant_observation_changes_model() {
    let mut logx = Log::genesis(1042).unwrap();
    append_obs(&mut logx, 1, &[obs_member("bob", 1)]);
    let mut logy = Log::genesis(1042).unwrap();
    append_obs(&mut logy, 1, &[obs_member("bob", 1)]);
    append_obs(&mut logy, 2, &[obs_member("bob", 1)]); // osservazione rilevante di bob

    let vx = build_social(&logx).unwrap();
    let vy = build_social(&logy).unwrap();
    let mx = vx.models.get(&id_altro("bob")).unwrap();
    let my = vy.models.get(&id_altro("bob")).unwrap();
    assert_ne!(mx.n, my.n);                 // 1 vs 2
    assert_ne!(mx.contatori, my.contatori); // {1:1} vs {1:2}
    assert_ne!(vx.head_hash, vy.head_hash); // catena L0 propaga la differenza
}

// ---------------------------------------------------------------------------
// checklist 5 — secondo ordine (L5.8): composizione identità
// ---------------------------------------------------------------------------

// [checklist 5 / L5.8] la composizione (A-su-T su U) è, per definizione,
// la STESSA funzione di previsione diretta su T: asserto byte per byte
// sulle componenti della `Prediction`; nessuna via alternativa.
#[test]
fn l5_check5_second_order_composition_equals_direct_prediction() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);
    append_obs(&mut log, 2, &[obs_member("bob", 3)]);
    append_obs(&mut log, 3, &[obs_member("bob", 3)]);
    append_obs(&mut log, 4, &[obs_member("bob", 1)]);

    let base = base_full(&log);
    // previsione diretta su bob
    let direct = predict_social(&log, &base, &id_altro("bob")).unwrap();

    // composizione A-su-bob (L5.8: identità): la previsione composta è
    // l'argmax dei contatori dello stesso modello T costruito dal log
    let view = build_social(&log).unwrap();
    let m = view.models.get(&id_altro("bob")).unwrap();
    let mut composed: Option<i64> = None;
    let mut best: Option<u64> = None;
    for (classe, conteggio) in m.contatori.iter() {
        match best {
            None => {
                composed = Some(*classe);
                best = Some(*conteggio);
            }
            Some(b) if *conteggio > b => {
                composed = Some(*classe);
                best = Some(*conteggio);
            }
            _ => {}
        }
    }
    assert_eq!(composed, Some(direct.previsto));
    // la bibliografia composta è la stessa funzione: seq osservate di T
    assert_eq!(direct.sources, vec![1, 2, 3, 4]);
}

// ---------------------------------------------------------------------------
// checklist 6 — errori precisi e non-panic (L5.1/L5.2/L5.6/L5.10)
// ---------------------------------------------------------------------------

// [checklist 6 / L5.10–L5.6] payload `l5` malformati (chiavi extra,
// chiavi assenti, id non hex64, `at` che non riferisce seq esistente,
// dominio fuori chiusa) → Schema.
#[test]
fn l5_check6_malformed_l5_payload_is_schema() {
    let idb = id_altro("bob");
    // chiave EXTRA nel predict
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);
    log.append(2, EventType::Action, Payload::from_pairs([
        ("l5", Value::Str("predict".into())),
        ("at", Value::Int(1)),
        ("id", Value::Str(idb.clone())),
        ("previsto", Value::Int(1)),
        ("sources", Value::Str("[1]".into())),
        ("extra", Value::Str("no".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // chiave ASSENTE nel model_update (manca `osservato`)
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);
    log.append(2, EventType::Action, Payload::from_pairs([
        ("l5", Value::Str("model_update".into())),
        ("at", Value::Int(1)),
        ("id", Value::Str(idb.clone())),
        ("previsto", Value::Int(1)),
        ("sources", Value::Str("[1,2]".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // id non hex64 nel payload
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);
    log.append(2, EventType::Action, predict_pairs(1, "bob-nonhex", 1, "[1]")).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // `at` che NON riferisce seq esistente
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);
    log.append(2, EventType::Action, predict_pairs(99, &idb, 1, "[1]")).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // valore `l5` fuori dominio chiuso (L5.1)
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Action, Payload::from_pairs([
        ("l5", Value::Str("quino".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);
}

// [checklist 6 / L5.2] `osservazioni` malformate (JSON non canonico, chiavi
// membro fuori ordine, chiave extra nel membro, label vuota, comportamento
// negativo) → Schema.
#[test]
fn l5_check6_malformed_osservazioni_is_schema() {
    // JSON non canonico (spazio dopo i due punti)
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Str(
            "[{\"altro\":\"bob\", \"comportamento\":1}]".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // chiavi membro NON in ordine lessicografico
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Str(
            "[{\"comportamento\":1,\"altro\":\"bob\"}]".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // chiave extra nel membro
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Str(
            "[{\"altro\":\"bob\",\"comportamento\":1,\"x\":2}]".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // label vuota
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("", 1)]);
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // comportamento negativo
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Str(
            "[{\"altro\":\"bob\",\"comportamento\":-1}]".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);
}

// [checklist 6 / L5.2 — rilievo R1 audit, `reports/audit_l5_impl.md`]
// la chiave `osservazioni` su SENSE dev'essere una STRINGA F1 canonica:
// valore NON-`Value::Str` (tipo errato) o stringa NON-F1 (JSON abbreviato /
// top-level sbagliato / spazi fuori canone) → `SocialError::Schema` — forma
// malformata (L5.2, righe 59–70), canone byte-per-byte L1/L2, specchio
// `src/selection.rs:47` e `src/identity.rs:283`.
#[test]
fn l5_check_osservazioni_non_f1_is_schema() {
    // valore NON-stringa: `Value::Int` — prima della patch veniva accettato
    // SILENZIOSAMENTE (R1): il test è ROSSO in commit 1 e VERDE dopo il fix.
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Int(5)),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);
    // evaluate_social rifiuta la STESSA forma (patch replicata su entrambi
    // i luoghi del modulo, social.rs:385 e :583)
    let base = base_full(&log);
    assert_eq!(evaluate_social(&log, &base).unwrap_err(), SocialError::Schema);

    // stringa NON-F1: array JSON abbreviato di stringhe (manca l'oggetto membro)
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Str("[\"bob\"]".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // stringa NON-F1: oggetto top-level (manca l'array)
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Str("{\"altro\":\"bob\"}".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // stringa F1 con SPACES: canone STRICT byte-per-byte (già L1/L2)
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("osservazioni", Value::Str("[ {\"altro\":\"bob\",\"comportamento\":1} ]".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);
}

// [checklist 6 / L5.2] la chiave `osservazioni` su un evento NON-SENSE →
// Schema (specchio della regola di patch L4: la chiave vive solo nel canale
// percettivo, mai sugli altri eventi).
#[test]
fn l5_check6_osservazioni_on_non_sense_is_schema() {
    let shape = "[{\"altro\":\"bob\",\"comportamento\":1}]";
    // ACTION con `osservazioni`
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Action, Payload::from_pairs([
        ("osservazioni", Value::Str(shape.into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);

    // STATE con `osservazioni`
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::State, Payload::from_pairs([
        ("osservazioni", Value::Str(shape.into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);
}

// [checklist 6 / L5.1] `l5` con `l2`/`l3`/`l4` assieme → Schema (domini
// chiusi mutualmente esclusivi); `l5` su evento non-ACTION → Schema.
#[test]
fn l5_check6_l5_mixed_with_other_tags_is_schema() {
    for extra in ["l2", "l3", "l4"] {
        let mut log = Log::genesis(1042).unwrap();
        log.append(1, EventType::Action, Payload::from_pairs([
            ("l5", Value::Str("predict".into())),
            (extra, Value::Str("x".into())),
        ])).unwrap();
        assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);
    }
    // `l5` su SENSE (non-ACTION)
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Sense, Payload::from_pairs([
        ("l5", Value::Str("predict".into())),
    ])).unwrap();
    assert_eq!(build_social(&log).unwrap_err(), SocialError::Schema);
}

// [checklist 6 / L5.10] base incoerente → HeadMatch (identico pattern L4.8).
#[test]
fn l5_check6_base_incoerente_is_head_match() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]);
    append_obs(&mut log, 2, &[obs_member("bob", 2)]);
    let good = base_full(&log);
    // head_hash sbagliato
    let bad_head = SocialBase { head_hash: "0".repeat(64), log_len: good.log_len };
    assert_eq!(evaluate_social(&log, &bad_head).unwrap_err(), SocialError::HeadMatch);
    assert_eq!(predict_social(&log, &bad_head, &id_altro("bob")).unwrap_err(), SocialError::HeadMatch);
    // log_len zero
    let zero = SocialBase { head_hash: good.head_hash.clone(), log_len: 0 };
    assert_eq!(evaluate_social(&log, &zero).unwrap_err(), SocialError::HeadMatch);
    // log_len oltre la lunghezza del log
    let too_long = SocialBase { head_hash: good.head_hash.clone(), log_len: good.log_len + 1 };
    assert_eq!(evaluate_social(&log, &too_long).unwrap_err(), SocialError::HeadMatch);
}

// [L5.7] finestra predict→osservazione: MISMATCH → UN update con
// at/tick/sources esatti (`sources` = [p, o] dedup crescente).
#[test]
fn l5_check7_window_update_mismatch_one_decision() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]); // seq 1
    let base = base_full(&log);
    log.append(10, EventType::Action, predict_pairs(1, &id_altro("bob"), 9, "[1]"))
        .unwrap(); // p = 2, previsto 9
    append_obs(&mut log, 11, &[obs_member("bob", 3)]); // o = 3, mismatch

    let dec = evaluate_social(&log, &base).unwrap();
    assert_eq!(dec.len(), 1);
    assert_eq!(dec[0], Decision5::ModelUpdate {
        at: 3,
        tick: 11,
        id: id_altro("bob"),
        previsto: 9,
        osservato: 3,
        sources: vec![2, 3],
    });
}

// [L5.7] osservazione UGUALE alla previsione → NESSUN update; la finestra è
// comunque consumata: l'osservazione successiva senza predict non riapre.
#[test]
fn l5_check7_window_equal_observation_no_update() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]); // seq 1
    let base = base_full(&log);
    log.append(10, EventType::Action, predict_pairs(1, &id_altro("bob"), 3, "[1]"))
        .unwrap(); // p = 2, previsto 3
    append_obs(&mut log, 11, &[obs_member("bob", 3)]); // o = 3, UGUALE
    append_obs(&mut log, 12, &[obs_member("bob", 1)]); // senza predict: nessuna finestra

    assert!(evaluate_social(&log, &base).unwrap().is_empty());
}

// [L5.7] un secondo predict SENZA osservazioni in mezzo SOSTITUISCE il primo
// come anchor della finestra: UNA sola decisione, del predict più recente.
#[test]
fn l5_check7_second_predict_replaces_anchor() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]); // seq 1
    let base = base_full(&log);
    log.append(10, EventType::Action, predict_pairs(1, &id_altro("bob"), 9, "[1]"))
        .unwrap(); // p = 2 (sostituito)
    log.append(11, EventType::Action, predict_pairs(1, &id_altro("bob"), 7, "[1]"))
        .unwrap(); // p = 3, anchor attuale
    append_obs(&mut log, 12, &[obs_member("bob", 2)]); // o = 4, mismatch

    let dec = evaluate_social(&log, &base).unwrap();
    assert_eq!(dec.len(), 1);
    assert_eq!(dec[0], Decision5::ModelUpdate {
        at: 4,
        tick: 12,
        id: id_altro("bob"),
        previsto: 7,
        osservato: 2,
        sources: vec![3, 4],
    });
}

// ---------------------------------------------------------------------------
// checklist 7 — cutoff L5.4 e interazioni
// ---------------------------------------------------------------------------

// [checklist 7 / L5.4] eventi `l5` nel prefisso della base → ZERO decisioni;
// solo le finestre con osservazione `o` > cutoff producono decisioni.
#[test]
fn l5_check7_l5_in_base_prefix_gives_zero_decisions() {
    let mut log = Log::genesis(1042).unwrap();
    append_obs(&mut log, 1, &[obs_member("bob", 1)]); // seq 1
    log.append(10, EventType::Action, predict_pairs(1, &id_altro("bob"), 9, "[1]"))
        .unwrap(); // seq 2: l5 nel prefisso
    append_obs(&mut log, 11, &[obs_member("bob", 3)]); // seq 3: mismatch pre-base
    let base = base_full(&log);
    // TUTTO nel prefisso: cutoff = seq base → zero decisioni
    assert!(evaluate_social(&log, &base).unwrap().is_empty());

    // nuova finestra oltre il cutoff (p = 4, o = 5): SOLO questa decide
    log.append(12, EventType::Action, predict_pairs(3, &id_altro("bob"), 0, "[1,3]"))
        .unwrap(); // p = 4, previsto 0
    append_obs(&mut log, 13, &[obs_member("bob", 5)]); // o = 5, mismatch

    let dec = evaluate_social(&log, &base).unwrap();
    assert_eq!(dec.len(), 1);
    assert_eq!(dec[0], Decision5::ModelUpdate {
        at: 5,
        tick: 13,
        id: id_altro("bob"),
        previsto: 0,
        osservato: 5,
        sources: vec![4, 5],
    });
}

// [checklist 7 / L5.4] chiusura finestra L2 da `l5`: pattern check6 L4 —
// append a finestra L2 aperta + `l5`, replay L2/L3/L4 intatti, nessun
// side-effect.
#[test]
fn l5_check7_l5_event_closes_l2_window_no_side_effects() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Action, Payload::from_pairs([
        ("l3", Value::Str("config".into())), ("T", Value::Int(0)), ("H", Value::Int(1)),
    ])).unwrap(); // seq 1: l3 config (controllo)
    log.append(2, EventType::Action, Payload::from_pairs([("l2", Value::Str("barrier".into()))]))
        .unwrap(); // seq 2: finestra L2 aperta
    log.append(3, EventType::Action, predict_pairs(0, &id_altro("bob"), 1, "[]"))
        .unwrap(); // seq 3: l5 — evento non-l2 → chiude la finestra (L2.25)

    // L2: replay sano, nessuna decisione indotta da l5
    let active = selection::build_active(&log).unwrap();
    assert!(selection::evaluate(&log, &active).unwrap().is_empty());

    // L3: replay sano, cutoff L3 non toccato da l5
    let mv = memory::build_memory(&log).unwrap();
    assert!(memory::evaluate(&log, &mv).unwrap().is_empty());

    // L4: intatto
    let sb = SelfBase {
        head_hash: log.events().last().unwrap().hash.clone(),
        log_len: log.events().len(),
    };
    assert!(build_self(&log).is_ok());
    assert!(evaluate_self(&log, &sb).unwrap().is_empty());

    // L5: il predict è forma-valida (at=0 esiste: genesi; sources [] canonico)
    let view = build_social(&log).unwrap();
    assert!(view.models.is_empty());
}

// ---------------------------------------------------------------------------
// checklist 8 — zero draw e determinismo (L5.11)
// ---------------------------------------------------------------------------

// [checklist 8 / L5.11] le API L5 non mutano il log (quindi né lo stato
// PRNG: nessun nuovo evento, nessun draw); due `evaluate_social` →
// decisioni identiche byte per byte.
#[test]
fn l5_check8_zero_draw_and_deterministic_decisions() {
    let mut log = Log::genesis(1042).unwrap();
    let mut rng = Pcg64::new(1042);
    cycle::run_cycles(&mut log, &mut rng, 2).unwrap(); // SENSE/ACTION/STATE ×2 → seq 0..6

    append_obs(&mut log, 10, &[obs_member("bob", 1)]); // seq 7
    append_obs(&mut log, 11, &[obs_member("bob", 2)]); // seq 8
    let base = base_full(&log); // log_len 9

    log.append(13, EventType::Action, predict_pairs(8, &id_altro("bob"), 0, "[7,8]"))
        .unwrap(); // p = 9, previsto 0
    append_obs(&mut log, 14, &[obs_member("bob", 9)]); // o = 10, mismatch

    let jsonl_before = log.to_jsonl();
    let len_before = log.events().len();

    let view1 = build_social(&log).unwrap();
    let view2 = build_social(&log).unwrap();
    assert_eq!(view1, view2);

    let d1 = evaluate_social(&log, &base).unwrap();
    let d2 = evaluate_social(&log, &base).unwrap();
    assert_eq!(d1, d2);
    assert_eq!(d1.len(), 1);
    assert_eq!(d1[0], Decision5::ModelUpdate {
        at: 10,
        tick: 14,
        id: id_altro("bob"),
        previsto: 0,
        osservato: 9,
        sources: vec![9, 10],
    });

    // nessuna mutazione del log dopo tutte le chiamate L5
    assert_eq!(log.events().len(), len_before);
    assert_eq!(log.to_jsonl(), jsonl_before);
    log.verify().unwrap();
}
