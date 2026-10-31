//! L4 — legge del sé (identità narrativa): test della legge
//! `notes/L4_legge_identita.md` (commit `2ec39a0`), checklist 1–8.
//!
//! Scelte dichiarate (report del producer):
//! - eventi `l4` apposti MANUALMENTE con payload L0 (`Log::append` + payload
//!   costruito), NON via `encode4` (nessuna dipendenza circolare dei test);
//!   `encode4` ha test dedicati sui payload attesi (L4.6);
//! - eventi `l2`/`l3` costruiti con gli `encode` pubblici vigenti
//!   (`selection::encode`, `memory::encode`, `memory::encode_recall`);
//! - i template L4.3 si assertano sulle DECISIONI (`evaluate_self`, base
//!   presa prima della coda: il cutoff L4.8/L2.22 esclude gli eventi in
//!   base) e su `build_self` con coppie apposte a mano; `self_hash` è
//!   calcolato NEI TEST dal preimage ricostruito a mano (pattern L3.8);
//! - il documento intero NON può coincidere fra log diversi (L4.4: head_hash
//!   = hash dell'ultimo evento, la catena L0 propaga ogni differenza): la
//!   divergenza scoped (i) asserta le `statements` byte per byte.

use ariadne::{EventType, Log, Payload, Value};
use ariadne::identity::{
    build_self, encode4, evaluate_self, Decision4, SelfBase, SelfError, Statement,
};
use ariadne::memory::{self, Decision3};
use ariadne::selection::{self, Decision};
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

fn json_escaped_value(s: &str) -> String {
    // rifuggie `s` con il serializzatore canonico pubblico del payload
    let p = Payload::from_pairs([("x", Value::Str(s.to_string()))]);
    let c = p.canonical(); // {"x":"…"}
    c[5..c.len() - 1].to_string()
}

/// membro `statements` del documento L4.4: {"at":…,"sources":[…],"text":"…"}
/// con array JSON reale e chiavi lessicografiche.
fn doc_member(at: u64, sources: &[u64], text: &str) -> String {
    let srcs: String = sources.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    format!(
        "{{\"at\":{},\"sources\":[{}],\"text\":{}}}",
        at,
        srcs,
        json_escaped_value(text)
    )
}

/// membro `statements` letto da una `Statement` del codice (stessa forma).
fn statement_member(s: &Statement) -> String {
    let srcs: String = s.sources.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    format!(
        "{{\"at\":{},\"sources\":[{}],\"text\":{}}}",
        s.at,
        srcs,
        json_escaped_value(&s.text)
    )
}

/// preimage del `self_hash` (L4.4): documento canonico privo del campo
/// `self_hash`, ricostruito a mano nei test (nessuna dipendenza dal codice).
fn self_preimage(head_hash: &str, members: &[String]) -> String {
    format!(
        "{{\"head_hash\":{},\"schema_version\":\"1\",\"statements\":[{}]}}",
        json_escaped_value(head_hash),
        members.join(",")
    )
}

fn narrate_pairs(at: u64, text: &str, sources: &str) -> Payload {
    Payload::from_pairs([
        ("l4", Value::Str("narrate".into())),
        ("at", Value::Int(at as i64)),
        ("text", Value::Str(text.into())),
        ("sources", Value::Str(sources.into())),
    ])
}

fn snapshot_pairs(at: u64, self_hash: &str) -> Payload {
    Payload::from_pairs([
        ("l4", Value::Str("snapshot".into())),
        ("at", Value::Int(at as i64)),
        ("self_hash", Value::Str(self_hash.into())),
    ])
}

const HEX64_ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// apposizione di una revisione: narrate + snapshot con `self_hash`
/// calcolato NEI TEST dal preimage del documento cumulativo (members `prior`
/// + questa revisione), head_hash = hash del narrate. Ritorna il nuovo membro.
fn appose_revision(
    log: &mut Log,
    tick: u64,
    at: u64,
    text: &str,
    sources: &[u64],
    prior_members: &[String],
) -> String {
    // forma F1: stringa con JSON canonico interno (L4.1/L4.6), es. "[1]"
    let srcs: String = format!(
        "[{}]",
        sources.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
    );
    log.append(tick, EventType::Action, narrate_pairs(at, text, &srcs))
        .unwrap();
    let narrate_hash = log.events().last().unwrap().hash.clone();
    let mut members: Vec<String> = prior_members.to_vec();
    members.push(doc_member(at, sources, text));
    let self_hash = sha256_hex(&self_preimage(&narrate_hash, &members));
    log.append(tick, EventType::Action, snapshot_pairs(at, &self_hash))
        .unwrap();
    doc_member(at, sources, text)
}

fn node_id(key: &str, kind: &str) -> String {
    sha256_hex(&format!("{{\"key\":\"{key}\",\"kind\":\"{kind}\"}}"))
}

fn base_full(log: &Log) -> SelfBase {
    SelfBase {
        head_hash: log.events().last().unwrap().hash.clone(),
        log_len: log.events().len(),
    }
}

/// base alla sola genesi (L4.8: la coda è valutata dopo).
fn base_genesis(log: &Log) -> SelfBase {
    SelfBase {
        head_hash: log.events()[0].hash.clone(),
        log_len: 1,
    }
}

/// evento ACTION con estrazione nodes [A,B]
fn node_a_then_b(log: &mut Log) {
    log.append(5, EventType::Action, Payload::from_pairs([(
        "nodes",
        Value::Str("[{\"key\":\"A\",\"kind\":\"letter\"},{\"key\":\"B\",\"kind\":\"letter\"}]".into()),
    )])).unwrap();
}

// ---------------------------------------------------------------------------
// checklist 1 — genesi only
// ---------------------------------------------------------------------------

// [checklist 1 / L4.7] log di sola genesi: statements vuoto, head_hash = hash
// genesi, self_hash = SHA-256 del preimage ricostruito a mano (L4.4).
#[test]
fn l4_check1_genesis_only_empty_statements_head_hash_and_preimage() {
    let log = Log::genesis(1042).unwrap();
    let doc: ariadne::identity::SelfDocument = build_self(&log).unwrap();
    let genesis_hash = log.events().last().unwrap().hash.clone();
    assert!(doc.statements.is_empty(), "L4.7: genesi only → statements []");
    assert_eq!(doc.head_hash, genesis_hash);
    assert_eq!(
        doc.self_hash,
        sha256_hex(&self_preimage(&genesis_hash, &[])),
        "L4.4: self_hash coincidente col preimage del documento privo del campo"
    );
}

// ---------------------------------------------------------------------------
// checklist 2 — divergenza scoped
// ---------------------------------------------------------------------------

// [checklist 2(i) / L4.3] log che differiscono SOLO per un evento non
// rilevante (azione senza estrazioni, apposta dopo le narrazioni) →
// narrazione IDENTICA byte per byte; head_hash/self_hash differiscono per la
// catena (L4.4) — vedi nota in intestazione.
#[test]
fn l4_check2_i_irrelevant_difference_produces_identical_narrated_statements() {
    let mut a = Log::genesis(1042).unwrap();
    let mut members: Vec<String> = Vec::new();
    a.append(5, EventType::Action, Payload::from_pairs([(
        "nodes",
        Value::Str("[{\"key\":\"A\",\"kind\":\"letter\"},{\"key\":\"B\",\"kind\":\"letter\"}]".into()),
    )])).unwrap();
    members.push(appose_revision(
        &mut a, 5, 1,
        "a seq 1 ho dichiarato l'elemento letter \"A\"\n\
         a seq 1 ho dichiarato l'elemento letter \"B\"",
        &[1], &[],
    )); // seq 2–3
    a.append(5, EventType::Action, Payload::from_pairs([(
        "edges",
        Value::Str(
            "[{\"dst_key\":\"B\",\"dst_kind\":\"letter\",\"rel\":\"knows\",\"src_key\":\"A\",\"src_kind\":\"letter\"}]".into(),
        ),
    )])).unwrap(); // seq 4: prime dichiarazione dell'arco (rilevante)
    members.push(appose_revision(
        &mut a, 5, 4,
        "a seq 4 ho dichiarato l'arco knows da letter \"A\" a letter \"B\"",
        &[4], &members,
    )); // seq 5–6

    // log B = A + UN evento non rilevante in più (nessuna estrazione, senza l4)
    let mut b = a.clone();
    b.append(5, EventType::Action, Payload::from_pairs([
        ("note", Value::Str("evento irrilevante".into())),
    ])).unwrap(); // seq 7

    let da = build_self(&a).unwrap();
    let db = build_self(&b).unwrap();
    let sa: String = da.statements.iter().map(statement_member).collect::<Vec<_>>().join(",");
    let sb: String = db.statements.iter().map(statement_member).collect::<Vec<_>>().join(",");
    assert_eq!(sa, sb, "divergenza scoped (i): eventi irrilevanti → narrazione identica");
    assert_eq!(db.statements.len(), 2);
    assert_eq!(da.statements[1].text, "a seq 4 ho dichiarato l'arco knows da letter \"A\" a letter \"B\"");
    // la catena propaga: head_hash (e il documento intero) differiscono
    assert_ne!(da.head_hash, db.head_hash, "L4.4: head_hash = hash dell'ultimo evento");
    assert_ne!(da.self_hash, db.self_hash, "l'ancora copre head_hash: i due log sono distinti");
}

// [checklist 2(ii) / L4.3] log che differiscono su eventi rilevanti (prima
// dichiarazione diversa) → narrazione DIVERSA. Nessun test che assuma
// iniettività universale (Q9 firmato).
#[test]
fn l4_check2_ii_relevant_difference_produces_divergent_narrated_statements() {
    let mk = |key: &str, text: &str| -> Log {
        let mut log = Log::genesis(1042).unwrap();
        log.append(5, EventType::Action, Payload::from_pairs([
            ("nodes", Value::Str(format!("[{{\"key\":\"{key}\",\"kind\":\"letter\"}}]"))),
        ])).unwrap();
        appose_revision(&mut log, 5, 1, text, &[1], &[]);
        log
    };
    let a = mk("X", "a seq 1 ho dichiarato l'elemento letter \"X\"");
    let b = mk("Y", "a seq 1 ho dichiarato l'elemento letter \"Y\"");
    let da = build_self(&a).unwrap();
    let db = build_self(&b).unwrap();
    assert_ne!(
        da.statements[0].text, db.statements[0].text,
        "divergenza scoped (ii): differenza su evento rilevante → narrazione diversa"
    );
}

// ---------------------------------------------------------------------------
// checklist 3 — tutti i template L4.3
// ---------------------------------------------------------------------------

// [checklist 3 / L4.3] tutti i template (nodo, arco, select, summary,
// promote, archive, recall), ordine interno lessicografico (nodi (kind,key),
// archi (rel,src_id,dst_id)), congiunzione `\n`, liste `[]` e virgole senza
// spazi, `sources`, ordine Narrate→Snapshot con at/tick coerenti. La narrazione
// viene assertata sulle DECISIONI (L4.3 via evaluate_self), base presa prima.
#[test]
fn l4_check3_all_templates_exact_texts_internal_order_sources_and_join() {
    let id_a = node_id("A", "letter");
    let mut log = Log::genesis(1042).unwrap();
    // seq 1 — un solo evento: 3 nodi (input non ordinato) + 2 archi (input non ordinato)
    log.append(5, EventType::Action, Payload::from_pairs([
        ("edges", Value::Str(
            "[{\"dst_key\":\"C\",\"dst_kind\":\"letter\",\"rel\":\"knows\",\"src_key\":\"A\",\"src_kind\":\"letter\"},\
             {\"dst_key\":\"B\",\"dst_kind\":\"letter\",\"rel\":\"add\",\"src_key\":\"A\",\"src_kind\":\"letter\"}]".into())),
        ("nodes", Value::Str(
            "[{\"key\":\"C\",\"kind\":\"letter\"},{\"key\":\"A\",\"kind\":\"letter\"},{\"key\":\"B\",\"kind\":\"letter\"}]".into())),
    ])).unwrap();
    let base = base_genesis(&log);
    // seq 2 — l2 select via encode pubblico
    log.append(5, EventType::Action, Payload::from_pairs(
        selection::encode(&Decision::Select { at: 1, tick: 5, ups: vec![1], downs: vec![] }),
    )).unwrap();
    // seq 3 — l2 summary (il campo text L2 è autorevole, non letto da L4)
    log.append(6, EventType::Action, Payload::from_pairs(
        selection::encode(&Decision::Summary { expulsions: vec![1], text: "textautorevole".into() }),
    )).unwrap();
    // seq 4 — l3 promote; seq 5 — archive; seq 6 — recall
    log.append(7, EventType::Action, Payload::from_pairs(
        memory::encode(&Decision3::Promote { origin_seq: 1, at: 1, tick: 7, id: id_a.clone() }),
    )).unwrap();
    log.append(8, EventType::Action, Payload::from_pairs(
        memory::encode(&Decision3::Archive { origin_seq: 1, at: 1, tick: 8, id: id_a.clone() }),
    )).unwrap();
    log.append(9, EventType::Action, Payload::from_pairs(
        memory::encode_recall(1, 1, id_a.clone()),
    )).unwrap();

    let d = evaluate_self(&log, &base).unwrap();
    assert_eq!(d.len(), 12, "6 eventi rilevanti → 6 revisioni × (Narrate, Snapshot)");
    for pair in d.chunks(2) {
        assert!(matches!(&pair[1], Decision4::Snapshot { .. }), "per revisione prima Narrate poi Snapshot");
        match (&pair[0], &pair[1]) {
            (Decision4::Narrate { at: a1, tick: t1, .. }, Decision4::Snapshot { at: a2, tick: t2, .. })
                if a1 == a2 && t1 == t2 => {}
            _ => panic!("Snapshot: at e tick identici al narrate"),
        }
    }
    let get = |i: usize, expected: &str, at: u64, tick: u64, sources: &[u64]| {
        match &d[i] {
            Decision4::Narrate { at: a, tick: t, text, sources: src } => {
                assert_eq!(text, expected);
                assert_eq!(*a, at);
                assert_eq!(*t, tick, "L4.6: tick dell'evento apposto (pattern L2.18)");
                assert_eq!(src, sources, "bibliografia: seq citate, dedup crescente");
            }
            _ => panic!("attesa decisione Narrate"),
        }
    };
    // seq 1: nodi in ordine lessicografico (kind,key) e archi per (rel,src,dst)
    get(0,
        "a seq 1 ho dichiarato l'elemento letter \"A\"\n\
         a seq 1 ho dichiarato l'elemento letter \"B\"\n\
         a seq 1 ho dichiarato l'elemento letter \"C\"\n\
         a seq 1 ho dichiarato l'arco add da letter \"A\" a letter \"B\"\n\
         a seq 1 ho dichiarato l'arco knows da letter \"A\" a letter \"C\"",
        1, 5, &[1]);
    // seq 2: select — parse canonico, `[]` vuoto, virgola senza spazi
    get(2, "a seq 2 ho selezionato: entrate [1], uscite []", 2, 5, &[1, 2]);
    // seq 3: summary
    get(4, "a seq 3 ho consolidato: espulsi [1]", 3, 6, &[1, 3]);
    // seq 4: promote con id L1 completo (hex minuscolo 64)
    get(6, &format!("a seq 4 ho promosso l'identità {id_a}"), 4, 7, &[4]);
    // seq 5: archive
    get(8, &format!("a seq 5 ho archiviato l'identità {id_a}"), 5, 8, &[5]);
    // seq 6: recall con origin_seq
    get(10, &format!("a seq 6 ho richiamato l'identità {id_a} (origin_seq 1)"), 6, 9, &[1, 6]);
}

// [checklist 3 / L4.3] liste: ordinamento crescente, dedup, vuoto `[]`.
#[test]
fn l4_check3_select_list_rendering_sorted_dedup_and_empty() {
    let mut log = Log::genesis(1042).unwrap();
    node_a_then_b(&mut log); // seq 1
    let base = base_genesis(&log);
    log.append(5, EventType::Action, Payload::from_pairs(
        selection::encode(&Decision::Select { at: 1, tick: 5, ups: vec![3, 1, 3], downs: vec![2, 2] }),
    )).unwrap(); // seq 2
    let d = evaluate_self(&log, &base).unwrap();
    match &d[2] {
        Decision4::Narrate { text, sources, .. } => {
            assert_eq!(text, "a seq 2 ho selezionato: entrate [1,3], uscite [2]");
            assert_eq!(sources, &[1, 2, 3], "bibliografia: seq citate nel template, dedup crescente");
        }
        _ => panic!(),
    }
}

// [L4.3] NON rilevanti: seconda dichiarazione dello stesso elemento, riprova
// di menzione, eventi senza estrazioni, eventi di controllo l2/l3 (config,
// barrier; l3 config escluso in (d)).
#[test]
fn l4_check3_control_rementions_and_no_extraction_events_are_not_relevant() {
    let id_a = node_id("A", "letter");
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([(
        "nodes", Value::Str("[{\"key\":\"A\",\"kind\":\"letter\"}]".into()),
    )])).unwrap(); // seq 1: prima dichiarazione A
    let base = base_genesis(&log);
    log.append(6, EventType::Action, Payload::from_pairs(
        selection::encode(&Decision::Select { at: 1, tick: 6, ups: vec![1], downs: vec![] }),
    )).unwrap(); // seq 2: select (rilevante)
    log.append(7, EventType::Action, Payload::from_pairs([
        ("l2", Value::Str("config".into())), ("k", Value::Int(8)), ("tau", Value::Int(1)),
    ])).unwrap(); // seq 3: controllo → escluso
    log.append(8, EventType::Action, Payload::from_pairs([("l2", Value::Str("barrier".into()))]))
        .unwrap(); // seq 4: controllo → escluso
    log.append(9, EventType::Action, Payload::from_pairs([
        ("l3", Value::Str("config".into())), ("T", Value::Int(0)), ("H", Value::Int(1)),
    ])).unwrap(); // seq 5: controllo → escluso
    node_a_then_b(&mut log); // seq 6: B prima dichiarazione (rilevante), A riprova
    log.append(5, EventType::Action, Payload::from_pairs([
        ("note", Value::Str("nessuna estrazione".into())),
    ])).unwrap(); // seq 7: irrilevante
    log.append(5, EventType::Action, Payload::from_pairs(
        memory::encode(&Decision3::Promote { origin_seq: 1, at: 1, tick: 5, id: id_a.clone() }),
    )).unwrap(); // seq 8: promote (rilevante)
    let d = evaluate_self(&log, &base).unwrap();
    // rilevanti: seq 1, 2, 6, 8 → 8 decisioni
    assert_eq!(d.len(), 8);
    let narrates: Vec<(u64, String)> = d.iter().filter_map(|x| match x {
        Decision4::Narrate { at, text, .. } => Some((*at, text.clone())),
        _ => None,
    }).collect();
    assert_eq!(narrates, vec![
        (1, "a seq 1 ho dichiarato l'elemento letter \"A\"".to_string()),
        (2, "a seq 2 ho selezionato: entrate [1], uscite []".to_string()),
        (6, "a seq 6 ho dichiarato l'elemento letter \"B\"".to_string()),
        (8, format!("a seq 8 ho promosso l'identità {id_a}")),
    ]);
}

// ---------------------------------------------------------------------------
// checklist 4 — replay completo ed errori
// ---------------------------------------------------------------------------

// [checklist 4 / L4.6↔L4.7] coerenza fra simulate (evaluate_self) e
// apposizione + replay (build_self): le decisioni Narrate/Snapshot calcolate
// sul log pulito coincidono con le coppie apposte a mano (testo e
// self_hash indipendente del test), il documento è ricostruito e gli
// snapshot verificati; con la narrazione già apposta il cutoff non produce
// nulla (il prefisso narrato non produce nulla)
#[test]
fn l4_check4_replay_rebuilds_document_and_snapshots_verified() {
    // log D: solo eventi rilevanti, nessun l4 → decisioni simulate
    let mut d_log = Log::genesis(1042).unwrap();
    node_a_then_b(&mut d_log); // seq 1: tick 5
    d_log.append(5, EventType::Action, Payload::from_pairs([(
        "nodes", Value::Str("[{\"key\":\"C\",\"kind\":\"letter\"}]".into()),
    )])).unwrap(); // seq 2: tick 5
    let d = evaluate_self(&d_log, &base_genesis(&d_log)).unwrap();
    let (t1, t2) = (match &d[0] { Decision4::Narrate { text, .. } => text.clone(), _ => panic!() },
                    match &d[2] { Decision4::Narrate { text, .. } => text.clone(), _ => panic!() });
    let (h1, h2) = (match &d[1] { Decision4::Snapshot { self_hash, .. } => self_hash.clone(), _ => panic!() },
                    match &d[3] { Decision4::Snapshot { self_hash, .. } => self_hash.clone(), _ => panic!() });
    assert_eq!(t1, "a seq 1 ho dichiarato l'elemento letter \"A\"\n\
                    a seq 1 ho dichiarato l'elemento letter \"B\"");
    assert_eq!(t2, "a seq 2 ho dichiarato l'elemento letter \"C\"");

    // log R: stessi eventi + coppie apposte A MANO (tick = decisione)
    let mut log = Log::genesis(1042).unwrap();
    node_a_then_b(&mut log);    // seq 1
    log.append(5, EventType::Action, Payload::from_pairs([(
        "nodes", Value::Str("[{\"key\":\"C\",\"kind\":\"letter\"}]".into()),
    )])).unwrap();               // seq 2
    let m1 = appose_revision(&mut log, 5, 1, &t1, &[1], &[]);  // seq 3–4
    let m2 = appose_revision(&mut log, 5, 2, &t2, &[2], &[m1.clone()]); // seq 5–6

    // le coppie apposte portano GLI STESSI self_hash delle decisioni
    let snap1 = log.events()[4].payload.get("self_hash").unwrap();
    assert_eq!(snap1, &Value::Str(h1), "L4.6 simulazione ↔ L4.7 apposizione");
    let snap2 = log.events()[6].payload.get("self_hash").unwrap();
    assert_eq!(snap2, &Value::Str(h2));

    // replay: documento ricostruito, snapshot verificati
    let doc = build_self(&log).unwrap();
    assert_eq!(statement_member(&doc.statements[0]), m1);
    assert_eq!(statement_member(&doc.statements[1]), m2);
    let head = log.events().last().unwrap().hash.clone();
    assert_eq!(doc.head_hash, head);
    assert_eq!(doc.self_hash, sha256_hex(&self_preimage(&head, &[m1, m2])));

    // cutoff: il prefisso narrato produce nulla (L4.6,L4.8)
    let base_prefix = SelfBase {
        head_hash: log.events()[4].hash.clone(),
        log_len: 5,
    };
    assert!(evaluate_self(&log, &base_prefix).unwrap().is_empty());
}

// [L4.7] un narrate solitario come ULTIMO evento del log è tollerato: nessun
// errore; il suo testo è autorevole (L2.11) e alimenta il documento anche
// se NON conforme al ricalcolo L4.3 (mismatch dichiarato, non errore).
#[test]
fn l4_check4_lone_narrate_as_last_event_is_tolerated_and_authoritative() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, narrate_pairs(0, "narrazione solitaria autorevole", "[]"))
        .unwrap();
    let doc = build_self(&log).unwrap();
    assert_eq!(doc.statements.len(), 1);
    assert_eq!(doc.statements[0].at, 0);
    assert_eq!(doc.statements[0].text, "narrazione solitaria autorevole");
    assert_eq!(doc.statements[0].sources, Vec::<u64>::new());
}

// [L4.7] snapshot non appaiato (nessun narrate precedente) → Schema.
#[test]
fn l4_check4_unpaired_snapshot_is_schema() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [L4.7] coppie rotte: narrate chiusa da un evento successivo non appaiato;
// narrate+narrate; snapshot+snapshot; snapshot con `at` diverso dal narrate.
#[test]
fn l4_check4_broken_pairs_are_schema() {
    // narrate chiusa da un evento qualunque non appaiato
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, narrate_pairs(0, "t", "[]")).unwrap();
    node_a_then_b(&mut log);
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // narrate+narrate
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, narrate_pairs(0, "t", "[]")).unwrap();
    log.append(5, EventType::Action, narrate_pairs(1, "t2", "[]")).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // snapshot+snapshot
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, narrate_pairs(0, "t", "[]")).unwrap();
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap();
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // snapshot subito dopo il narrate ma `at` diverso
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([(
        "nodes", Value::Str("[{\"key\":\"A\",\"kind\":\"letter\"}]".into()),
    )])).unwrap();
    log.append(5, EventType::Action, narrate_pairs(1, "t", "[]")).unwrap();
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [L4.7] `self_hash` duramente verificato: mismatch → Schema.
#[test]
fn l4_check4_self_hash_mismatch_is_schema() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, narrate_pairs(0, "t", "[]")).unwrap();
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap(); // hash non conforme
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [L4.7] `at` impossibile: seq di evento inesistente (≥ seq del narrate) o
// che punta a un evento `l4` (autoreferenzialità vietata) → Schema.
#[test]
fn l4_check4_narrate_at_inexistent_or_l4_is_schema() {
    // at == seq del narrate (evento inesistente/proiezione su se stesso)
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, narrate_pairs(1, "t", "[]")).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // at che punta a un evento l4 (lo snapshot a seq 2)
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, narrate_pairs(0, "t", "[]")).unwrap();
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap();
    log.append(5, EventType::Action, narrate_pairs(2, "t2", "[]")).unwrap();
    log.append(5, EventType::Action, snapshot_pairs(0, HEX64_ZERO)).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [L4.1] domini chiusi: `l4` assieme a `l2` (o `l3`) → Schema.
#[test]
fn l4_check4_l4_with_l2_or_l3_is_schema() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l4", Value::Str("narrate".into())),
        ("l2", Value::Str("select".into())),
        ("at", Value::Int(0)),
        ("text", Value::Str("t".into())),
        ("sources", Value::Str("[]".into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l4", Value::Str("snapshot".into())),
        ("l3", Value::Str("promote".into())),
        ("at", Value::Int(0)),
        ("self_hash", Value::Str(HEX64_ZERO.into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [L4.1/L4.7] chiavi extra → Schema; campi mancanti o fuori forma → Schema.
#[test]
fn l4_check4_extra_keys_and_bad_shapes_are_schema() {
    // chiave extra su narrate
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l4", Value::Str("narrate".into())),
        ("at", Value::Int(0)),
        ("text", Value::Str("t".into())),
        ("sources", Value::Str("[]".into())),
        ("note", Value::Str("extra".into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // chiave extra su snapshot
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l4", Value::Str("snapshot".into())),
        ("at", Value::Int(0)),
        ("self_hash", Value::Str(HEX64_ZERO.into())),
        ("text", Value::Str("extra".into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // valore di l4 fuori dominio
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l4", Value::Str("narration".into())),
        ("at", Value::Int(0)),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // narrate senza text
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l4", Value::Str("narrate".into())),
        ("at", Value::Int(0)),
        ("sources", Value::Str("[]".into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);

    // `sources` non canonico
    for src in ["[1,0]", "[1,1]", "nope", "[ 1]", "[5", "[] "] {
        let mut log = Log::genesis(1042).unwrap();
        log.append(5, EventType::Action, Payload::from_pairs([
            ("l4", Value::Str("narrate".into())),
            ("at", Value::Int(0)),
            ("text", Value::Str("t".into())),
            ("sources", Value::Str(src.into())),
        ])).unwrap();
        assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema, "sources {src}");
    }

    // `self_hash` non hex minuscolo a 64
    for bad in ["AB", &"A".repeat(64), &"g".repeat(64)] {
        let mut log = Log::genesis(1042).unwrap();
        log.append(5, EventType::Action, narrate_pairs(0, "t", "[]")).unwrap();
        log.append(5, EventType::Action, snapshot_pairs(0, bad)).unwrap();
        assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema, "self_hash {bad}");
    }

    // `at` non intero
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l4", Value::Str("narrate".into())),
        ("at", Value::Str("0".into())),
        ("text", Value::Str("t".into())),
        ("sources", Value::Str("[]".into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [L4.1, pattern L3.1 secondo capoverso] una scritta L4 è un evento ACTION:
// un evento STATE con `l4` → Schema (lettura dichiarata nel report).
#[test]
fn l4_check4_state_with_l4_is_schema() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::State, Payload::from_pairs([
        ("l4", Value::Str("narrate".into())),
        ("at", Value::Int(0)),
        ("text", Value::Str("t".into())),
        ("sources", Value::Str("[]".into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [rilievo audit L4 / L4.2+L4.1] un evento non-ACTION (SENSE) con chiavi
// l2/l3 nel payload è payload decisionale malformato: la validazione a
// campi limitati vale per OGNI evento che il piano L4 legge, a prescindere
// dal tipo L0 → Schema ANCHE nel replay di build_self (stessa esito di
// evaluate_self, nessuna divergenza fra le due API).
#[test]
fn l4_check4_replay_non_action_with_l2_or_l3_is_schema() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(3, EventType::Sense, Payload::from_pairs([
        ("l2", Value::Str("select".into())),
        ("at", Value::Int(0)),
        ("ups", Value::Str("[0]".into())),
        ("downs", Value::Str("[]".into())),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// [L4.3(d)/L4.7] errore dentro un evento letto da L4: id L3 non conforme
// (non hex minuscolo a 64) → Schema.
#[test]
fn l4_check4_malformed_l3_fields_are_schema() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs([
        ("l3", Value::Str("promote".into())),
        ("at", Value::Int(1)),
        ("id", Value::Str("not-an-id".into())),
        ("origin_seq", Value::Int(1)),
    ])).unwrap();
    assert_eq!(build_self(&log).unwrap_err(), SelfError::Schema);
}

// ---------------------------------------------------------------------------
// checklist 5 — cutoff e coerenza di prefisso
// ---------------------------------------------------------------------------

// [checklist 5 / L4.6/L4.8/L2.22] decisioni SOLO per la coda oltre il
// cutoff; nessuna decisione per eventi nella base; HeadMatch su base
// incoerente (fungente e hash in posizione log_len−1).
#[test]
fn l4_check5_cutoff_tail_only_and_prefix_coherence() {
    let mut log = Log::genesis(1042).unwrap();
    node_a_then_b(&mut log); // seq 1
    let members: Vec<String> = Vec::new();
    let _m = appose_revision(
        &mut log, 5, 1,
        "a seq 1 ho dichiarato l'elemento letter \"A\"\n\
         a seq 1 ho dichiarato l'elemento letter \"B\"",
        &[1], &members,
    ); // seq 2–3 (cutoff = 3)
    let head_after = log.events().last().unwrap().hash.clone();

    log.append(5, EventType::Action, Payload::from_pairs([(
        "nodes", Value::Str("[{\"key\":\"C\",\"kind\":\"letter\"}]".into()),
    )])).unwrap(); // seq 4

    // base sul prefisso: decisioni solo per la coda (seq 4)
    let base = SelfBase { head_hash: head_after.clone(), log_len: log.events().len() - 1 };
    let d = evaluate_self(&log, &base).unwrap();
    assert_eq!(d.len(), 2);
    match &d[0] {
        Decision4::Narrate { at, text, sources, .. } => {
            assert_eq!(*at, 4);
            assert_eq!(text, "a seq 4 ho dichiarato l'elemento letter \"C\"");
            assert_eq!(sources, &[4]);
        }
        _ => panic!("attesa Narrate"),
    }
    assert!(matches!(&d[1], Decision4::Snapshot { at, .. } if *at == 4));

    // base = log intero: nessuna decisione per eventi nella base
    assert!(evaluate_self(&log, &base_full(&log)).unwrap().is_empty());

    // coerenza di prefisso (L2.22): hash in posizione log_len−1 non conforme
    let wrong = SelfBase { head_hash: HEX64_ZERO.into(), log_len: 4 };
    assert_eq!(evaluate_self(&log, &wrong).unwrap_err(), SelfError::HeadMatch);
    // log più corto della base
    let too_short = SelfBase { head_hash: head_after.clone(), log_len: log.events().len() + 1 };
    assert_eq!(evaluate_self(&log, &too_short).unwrap_err(), SelfError::HeadMatch);
    // base vuota
    let zero = SelfBase { head_hash: head_after.clone(), log_len: 0 };
    assert_eq!(evaluate_self(&log, &zero).unwrap_err(), SelfError::HeadMatch);
}

// ---------------------------------------------------------------------------
// checklist 6 — interazione L2/L3
// ---------------------------------------------------------------------------

// [checklist 6 / L4.9] un evento `l4` (non-l2) chiude IMPLICITAMENTE la
// finestra L2 aperta; nessun side-effect su L2/L3 né sui loro cutoff.
#[test]
fn l4_check6_l4_event_closes_l2_window_without_side_effects() {
    let mut log = Log::genesis(1042).unwrap();
    log.append(1, EventType::Action, Payload::from_pairs([
        ("l3", Value::Str("config".into())), ("T", Value::Int(0)), ("H", Value::Int(1)),
    ])).unwrap(); // seq 1: l3 config (controllo)
    log.append(2, EventType::Action, Payload::from_pairs([("l2", Value::Str("barrier".into()))]))
        .unwrap(); // seq 2: finestra L2 aperta
    log.append(3, EventType::Action, narrate_pairs(0, "chiudo la finestra", "[]"))
        .unwrap(); // seq 3: l4 — evento non-l2 → chiude la finestra (L2.25)

    // L2: replay autorevole senza errore, nessuna decisione indotta da l4
    let active = selection::build_active(&log).unwrap();
    assert!(selection::evaluate(&log, &active).unwrap().is_empty());

    // L3: replay sano, cutoff L3 non toccato da l4
    let mv = memory::build_memory(&log).unwrap();
    assert!(memory::evaluate(&log, &mv).unwrap().is_empty());

    // L4: narrate solitario valido come ULTIMO evento
    let doc = build_self(&log).unwrap();
    assert_eq!(doc.statements.len(), 1);
    assert!(evaluate_self(&log, &base_full(&log)).unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// checklist 7 — determinismo
// ---------------------------------------------------------------------------

// [checklist 7 / L4.4] due generazioni dal medesimo log → output identico
// byte per byte (struttura e rappresentazione).
#[test]
fn l4_check7_determinism_two_evaluations_identical() {
    let id_a = node_id("A", "letter");
    let mut log = Log::genesis(1042).unwrap();
    node_a_then_b(&mut log);
    let base = base_genesis(&log);
    log.append(5, EventType::Action, Payload::from_pairs(
        selection::encode(&Decision::Select { at: 1, tick: 5, ups: vec![1], downs: vec![1] }),
    )).unwrap();
    log.append(6, EventType::Action, Payload::from_pairs(
        memory::encode(&Decision3::Promote { origin_seq: 1, at: 1, tick: 6, id: id_a.clone() }),
    )).unwrap();
    let d1 = evaluate_self(&log, &base).unwrap();
    let d2 = evaluate_self(&log, &base).unwrap();
    assert_eq!(d1, d2);
    assert_eq!(format!("{:?}", d1), format!("{:?}", d2), "identici byte per byte");
    let doc1 = build_self(&log).unwrap();
    let doc2 = build_self(&log).unwrap();
    assert_eq!(doc1, doc2);
}

// ---------------------------------------------------------------------------
// checklist 8 — zero draw
// ---------------------------------------------------------------------------

// [checklist 8 / L4.5] le API L4 girano senza PRNG: lo stato PCG64 della
// genesi resta identico prima/dopo, nessun avanzamento, catena intatta.
#[test]
fn l4_check8_l4_apis_do_not_advance_prng_state() {
    let mut log = Log::genesis(1042).unwrap();
    let state_before = match log.events()[0].payload.get("state") {
        Some(Value::Str(s)) => s.clone(),
        _ => panic!("payload genesi"),
    };
    node_a_then_b(&mut log);
    let _ = build_self(&log).unwrap();
    let _ = evaluate_self(&log, &base_full(&log)).unwrap();
    let state_after = match log.events()[0].payload.get("state") {
        Some(Value::Str(s)) => s.clone(),
        _ => panic!("payload genesi"),
    };
    assert_eq!(state_before, state_after, "L4.5: zero draw, stato PRNG intatto");
    log.verify().expect("la catena resta valida (nessuna scrittura L4)");
}

// ---------------------------------------------------------------------------
// encode4 (payload attesi, L4.6)
// ---------------------------------------------------------------------------

// [L4.6] encode4: tuple nell'ordine della legge (l4, at, text, sources) /
// (l4, at, self_hash); liste come Value::Str con JSON canonico, at Value::Int.
#[test]
fn l4_encode4_narrate_and_snapshot_payload_pairs() {
    let d = Decision4::Narrate {
        at: 5,
        tick: 2,
        text: "a seq 5 ho dichiarato l'elemento letter \"A\"".into(),
        sources: vec![5],
    };
    assert_eq!(
        encode4(&d),
        vec![
            ("l4".into(), Value::Str("narrate".into())),
            ("at".into(), Value::Int(5)),
            ("text".into(), Value::Str("a seq 5 ho dichiarato l'elemento letter \"A\"".into())),
            ("sources".into(), Value::Str("[5]".into())),
        ]
    );
    let d = Decision4::Narrate { at: 5, tick: 2, text: "t".into(), sources: vec![1, 2, 5] };
    assert_eq!(encode4(&d)[3], ("sources".into(), Value::Str("[1,2,5]".into())));
    let s = Decision4::Snapshot {
        at: 5,
        tick: 2,
        self_hash: "ab".repeat(32),
    };
    assert_eq!(
        encode4(&s),
        vec![
            ("l4".into(), Value::Str("snapshot".into())),
            ("at".into(), Value::Int(5)),
            ("self_hash".into(), Value::Str("ab".repeat(32).into())),
        ]
    );
}

// [L4.6] il payload di encode4 è apponibile via append L0 e rinsalato dal
// replay come narrazione autorevole (round trip costruttivo).
#[test]
fn l4_encode4_payload_appendable_as_l0_event() {
    let d = Decision4::Narrate { at: 0, tick: 5, text: "round trip".into(), sources: vec![] };
    let mut log = Log::genesis(1042).unwrap();
    log.append(5, EventType::Action, Payload::from_pairs(encode4(&d))).unwrap();
    let doc = build_self(&log).unwrap();
    assert_eq!(doc.statements.len(), 1);
    assert_eq!(doc.statements[0].text, "round trip");
}
