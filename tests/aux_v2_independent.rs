//! Oracolo indipendente AUX v2 — mandato WP3 (2026-11-05).
//!
//! Il fixture `preparazione_stress/fixtures/aux_v2_small/expected.jsonl`
//! è congelato a build dall'oracolo indipendente
//! `preparazione_stress/aux_v2_oracle.py` (solo standard library, nessun
//! import da ariadne, nessuna chiamata al backend Rust). Questo driver NON
//! lo rigenera: legge i byte dal disco, ne verifica il digest e li confronta
//! byte per byte con il log costruito tramite la sola API pubblica `V2Log`.
//!
//! I fixture negativi in `negative/` sono preregistrati con l'errore atteso;
//! qui `V2Log::from_jsonl` è usato solo come SUT per verificare che il replay
//! si fermi con l'errore congelato e che il log di riferimento resti invariato.

use ariadne::aux_v2::{Error, Json, V2Log};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

// Valori preregistrati nel mandato e nel manifest.json; NON rigenerati nel test.
const GOLDEN_DIGEST: &str = "690831b39906070c40eff25e64092d4c8f005a04d017ed4cb347d514f98cd0b4";
const GOLD_EVENTI: usize = 121;
const ALFA: &str = "c735556705ac4d4485024861e77e1c1b2d569542c8d5a4359f5ed96334aa8feb";
const BETA: &str = "45aba7c6bf7186d413becacb7f88e22665b4b50b25829e668df7435020664ee0";
const GOLD_GENESIS: &str = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("preparazione_stress/fixtures/aux_v2_small")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn data(log: &V2Log, seq: usize) -> Json {
    Json::parse(log.events()[seq].payload.get("data").unwrap().string().unwrap()).unwrap()
}

/// Il fixture congelato letto dal disco: se manca, o se il digest non coincide
/// con `expected.sha256` o con il preregistrato del mandato, il test fallisce
/// e NESSUN byte viene rigenerato qui.
fn frozen_fixture() -> String {
    let path = fixtures_dir().join("expected.jsonl");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("fixture congelato mancante {}: {e}", path.display())
    });
    let digest = sha256_hex(text.as_bytes());
    let recorded_path = fixtures_dir().join("expected.sha256");
    let recorded = std::fs::read_to_string(&recorded_path)
        .unwrap_or_else(|e| panic!("expected.sha256 mancante {}: {e}", recorded_path.display()));
    let recorded = recorded.trim();
    assert_eq!(digest, recorded, "digest del fixture non coincide con expected.sha256");
    assert_eq!(digest, GOLDEN_DIGEST, "fixture rigenerato dopo la congelazione: digest {digest} != preregistrato {GOLDEN_DIGEST}");
    text
}

/// Lo stesso scenario costruito SOLO con l'API pubblica, nell'ordine del
/// mandato. Nessun reference al fixture o all'oracolo per produrre l'atteso.
fn scenario_from_public_api() -> V2Log {
    let mut log = V2Log::genesis(1042).unwrap();
    log.configure(1, 5, 1, 1).unwrap();
    // tick 1: estrazione di alfa/beta con arco, finestra chiusa
    let source = log.cycle().unwrap();
    let e1 = log
        .extract(
            source,
            &[("elemento", "alfa"), ("elemento", "beta")],
            &[("elemento", "alfa", "elemento", "beta", "conosce")],
        )
        .unwrap();
    log.close_selection_window(e1).unwrap();
    // tick 2: estrazione di alfa, promozione L3, espulsione di beta
    let source = log.cycle().unwrap();
    let e2 = log.extract(source, &[("elemento", "alfa")], &[]).unwrap();
    log.evaluate_memory(e2).unwrap();
    log.close_selection_window(e2).unwrap();
    // tick 3..26: 24 cicli L0 puri
    for _ in 0..24 {
        log.cycle().unwrap();
    }
    // tick 27: estrazione di beta, archive di alfa e promote di beta
    let source = log.cycle().unwrap();
    let e27 = log.extract(source, &[("elemento", "beta")], &[]).unwrap();
    log.evaluate_memory(e27).unwrap();
    // tick 28: estrazione di alfa e recall
    let source = log.cycle().unwrap();
    let e28 = log.extract(source, &[("elemento", "alfa")], &[]).unwrap();
    log.evaluate_memory(e28).unwrap();
    log
}

#[test]
fn frozen_fixture_digest_is_preregistered() {
    let text = frozen_fixture();
    assert!(text.ends_with('\n'), "manca il LF finale nel fixture");
    assert!(!text.contains('\r'), "trovato CR nel fixture");
    let lines: Vec<&str> = text.strip_suffix('\n').unwrap().split('\n').collect();
    assert_eq!(lines.len(), GOLD_EVENTI, "il fixture ha {len} eventi, attesi {GOLD_EVENTI}", len = lines.len());
}

#[test]
fn reconstruction_from_public_api_matches_fixture_byte_per_byte() {
    let fixture = frozen_fixture();
    let log = scenario_from_public_api();
    assert_eq!(log.events().len(), GOLD_EVENTI);
    let rebuilt = log.to_jsonl();
    if rebuilt != fixture {
        // riporta la prima divergenza: discrepanza tra oracolo e API, da
        // notificare all'orchestratore (mai piegare l'oracolo all'implementazione)
        let rebuilt_lines: Vec<&str> = rebuilt.strip_suffix('\n').unwrap().split('\n').collect();
        let fixture_lines: Vec<&str> = fixture.strip_suffix('\n').unwrap().split('\n').collect();
        for (seq, (a, b)) in rebuilt_lines.iter().zip(fixture_lines.iter()).enumerate() {
            if a != b {
                panic!("divergenza byte per byte alla riga {seq} (seq evento {seq}):\n  API:      {a}\n  fixture:  {b}");
            }
        }
        assert_eq!(rebuilt_lines.len(), fixture_lines.len(), "divergenza di lunghezza");
    }
    assert_eq!(rebuilt, fixture);
}

#[test]
fn golden_points_from_mandate_hold_in_reconstruction() {
    let log = scenario_from_public_api();
    assert_eq!(log.events()[0].hash, GOLD_GENESIS, "hash genesi");
    assert_eq!(V2Log::node_id("elemento", "alfa"), ALFA);
    assert_eq!(V2Log::node_id("elemento", "beta"), BETA);
    // select tick 1: parità di score e origine, vince l'id minore (beta)
    assert_eq!(data(&log, 10).get("ups").unwrap().canonical(), format!("[\"{BETA}\"]"));
    assert_eq!(data(&log, 10).get("downs").unwrap().canonical(), "[]");
    assert_eq!(data(&log, 10).get("at").unwrap().canonical(), "6");
    // select tick 2: entrano alfa ed esce beta
    assert_eq!(data(&log, 25).get("ups").unwrap().canonical(), format!("[\"{ALFA}\"]"));
    assert_eq!(data(&log, 25).get("downs").unwrap().canonical(), format!("[\"{BETA}\"]"));
    assert_eq!(data(&log, 27).get("expulsions").unwrap().canonical(), "[3]");
    assert_eq!(data(&log, 27).get("text").unwrap().string().unwrap(), BETA);
    // L3: promote al tick 2, archive di alfa e promote di beta al tick 27, recall al 28
    let expected = [
        (21u64, "promote", ALFA, 20u64),
        (108, "archive", ALFA, 107),
        (111, "promote", BETA, 107),
        (118, "recall", ALFA, 117),
    ];
    for (seq, kind, id, at) in expected {
        assert_eq!(
            log.events()[seq as usize].payload.get("kind").unwrap().string().unwrap(),
            kind,
            "seq {seq}"
        );
        let d = data(&log, seq as usize);
        assert_eq!(d.get("at").unwrap().canonical(), format!("{at}"), "seq {seq} at");
        assert_eq!(d.get("id").unwrap().string().unwrap(), id, "seq {seq} id");
        assert_eq!(d.get("origin_seq").unwrap().canonical(), "3", "seq {seq} origin");
    }
}

#[test]
fn implementation_replay_accepts_the_independent_fixture() {
    // CONTROLLO di coerenza, non il giudizio indipendente: se il replay
    // rifiuta i byte congelati c'è una discrepanza da riportare e il test
    // di confronto byte per byte già la evidenzia con la riga esatta.
    let fixture = frozen_fixture();
    let result = V2Log::from_jsonl(&fixture, "2", &[]);
    assert_eq!(result.map(|log| log.events().len()), Ok(GOLD_EVENTI),
        "il replay rifiuta l'oracolo indipendente: discrepanza da notificare");
}

/// Tabella dei negativi preregistrati (file → errore congelato).
fn negative_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("select_ups_ricalcolata.jsonl", "Mismatch"),
        ("un_byte_alterato.jsonl", "Chain"),
        ("crlf.jsonl", "Schema"),
        ("senza_lf_finale.jsonl", "Schema"),
        ("riga_vuota_intermedia.jsonl", "Schema"),
        ("protocol_version_errato.jsonl", "Schema"),
        ("summary_text_ricalcolata.jsonl", "Mismatch"),
        ("snapshot_self_hash_ricalcolata.jsonl", "Mismatch"),
        ("prefix_dopo_barrier_open.jsonl", "IncompleteBatch"),
        ("prefix_dopo_barrier_close.jsonl", "Mismatch"),
    ]
}

/// Generatore di testo per i negativi: mappa ogni variante al nome congelato.
fn error_kind(err: &Result<V2Log, Error>) -> &'static str {
    match err {
        Err(Error::Schema) => "Schema",
        Err(Error::Chain) => "Chain",
        Err(Error::Phase) => "Phase",
        Err(Error::Config) => "Config",
        Err(Error::Reference) => "Reference",
        Err(Error::WindowState) => "WindowState",
        Err(Error::IncompleteBatch) => "IncompleteBatch",
        Err(Error::Mismatch) => "Mismatch",
        Err(Error::NoModel) => "NoModel",
        Err(Error::DigestMismatch) => "DigestMismatch",
        Err(Error::Overflow) => "Overflow",
        Ok(_) => "Ok",
    }
}

#[test]
fn negative_fixtures_are_rejected_as_preregistered() {
    for (file, atteso) in negative_cases() {
        let path = fixtures_dir().join("negative").join(file);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("negativo preregistrato mancante {}: {e}", path.display()));
        let text = String::from_utf8(bytes).unwrap_or_else(|e| panic!("negativo {file}: utf8 {e}"));
        let result = V2Log::from_jsonl(&text, "2", &[]);
        let kind = error_kind(&result);
        assert_eq!(kind, atteso, "{file}: ottenuto {kind}, atteso {atteso}");
    }
    // invarianza del log di riferimento: il caso buono continua a riprodurre
    // l'identico JSONL dopo l'esercizio dei negativi sul solo SUT puro
    let fixture = frozen_fixture();
    let replay = V2Log::from_jsonl(&fixture, "2", &[]).expect("fixture canonico deve riprodurre");
    assert_eq!(replay.to_jsonl(), fixture);
    assert!(!replay.window_is_open());
}

#[test]
fn prefix_after_barrier_open_select_close_is_coherent() {
    let dir = fixtures_dir().join("negative");
    // open: replay interrotto subito dopo barrier open → IncompleteBatch
    let open = std::fs::read_to_string(dir.join("prefix_dopo_barrier_open.jsonl")).expect("negativo prefix open");
    assert_eq!(error_kind(&V2Log::from_jsonl(&open, "2", &[])), "IncompleteBatch");
    // select: il prefisso open/select ancora aperto è ACCETTATO dal replay
    let select = std::fs::read_to_string(dir.join("prefix_dopo_barrier_select.jsonl")).expect("negativo prefix select");
    let replay = V2Log::from_jsonl(&select, "2", &[]).expect("prefisso open/select deve essere accettato");
    assert!(replay.window_is_open(), "finestra ancora aperta nel prefisso");
    assert_eq!(replay.events().len(), 11);
    // close: replay interrotto tra close e summary → Mismatch (batch incompleto)
    let close = std::fs::read_to_string(dir.join("prefix_dopo_barrier_close.jsonl")).expect("negativo prefix close");
    assert_eq!(error_kind(&V2Log::from_jsonl(&close, "2", &[])), "Mismatch");
}
