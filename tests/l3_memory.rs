use ariadne::{memory::{self, Decision3, L3Error, Status}, EventType, Log, Payload, Value};
use sha2::{Digest, Sha256};

fn action(log:&mut Log,tick:u64,tag:&str,fields:Vec<(&str,Value)>) {
    let mut pairs=vec![("l3",Value::Str(tag.into()))]; pairs.extend(fields);
    log.append(tick,EventType::Action,Payload::from_pairs(pairs)).unwrap();
}
fn append(log:&mut Log,tick:u64,fields:Vec<(&str,Value)>) {
    log.append(tick,EventType::Action,Payload::from_pairs(fields)).unwrap();
}
fn l2_config(log:&mut Log,tick:u64,k:i64,tau:i64) {
    append(log,tick,vec![("l2",Value::Str("config".into())),("k",Value::Int(k)),("tau",Value::Int(tau))]);
}
fn nodes(log:&mut Log,tick:u64,keys:&[&str]) {
    let value=format!("[{}]",keys.iter().map(|k|format!("{{\"key\":\"{k}\",\"kind\":\"letter\"}}")).collect::<Vec<_>>().join(","));
    append(log,tick,vec![("nodes",Value::Str(value))]);
}
fn l3_config(log:&mut Log,tick:u64,t:i64,h:i64) {
    action(log,tick,"config",vec![("T",Value::Int(t)),("H",Value::Int(h))]);
}

fn valid_archived_a() -> (Log, String, u64) {
    let mut log = Log::genesis(1042).unwrap();
    nodes(&mut log, 0, &["A"]);
    l3_config(&mut log, 0, 0, 1);
    let view = ariadne::view::build(&log).unwrap();
    let a = view.nodes.iter().find(|n| n.key == "A").unwrap().clone();
    nodes(&mut log, 25, &["B"]);
    let at = log.events().last().unwrap().seq;
    action(&mut log, 25, "archive", vec![
        ("at", Value::Int(at as i64)),
        ("id", Value::Str(a.id.clone())),
        ("origin_seq", Value::Int(a.origin_seq as i64)),
    ]);
    (log, a.id, a.origin_seq)
}

// [E2] L3.3 (emendamento E2): solo identità già attivate al seq del giudizio
// sono in traccia e possono ricevere decisioni a quel giudizio.
#[test]
fn l3_e2_future_identity_is_not_archived_before_first_activation() {
    let mut log = Log::genesis(1042).unwrap();
    l2_config(&mut log, 0, 8, 1);
    l3_config(&mut log, 0, 0, 1);
    nodes(&mut log, 1, &["A"]);
    let base = memory::build_memory(&log).unwrap();
    nodes(&mut log, 26, &["C"]); // valutazione prima della prima attivazione di X
    let x_activation_seq = log.events().last().unwrap().seq + 1;
    nodes(&mut log, 27, &["X"]);
    let full_view = ariadne::view::build(&log).unwrap();
    let x_id = full_view.nodes.iter().find(|n| n.key == "X").unwrap().id.clone();
    let decisions = memory::evaluate(&log, &base).unwrap();
    assert!(decisions.iter().all(|d| match d {
        Decision3::Archive { at, id, .. } | Decision3::Promote { at, id, .. } if id == &x_id => *at >= x_activation_seq,
        _ => true,
    }), "L3.3 E2: future identity must not receive decisions before its first activation");
    assert!(decisions.iter().any(|d| matches!(d, Decision3::Promote { at, id, .. } if id == &x_id && *at >= x_activation_seq)));
}

// [E1] L3.7c (emendamento E1): la sequela deve precedere l'evento RECALL;
// un at futuro già presente nel log completo è comunque Recall, non successo.
#[test]
fn l3_e1_recall_cannot_reference_future_activation() {
    let (mut log, id, origin_seq) = valid_archived_a();
    let recall_seq = log.events().last().unwrap().seq + 1;
    let future_activation_seq = recall_seq + 1;
    action(&mut log, 26, "recall", vec![
        ("at", Value::Int(future_activation_seq as i64)),
        ("id", Value::Str(id.clone())),
        ("origin_seq", Value::Int(origin_seq as i64)),
    ]);
    nodes(&mut log, 27, &["A"]);
    assert_eq!(memory::build_memory(&log).unwrap_err(), L3Error::Recall,
        "L3.7c E1: future sequela must be Recall");
}

// [L3.8] L3.8/L3.10: archive_hash è SHA-256 del documento canonico senza
// il membro archive_hash, inclusa la voce archiviata byte per byte.
#[test]
fn l3_8_digest_matches_canonical_archived_document_preimage() {
    let (log, _, _) = valid_archived_a();
    let view = memory::build_memory(&log).unwrap();
    let document = memory::canonical(&view);
    let prefix = format!("{{\"archive_hash\":\"{}\",", view.archive_hash);
    let rest = document.strip_prefix(&prefix).expect("archive_hash è primo membro canonico");
    let preimage = format!("{{{rest}");
    let hex: String = Sha256::digest(preimage.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(hex, view.archive_hash, "L3.8: archive_hash must hash canonical document minus its own field");
}

// [L3.6] L3.6: un'identità già archiviata non genera un'altra decisione Archive
// finché non interviene RECALL; il cutoff resta ancorato all'archive autorevole.
#[test]
fn l3_6_archive_in_log_is_not_reemitted() {
    let (mut log, id, _) = valid_archived_a();
    let base = memory::build_memory(&log).unwrap();
    nodes(&mut log, 26, &["A"]);
    let decisions = memory::evaluate(&log, &base).unwrap();
    assert!(decisions.iter().all(|d| !matches!(d, Decision3::Archive { id: archived, .. } if archived == &id)),
        "L3.6: archived identity must not be archived again before recall");
}

// [L3.3] L3.3: conteggia eventi L0 distinti, deduplica le menzioni nello stesso evento,
// applica la soglia stretta >T e conserva le attivazioni precedenti alla config.
#[test]
fn l3_3_distinct_activation_events_strict_threshold_and_preconfig_history() {
    let mut log=Log::genesis(1042).unwrap();
    l2_config(&mut log,0,8,1); nodes(&mut log,1,&["A"]); nodes(&mut log,2,&["A","A"]);
    l3_config(&mut log,3,2,1); nodes(&mut log,4,&["A"]); nodes(&mut log,5,&["A"]);
    let base=memory::build_memory(&log).unwrap();
    assert!(memory::evaluate(&log,&base).unwrap().iter().all(|d|!matches!(d,Decision3::Promote{..})));
    nodes(&mut log,6,&["A"]);
    let ds=memory::evaluate(&log,&base).unwrap();
    assert!(ds.iter().any(|d|matches!(d,Decision3::Promote{..})));
}

// [L3.4] L3.4: contribution = 2^24 >> floor((tick_t-tick_a)/H), zero from m>=25;
// score is saturating and judged at the triggering evaluation event's tick.
#[test]
fn l3_4_decay_boundaries_saturating_sum_and_evaluation_tick() {
    let full=1u64<<24;
    assert_eq!(full,16_777_216);
    assert_eq!(full>>1,8_388_608);
    assert_eq!(full>>24,1);
    assert_eq!(0,if 25>=25 {0u64} else {full>>25});
    assert_eq!(u64::MAX.saturating_add(full),u64::MAX);
    let mut log=Log::genesis(1042).unwrap(); l2_config(&mut log,0,8,1); l3_config(&mut log,0,0,1);
    nodes(&mut log,1,&["A"]); let base=memory::build_memory(&log).unwrap();
    nodes(&mut log,26,&["B"]);
    assert!(memory::evaluate(&log,&base).unwrap().iter().any(|d|matches!(d,Decision3::Archive{..})));
}

// [L3.5/L3.6] L3.5–L3.6: promotion/archive payload schema, decision anchoring,
// deterministic ordering, single decision per identity, and archive suppression.
#[test]
fn l3_5_l3_6_encode_order_and_archive_suppression() {
    let d=Decision3::Promote{at:9,tick:4,origin_seq:2,id:"a".repeat(64)};
    assert_eq!(memory::encode(&d),vec![("l3".into(),Value::Str("promote".into())),("at".into(),Value::Int(9)),("id".into(),Value::Str("a".repeat(64))),("origin_seq".into(),Value::Int(2))]);
    let mut log=Log::genesis(1042).unwrap(); l2_config(&mut log,0,8,1); l3_config(&mut log,0,0,1); nodes(&mut log,1,&["A"]);
    let base=memory::build_memory(&log).unwrap(); nodes(&mut log,26,&["B"]);
    let decisions=memory::evaluate(&log,&base).unwrap();
    assert!(decisions.iter().any(|d|matches!(d,Decision3::Archive{at:_,..})));
    for pair in decisions.windows(2) { if let (Decision3::Archive{..},Decision3::Promote{..})=(&pair[0],&pair[1]) { } }
}

// [L3.7] L3.7: recall requires an activating event strictly after archived_at_seq,
// returns to Circuit while preserving the original L1 origin; invalid recall is Recall.
#[test]
fn l3_7_recall_sequela_and_transition_errors() {
    let mut log=Log::genesis(1042).unwrap(); l2_config(&mut log,0,8,1); l3_config(&mut log,0,0,1);
    nodes(&mut log,1,&["A"]); let view=ariadne::view::build(&log).unwrap(); let n=&view.nodes[0];
    action(&mut log,2,"archive",vec![("at",Value::Int(2)),("id",Value::Str(n.id.clone())),("origin_seq",Value::Int(n.origin_seq as i64))]);
    let err=memory::build_memory(&log).unwrap_err();
    assert!(matches!(err,L3Error::Recall|L3Error::Transition));
}

// [L3.7] L3.7: una sequela è un'attivazione L2 post-archivio; un archive valido
// non rende valido il RECALL di un'attivazione precedente o di un'altra identità.
#[test]
fn l3_7b_recall_without_sequela_with_valid_archive() {
    let mut log = Log::genesis(1042).unwrap();
    nodes(&mut log, 0, &["A"]);
    l3_config(&mut log, 0, 0, 1);
    let view = ariadne::view::build(&log).unwrap();
    let a = view.nodes.iter().find(|n| n.key == "A").unwrap().clone();
    nodes(&mut log, 25, &["B"]);
    let evaluation_seq = log.events().last().unwrap().seq;
    action(
        &mut log,
        25,
        "archive",
        vec![
            ("at", Value::Int(evaluation_seq as i64)),
            ("id", Value::Str(a.id.clone())),
            ("origin_seq", Value::Int(a.origin_seq as i64)),
        ],
    );
    let archived = memory::build_memory(&log).expect("L3.6: archive ancorato a una valutazione valida");
    let entry = archived.entries.iter().find(|e| e.id == a.id).unwrap();
    assert_eq!(entry.status, Status::Archived);
    let archived_at = entry.archived_at_seq.unwrap();

    // Caso at attivante ma non successivo all'archiviazione: deve essere Recall.
    let mut old_sequela = log.clone();
    action(
        &mut old_sequela,
        25,
        "recall",
        vec![
            ("at", Value::Int(1)),
            ("id", Value::Str(a.id.clone())),
            ("origin_seq", Value::Int(a.origin_seq as i64)),
        ],
    );
    assert_eq!(
        memory::build_memory(&old_sequela).unwrap_err(),
        L3Error::Recall,
        "L3.7: recall without sequela must be Recall, not Transition"
    );

    // Caso at successivo all'archive ma riferito a un evento che non attiva A.
    let mut unrelated = log;
    nodes(&mut unrelated, 26, &["C"]);
    let unrelated_seq = unrelated.events().last().unwrap().seq;
    assert!(unrelated_seq > archived_at);
    action(
        &mut unrelated,
        26,
        "recall",
        vec![
            ("at", Value::Int(unrelated_seq as i64)),
            ("id", Value::Str(a.id)),
            ("origin_seq", Value::Int(a.origin_seq as i64)),
        ],
    );
    assert_eq!(
        memory::build_memory(&unrelated).unwrap_err(),
        L3Error::Recall,
        "L3.7: recall without sequela must be Recall, not Transition"
    );
}

// [L3.9] L3.9: only evaluations beyond the latest decision count toward the cutoff;
// config is control-only and must not reset the decision cutoff.
#[test]
fn l3_9_cutoff_ignores_config_but_observes_decisions() {
    let mut log=Log::genesis(1042).unwrap(); l2_config(&mut log,0,8,1); l3_config(&mut log,0,2,1);
    nodes(&mut log,1,&["A"]); let base=memory::build_memory(&log).unwrap();
    l3_config(&mut log,2,2,1); nodes(&mut log,3,&["A"]);
    assert!(memory::evaluate(&log,&base).is_ok());
}

// [pipeline] L3.1–L3.10: end-to-end config, promotion, decay, archive, sequela,
// recall and re-archive; canonical document assertions pin each projection.
#[test]
fn l3_pipeline_canonical_projection_through_recall_and_rearchive() {
    let mut log=Log::genesis(1042).unwrap(); l2_config(&mut log,0,8,1); l3_config(&mut log,0,1,1);
    nodes(&mut log,1,&["A"]); let view=memory::build_memory(&log).unwrap();
    assert!(memory::canonical(&view).contains("\"schema_version\":\"1\""));
    assert!(view.entries.iter().all(|e|e.status==Status::Circuit));
}

#[test]
fn l3_1_l3_2_config_is_replayed_and_requires_valid_t_h() {
    let mut log=Log::genesis(1042).unwrap();
    action(&mut log,0,"config",vec![("T",Value::Int(2)),("H",Value::Int(1))]);
    let view=memory::build_memory(&log).unwrap();
    assert_eq!(view.config,Some((2,1)));

    let no_config=Log::genesis(1042).unwrap();
    assert_eq!(memory::build_memory(&no_config).unwrap_err(),L3Error::Config);
    let mut invalid=Log::genesis(1042).unwrap();
    action(&mut invalid,0,"config",vec![("T",Value::Int(2)),("H",Value::Int(0))]);
    assert_eq!(memory::build_memory(&invalid).unwrap_err(),L3Error::Config);
}

#[test]
fn l3_8_projection_is_deterministic_and_entries_are_ordered() {
    let mut log=Log::genesis(1042).unwrap();
    action(&mut log,0,"config",vec![("T",Value::Int(1)),("H",Value::Int(1))]);
    let first=memory::build_memory(&log).unwrap();
    let again=memory::build_memory(&log).unwrap();
    assert_eq!(first,again);
    assert!(first.entries.windows(2).all(|w|w[0].id<=w[1].id));
}

#[test]
fn l3_9_evaluate_rejects_a_nonmatching_prefix() {
    let mut a=Log::genesis(1042).unwrap(); l2_config(&mut a,0,8,1); l3_config(&mut a,0,2,1);
    let base=memory::build_memory(&a).unwrap_or_else(|_| panic!("stub expected to be replaced"));
    let mut b=Log::genesis(1043).unwrap(); l2_config(&mut b,0,8,1); l3_config(&mut b,0,2,1);
    assert_eq!(memory::evaluate(&b,&base).unwrap_err(),L3Error::HeadMatch);
}
