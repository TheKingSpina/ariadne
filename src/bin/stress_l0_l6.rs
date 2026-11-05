//! Stress test L0–L6, manifest v1 (5 novembre 2026).
//!
//! Costruisce un unico log L0–L5 deterministico con il profilo AUX v2
//! ratificato, verifica le posizioni e gli stati attesi, ricostruisce il
//! replay e scrive `runs/stress_l0_l6/main.jsonl`. L6 è eseguito dal binario
//! `l6_experiment` e resta un esperimento separato.
//!
//! Questo manifest è stato compilato il 5 novembre 2026 sulla base delle
//! decisioni ratificate; non è il vecchio `manifest_stress_L0_L6_bozza.md`.

use std::fs;
use std::path::Path;

use ariadne::aux_v2::{Fixture, Json, MemoryStatus, V2Log};
use sha2::{Digest, Sha256};

const NODE_ALPHA: &str = "c735556705ac4d4485024861e77e1c1b2d569542c8d5a4359f5ed96334aa8feb";
const NODE_BETA: &str = "45aba7c6bf7186d413becacb7f88e22665b4b50b25829e668df7435020664ee0";
const AGENT_ALPHA: &str = "b7b51403fc2c3c26314fc288bfa3b12b381deeed809870d5b78fca03c8784af3";
const AGENT_BETA: &str = "8626c5c95ee85f1a1e9eb37d0059b9b7d33a3a9c1d2ef00e28a970f605d6376f";

const F_GENESIS: &str = r#"{"observations":[{"agente":"agente_alpha","confidenza":1000,"ordine_tom":0,"proposizione":"1","soggetto":"agente_alpha","sources":[0]}]}"#;
const F_BETA: &str = r#"{"observations":[{"agente":"agente_beta","confidenza":1000,"ordine_tom":0,"proposizione":"2","soggetto":"agente_beta","sources":[0]}]}"#;
const F_ALPHA2: &str = r#"{"observations":[{"agente":"agente_alpha","confidenza":1000,"ordine_tom":0,"proposizione":"2","soggetto":"agente_alpha","sources":[0]}]}"#;
const F_PERSPECTIVE: &str = r#"{"observations":[{"agente":"agente_alpha","confidenza":1000,"ordine_tom":1,"proposizione":"2","soggetto":"agente_beta","sources":[0]}]}"#;

const D_GENESIS: &str = "c66da3195548417fcbcaea078e8bab66b98b539a68ddde6011223800c117fbac";
const D_BETA: &str = "b0a0fdb92c0dfbe7bfaf9cd56bbbd78e4cfef58378aeacc4037036a975fa77c5";
const D_ALPHA2: &str = "7d4d7aa8059739b6b4c09c15a565254a05b208e662f51b3ee31f81307688f099";
const D_PERSPECTIVE: &str = "9a5b3c1cd79054c46f205684e034fc3c96248cf8dc58843f7fd793c76386cc79";

fn digest(text: &str) -> String {
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

fn data(event: &ariadne::aux_v2::Record) -> Json {
    Json::parse(event.payload.get("data").expect("data").string().expect("string")).expect("json")
}

fn aux_kind(event: &ariadne::aux_v2::Record) -> Option<&str> {
    event.payload.get("kind").ok()?.string().ok()
}

fn aux_level(event: &ariadne::aux_v2::Record) -> Option<&str> {
    event.payload.get("level").ok()?.string().ok()
}

fn main() {
    for (id, bytes, expected) in [
        ("l5-genesis-01", F_GENESIS, D_GENESIS),
        ("l5-beta-01", F_BETA, D_BETA),
        ("l5-alpha-02", F_ALPHA2, D_ALPHA2),
        ("l5-perspective-01", F_PERSPECTIVE, D_PERSPECTIVE),
    ] {
        assert_eq!(digest(bytes), expected, "digest fixture {id}");
    }

    let fixtures = vec![
        Fixture { id: "l5-genesis-01", sha256: D_GENESIS, bytes: F_GENESIS, anchor: 0 },
        Fixture { id: "l5-beta-01", sha256: D_BETA, bytes: F_BETA, anchor: 0 },
        Fixture { id: "l5-alpha-02", sha256: D_ALPHA2, bytes: F_ALPHA2, anchor: 0 },
        Fixture { id: "l5-perspective-01", sha256: D_PERSPECTIVE, bytes: F_PERSPECTIVE, anchor: 0 },
    ];

    let mut log = V2Log::genesis(1042).expect("genesis");
    log.configure(3, 5, 1, 10).expect("config L2/L3");

    // L5 sul tick 0, con ancora GENESIS. Le configurazioni non sono STATE.
    let obs_genesis = log.observe(&fixtures[0]).expect("osservazione genesi");
    assert_eq!(log.predict(AGENT_ALPHA).expect("predict alpha 1"), 1);
    let predict_1 = log.events().last().expect("predict 1").seq;
    let obs_agree = log.observe(&fixtures[0]).expect("osservazione accordo");
    let obs_beta = log.observe(&fixtures[1]).expect("osservazione beta");
    assert_eq!(log.predict(AGENT_ALPHA).expect("predict alpha 2"), 1);
    let predict_2 = log.events().last().expect("predict 2").seq;
    let obs_disagree = log.observe(&fixtures[2]).expect("osservazione disaccordo");
    let update = log.events().last().expect("model_update").seq;
    let obs_perspective = log.observe(&fixtures[3]).expect("osservazione prospettiva");

    // Tick 1: quattro nodi e un arco; select k=3.
    let tick1 = log.cycle().expect("ciclo 1");
    let extract1 = log
        .extract(
            tick1,
            &[("elemento", "alfa"), ("elemento", "beta"), ("elemento", "gamma"), ("elemento", "delta")],
            &[("elemento", "alfa", "elemento", "beta", "conosce")],
        )
        .expect("estrazione 1");
    log.close_selection_window(extract1).expect("select 1");

    // Tick 2: attivazione di alpha, senza promozione (T=1).
    let tick2 = log.cycle().expect("ciclo 2");
    let extract2 = log.extract(tick2, &[("elemento", "alfa")], &[]).expect("estrazione 2");
    log.close_selection_window(extract2).expect("select 2");
    log.evaluate_memory(extract2).expect("L3 tick 2");

    // Tick 3..6: al tick 6 alpha supera gli altri e gamma viene espulso.
    for _ in 3..6 { log.cycle().expect("ciclo intermedio"); }
    let tick6 = log.cycle().expect("ciclo 6");
    let extract6 = log.extract(tick6, &[("elemento", "alfa")], &[]).expect("estrazione 6");
    log.close_selection_window(extract6).expect("select 6");
    log.evaluate_memory(extract6).expect("L3 tick 6");

    // Fino al tick 256 per portare l'età L3 a 25 con H=10.
    for _ in 7..256 { log.cycle().expect("ciclo lungo"); }
    let tick256 = log.cycle().expect("ciclo 256");
    let extract256 = log.extract(tick256, &[("elemento", "beta")], &[]).expect("estrazione 256");
    log.close_selection_window(extract256).expect("select 256");
    log.evaluate_memory(extract256).expect("L3 tick 256");

    // Tick 257: recall di alpha.
    let tick257 = log.cycle().expect("ciclo 257");
    let extract257 = log.extract(tick257, &[("elemento", "alfa")], &[]).expect("estrazione 257");
    log.close_selection_window(extract257).expect("select 257");
    log.evaluate_memory(extract257).expect("L3 tick 257");

    let jsonl = log.to_jsonl();
    let replay = V2Log::from_jsonl(&jsonl, "2", &fixtures).expect("replay v2");
    assert_eq!(replay.to_jsonl(), jsonl, "replay byte-identico");

    // Posizioni e transizioni attese.
    assert_eq!(obs_genesis, 3);
    assert_eq!(predict_1, 4);
    assert_eq!(obs_agree, 5);
    assert_eq!(obs_beta, 6);
    assert_eq!(predict_2, 7);
    assert_eq!(obs_disagree, 8);
    assert_eq!(update, 9, "model_update dopo l'osservazione in disaccordo");
    assert_eq!(obs_perspective, 10);

    let summaries: Vec<_> = log.events().iter().filter(|e| aux_level(e) == Some("L2") && aux_kind(e) == Some("summary")).collect();
    assert_eq!(summaries.len(), 5, "una summary per select");
    let first = data(summaries[0]);
    let second = data(summaries[1]);
    assert_eq!(first.get("expulsions").expect("expulsions").canonical(), "[]");
    let expelled = second.get("expulsions").expect("expulsions").array().expect("array");
    assert_eq!(expelled.len(), 1, "gamma espulso al tick 2");
    assert_eq!(expelled[0].int().expect("seq"), tick1);

    let l3: Vec<_> = log.events().iter().filter(|e| aux_level(e) == Some("L3")).collect();
    let transition = |kind: &str, id: &str| l3.iter().any(|e| aux_kind(e) == Some(kind) && data(e).get("id").expect("id").string().expect("id") == id);
    assert!(transition("promote", NODE_ALPHA), "promote alpha");
    assert!(transition("promote", NODE_BETA), "promote beta al tick 256");
    assert!(transition("archive", NODE_ALPHA), "archive alpha");
    assert!(transition("recall", NODE_ALPHA), "recall alpha");
    assert_eq!(log.memory().get(NODE_ALPHA), Some(&MemoryStatus::Circuit));
    assert_eq!(log.memory().get(NODE_BETA), Some(&MemoryStatus::Long));
    assert!(log.active().contains(NODE_ALPHA));

    let graph = log.graph_document();
    assert_eq!(graph.get("nodes").expect("nodes").array().expect("nodes").len(), 4);
    assert_eq!(graph.get("edges").expect("edges").array().expect("edges").len(), 1);
    let prediction = log.project_prediction(AGENT_ALPHA, AGENT_BETA).expect("secondo ordine");
    assert_eq!(prediction.get("previsto").expect("previsto").int().expect("int"), 2);

    let dir = Path::new("runs/stress_l0_l6");
    fs::create_dir_all(dir).expect("creazione runs/stress_l0_l6");
    fs::write(dir.join("main.jsonl"), &jsonl).expect("scrittura log");
    for (name, bytes) in [("l5-genesis-01.json", F_GENESIS), ("l5-beta-01.json", F_BETA),
                          ("l5-alpha-02.json", F_ALPHA2), ("l5-perspective-01.json", F_PERSPECTIVE)] {
        fs::write(dir.join(name), bytes).expect("scrittura fixture");
    }

    let summary = Json::object([
        ("scenario", "stress_l0_l6_v1".into()),
        ("seed", 1042.into()),
        ("events", (log.events().len() as i64).into()),
        ("head_hash", log.events().last().expect("head").hash.clone().into()),
        ("log_sha256", digest(&jsonl).into()),
        ("fixtures", (fixtures.len() as i64).into()),
        ("l5_observations", obs_perspective.into()),
        ("expelled_source_seq", tick1.into()),
        ("node_alpha", NODE_ALPHA.into()),
        ("node_beta", NODE_BETA.into()),
        ("prediction_beta", 2.into()),
    ]);
    fs::write(dir.join("summary.json"), summary.canonical() + "\n").expect("scrittura summary");
    println!("{}", summary.canonical());
}
