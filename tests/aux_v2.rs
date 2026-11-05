use ariadne::aux_v2::{Error, Fixture, Json, MemoryStatus, V2Log};
use sha2::{Digest,Sha256};

fn hash(s:&str)->String {Sha256::digest(s.as_bytes()).iter().map(|b|format!("{b:02x}")).collect()}
fn data(log:&V2Log, seq:usize)->Json {Json::parse(log.events()[seq].payload.get("data").unwrap().string().unwrap()).unwrap()}

#[test]
fn canonical_json_rejects_ambiguous_encodings() {
    for bad in ["{\"b\":1,\"a\":2}","{\"a\":1,\"a\":2}","[01]","[-0]","[1.0]","[true]","[null]","[\"\\u0061\"]","[1,]","{} "] {
        assert_eq!(Json::parse(bad),Err(Error::Schema),"{bad}");
    }
    let good="{\"a\":[1,\"è\\n\"],\"b\":{}}";
    assert_eq!(Json::parse(good).unwrap().canonical(),good);
}

#[test]
fn genesis_and_l0_cycles_match_v1_bytes() {
    let mut log=V2Log::genesis(1042).unwrap();
    let mut old=ariadne::Log::genesis(1042).unwrap();
    let mut rng=ariadne::prng::Pcg64::new(1042);
    for _ in 0..3 {log.cycle().unwrap();ariadne::cycle::run_cycle(&mut old,&mut rng).unwrap();}
    assert_eq!(log.to_jsonl(),old.to_jsonl());
    log.verify(&[]).unwrap();
}

#[test]
fn shared_origins_select_exact_ids_not_ambiguous_sequences() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(3,5,1,10).unwrap();
    let source=log.cycle().unwrap();
    let extraction=log.extract(source,&[("elemento","alfa"),("elemento","beta"),("elemento","gamma"),("elemento","delta")],&[("elemento","alfa","elemento","beta","conosce")]).unwrap();
    log.close_selection_window(extraction).unwrap();
    let a=V2Log::node_id("elemento","alfa");
    assert!(!log.active().contains(&a));assert_eq!(log.active().len(),3);
    assert!(log.nodes().values().all(|node|node.origin_seq==source));
    let select=log.events().iter().find(|e| e.kind=="AUX"&&e.payload.get("kind").unwrap().string().unwrap()=="select").unwrap();
    assert_eq!(data(&log,select.seq as usize).get("ups").unwrap().array().unwrap().len(),3);
    assert_eq!(data(&log,select.seq as usize).get("downs").unwrap().array().unwrap().len(),0);
    let summary=log.events().iter().find(|e|e.kind=="AUX"&&e.payload.get("kind").unwrap().string().unwrap()=="summary").unwrap();
    assert_eq!(data(&log,summary.seq as usize).get("expulsions").unwrap().canonical(),"[]");
    assert_eq!(log.events()[summary.seq as usize-1].payload.get("kind").unwrap().string().unwrap(),"barrier");
    log.verify(&[]).unwrap();
}

#[test]
fn nonempty_summary_reports_actual_previous_member() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(1,5,1,10).unwrap();
    let source=log.cycle().unwrap();let e=log.extract(source,&[("elemento","alfa")],&[]).unwrap();log.close_selection_window(e).unwrap();
    let source=log.cycle().unwrap();let e=log.extract(source,&[("elemento","beta")],&[]).unwrap();log.close_selection_window(e).unwrap();
    let summary=log.events().iter().rev().find(|e|e.kind=="AUX"&&e.payload.get("kind").unwrap().string().unwrap()=="summary").unwrap();
    assert_eq!(data(&log,summary.seq as usize).get("text").unwrap().string().unwrap(),V2Log::node_id("elemento","alfa"));
    log.verify(&[]).unwrap();
}

#[test]
fn full_l1_l4_promote_archive_recall_and_snapshot_replay() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(3,5,1,10).unwrap();
    let source=log.cycle().unwrap();let first=log.extract(source,&[("elemento","alfa")],&[]).unwrap();
    log.close_selection_window(first).unwrap();log.evaluate_memory(first).unwrap();
    let id=V2Log::node_id("elemento","alfa");
    let origin=log.nodes()[&id].origin_seq;
    let source=log.cycle().unwrap();let e=log.extract(source,&[("elemento","alfa")],&[]).unwrap();log.evaluate_memory(e).unwrap();
    assert_eq!(log.memory()[&id],MemoryStatus::Long);
    for _ in 3..=253 {log.cycle().unwrap();}
    let source=log.events().iter().rev().find(|e|e.kind=="SENSE").unwrap().seq;
    let e=log.extract(source,&[("elemento","beta")],&[]).unwrap();log.evaluate_memory(e).unwrap();
    let archive=match log.memory()[&id] {MemoryStatus::Archived{at}=>at,_=>panic!("not archived")};
    let source=log.cycle().unwrap();assert!(source>archive);
    let e=log.extract(source,&[("elemento","alfa")],&[]).unwrap();log.evaluate_memory(e).unwrap();
    assert_eq!(log.memory()[&id],MemoryStatus::Circuit);assert_eq!(log.nodes()[&id].origin_seq,origin);
    let replay=V2Log::from_jsonl(&log.to_jsonl(),"2",&[]).unwrap();
    assert_eq!(replay.self_document(),log.self_document());
    let doc=log.self_document();
    let pre=Json::object([("head_hash",doc.get("head_hash").unwrap().clone()),("schema_version","2".into()),("statements",doc.get("statements").unwrap().clone())]);
    assert_eq!(doc.get("self_hash").unwrap().string().unwrap(),hash(&pre.canonical()));
}

#[test]
fn errors_leave_every_event_unchanged() {
    let mut log=V2Log::genesis(1042).unwrap();let before=log.to_jsonl();
    assert_eq!(log.configure(3,0,1,10),Err(Error::Config));assert_eq!(log.to_jsonl(),before);
    let source=log.cycle().unwrap();let before=log.to_jsonl();
    assert_eq!(log.extract(source,&[("elemento","alfa")],&[("elemento","alfa","elemento","missing","x")]),Err(Error::Reference));
    assert_eq!(log.to_jsonl(),before);assert!(log.nodes().is_empty());
    assert_eq!(log.close_selection_window(999),Err(Error::Config));assert_eq!(log.to_jsonl(),before);
}

#[test]
fn incomplete_batches_tampering_and_wrong_versions_are_rejected() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(3,5,1,10).unwrap();
    let source=log.cycle().unwrap();let e=log.extract(source,&[("x","a")],&[]).unwrap();log.close_selection_window(e).unwrap();
    let text=log.to_jsonl();
    assert!(V2Log::from_jsonl(&text,"1",&[]).is_err());
    assert!(V2Log::from_jsonl(&text.replace("\"tau\\\":5","\"tau\\\":6"),"2",&[]).is_err());
    let lines:Vec<_>=text.lines().collect();
    for i in 1..lines.len() {
        let e=&log.events()[i];
        if e.kind=="AUX" && e.payload.get("kind").unwrap().string().unwrap()=="barrier" && data(&log,i).get("phase").unwrap().string().unwrap()=="close" {
            let prefix=format!("{}\n",lines[..=i].join("\n"));assert!(V2Log::from_jsonl(&prefix,"2",&[]).is_err());
        }
    }
    assert!(ariadne::Log::from_jsonl(&text).is_err());
}

#[test]
fn l5_fixture_gate_and_agreement_disagreement_replay() {
    let one="{\"observations\":[{\"agente\":\"agente_alpha\",\"confidenza\":1000,\"ordine_tom\":0,\"proposizione\":\"1\",\"soggetto\":\"agente_alpha\",\"sources\":[0]}]}";
    let two=one.replace("\"proposizione\":\"1\"","\"proposizione\":\"2\"");
    let d1=hash(one);let d2=hash(&two);
    let f1=Fixture{id:"one",sha256:&d1,bytes:one,anchor:0};let f2=Fixture{id:"two",sha256:&d2,bytes:&two,anchor:0};
    let mut log=V2Log::genesis(1042).unwrap();
    let id=hash("{\"altro\":\"agente_alpha\"}");
    log.observe(&f1).unwrap();assert_eq!(log.predict(&id).unwrap(),1);log.observe(&f1).unwrap();
    assert_eq!(log.predict(&id).unwrap(),1);log.observe(&f2).unwrap();
    assert_eq!(log.social()[&id].counts[&1],2);assert_eq!(log.social()[&id].counts[&2],1);
    let updates=log.events().iter().filter(|e|e.kind=="AUX"&&e.payload.get("kind").unwrap().string().unwrap()=="model_update").count();assert_eq!(updates,1);
    let before=log.to_jsonl();let negative=Fixture{id:"one",sha256:&d1,bytes:&two,anchor:0};
    assert_eq!(log.observe(&negative),Err(Error::DigestMismatch));assert_eq!(log.to_jsonl(),before);
    assert!(V2Log::from_jsonl(&before,"2",&[]).is_err());log.verify(&[f1,f2]).unwrap();
}

#[test]
fn open_window_blocks_cycles_and_non_l2_without_implicit_close() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(3,5,1,10).unwrap();
    let source=log.cycle().unwrap();let e=log.extract_synthetic(source).unwrap();
    log.open_selection_window(e).unwrap();assert!(log.window_is_open());
    let before=log.to_jsonl();
    assert_eq!(log.cycle(),Err(Error::WindowState));
    assert_eq!(log.configure(1,5,1,10),Err(Error::WindowState));
    assert_eq!(log.evaluate_memory(e),Err(Error::WindowState));
    assert_eq!(log.open_selection_window(e),Err(Error::WindowState));
    assert_eq!(log.to_jsonl(),before);
    assert!(V2Log::from_jsonl(&before,"2",&[]).unwrap().window_is_open());
    log.close_window().unwrap();assert!(!log.window_is_open());
    let before=log.to_jsonl();assert_eq!(log.close_window(),Err(Error::WindowState));assert_eq!(log.to_jsonl(),before);
    log.verify(&[]).unwrap();
}

#[test]
fn duplicate_judgments_are_not_reemitted_and_sources_are_deduplicated() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(3,5,0,10).unwrap();
    let source=log.cycle().unwrap();let e=log.extract(source,&[("x","a"),("x","a")],&[]).unwrap();
    log.close_selection_window(e).unwrap();log.evaluate_memory(e).unwrap();
    let before=log.to_jsonl();
    assert_eq!(log.close_selection_window(e),Err(Error::Reference));
    assert_eq!(log.evaluate_memory(e),Err(Error::Reference));assert_eq!(log.to_jsonl(),before);
    log.verify(&[]).unwrap();
}

#[test]
fn integrated_four_node_sequence_has_preregisterable_positions() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(3,5,1,10).unwrap();
    for tick in 1..=254 {
        let source=log.cycle().unwrap();
        if [1,2,253,254].contains(&tick) {
            let e=log.extract_synthetic(source).unwrap();
            log.close_selection_window(e).unwrap();log.evaluate_memory(e).unwrap();
        }
    }
    // Derived from the operation schedule, not from the replay result.
    for (seq,kind) in [(6,"extract"),(10,"select"),(20,"extract"),(29,"promote"),(785,"extract"),(794,"archive"),(797,"archive"),(800,"archive"),(803,"promote"),(809,"extract"),(818,"recall")] {
        assert_eq!(log.events()[seq].payload.get("kind").unwrap().string().unwrap(),kind,"seq {seq}");
    }
    assert_eq!(log.events().len(),821);
    let replay=V2Log::from_jsonl(&log.to_jsonl(),"2",&[]).unwrap();
    assert_eq!(replay.graph_document(),log.graph_document());
    assert_eq!(replay.memory_document(),log.memory_document());
    assert_eq!(replay.active_document(),log.active_document());
    assert_eq!(replay.self_document(),log.self_document());
    // Hash every record independently from its exposed fields.
    for e in log.events() {
        let pre=format!("{}\x1f{}\x1f{}\x1f{}\x1f{}",e.seq,e.tick,e.kind,e.payload.canonical(),e.prev_hash);
        assert_eq!(hash(&pre),e.hash);
    }
}

#[test]
fn second_order_is_identity_projection_not_an_independent_mind() {
    let bytes="{\"observations\":[{\"agente\":\"a\",\"confidenza\":0,\"ordine_tom\":0,\"proposizione\":\"2\",\"soggetto\":\"a\",\"sources\":[0]},{\"agente\":\"b\",\"confidenza\":1000,\"ordine_tom\":0,\"proposizione\":\"1\",\"soggetto\":\"b\",\"sources\":[0]},{\"agente\":\"a\",\"confidenza\":1000,\"ordine_tom\":1,\"proposizione\":\"9\",\"soggetto\":\"b\",\"sources\":[0]}]}";
    let digest=hash(bytes);let fixture=Fixture{id:"composition",bytes,sha256:&digest,anchor:0};
    let mut log=V2Log::genesis(1042).unwrap();log.observe(&fixture).unwrap();
    let a=hash("{\"altro\":\"a\"}");let b=hash("{\"altro\":\"b\"}");
    assert_eq!(log.project_prediction(&a,&b).unwrap(),log.prediction_document(&b).unwrap());
    assert_eq!(log.social()[&a].counts.len(),1);assert_eq!(log.social()[&a].counts[&2],1);
    let before=log.to_jsonl();log.project_prediction(&a,&b).unwrap();assert_eq!(before,log.to_jsonl());
    log.verify(&[fixture]).unwrap();
}

#[test]
fn semantic_tampering_is_rejected_even_with_a_rebuilt_hash_chain() {
    let mut log=V2Log::genesis(1042).unwrap();log.configure(1,5,1,10).unwrap();
    let source=log.cycle().unwrap();let e=log.extract_synthetic(source).unwrap();log.close_selection_window(e).unwrap();
    let mut records=log.events().to_vec();
    let select=records.iter_mut().find(|e|e.kind=="AUX"&&e.payload.get("kind").unwrap().string().unwrap()=="select").unwrap();
    let mut d=Json::parse(select.payload.get("data").unwrap().string().unwrap()).unwrap();
    if let Json::Object(ref mut fields)=d {fields.insert("ups".into(),Json::Array(vec!["not-a-node".into()]));}
    if let Json::Object(ref mut fields)=select.payload {fields.insert("data".into(),d.canonical().into());}
    let mut previous="0".repeat(64);
    for record in &mut records {
        record.prev_hash=previous;
        record.hash=hash(&format!("{}\x1f{}\x1f{}\x1f{}\x1f{}",record.seq,record.tick,record.kind,record.payload.canonical(),record.prev_hash));
        previous=record.hash.clone();
    }
    let text:String=records.iter().map(|e|format!("{}\n",e.canonical())).collect();
    assert!(matches!(V2Log::from_jsonl(&text,"2",&[]),Err(Error::Mismatch)));
}

#[test]
fn fixture_identifier_cannot_be_rebound_and_bad_anchor_rolls_back() {
    let one="{\"observations\":[]}";
    let d=hash(one);let f=Fixture{id:"fixed",bytes:one,sha256:&d,anchor:0};
    let mut log=V2Log::genesis(1042).unwrap();log.observe(&f).unwrap();let before=log.to_jsonl();
    let bad=Fixture{id:"fixed",bytes:one,sha256:&d,anchor:1};
    assert_eq!(log.observe(&bad),Err(Error::Mismatch));assert_eq!(log.to_jsonl(),before);
    let bad=Fixture{id:"other",bytes:one,sha256:&d,anchor:1};
    assert_eq!(log.observe(&bad),Err(Error::Reference));assert_eq!(log.to_jsonl(),before);
    log.verify(&[f]).unwrap();
}

#[test]
fn ratified_l3_promote_order_is_origin_then_id_ascending() {
    // Decisione ratificata 2026-11-05: dentro promote, origin_seq crescente
    // e poi ID crescente. T=0 promuove entrambe le identità nello stesso
    // giudizio, con la stessa origine.
    let mut log=V2Log::genesis(1042).unwrap();
    log.configure(3,5,0,10).unwrap();
    let source=log.cycle().unwrap();
    let extraction=log.extract(source,&[("elemento","gamma"),("elemento","alfa")],&[]).unwrap();
    log.close_selection_window(extraction).unwrap();
    log.evaluate_memory(extraction).unwrap();
    let promoted:Vec<String>=log.events().iter().filter(|e| {
        e.kind=="AUX" && e.payload.get("level").and_then(|j|j.string()).ok()==Some("L3")
            && e.payload.get("kind").and_then(|j|j.string()).ok()==Some("promote")
    }).map(|e| {
        Json::parse(e.payload.get("data").unwrap().string().unwrap()).unwrap()
            .get("id").unwrap().string().unwrap().to_owned()
    }).collect();
    let mut expected=vec![V2Log::node_id("elemento","alfa"),V2Log::node_id("elemento","gamma")];
    expected.sort();
    assert_eq!(promoted,expected);
    log.verify(&[]).unwrap();
}

#[test]
fn ratified_l3_archive_order_is_origin_descending() {
    // Decisione ratificata 2026-11-05: dentro archive, origin_seq
    // decrescente (poi ID decrescente). Al tick 27 entrambe le identità
    // hanno punteggio zero; gamma ha origine successiva e viene prima.
    let mut log=V2Log::genesis(1042).unwrap();
    log.configure(3,5,1,1).unwrap();
    let source=log.cycle().unwrap();
    let e=log.extract(source,&[("elemento","alfa")],&[]).unwrap();
    log.close_selection_window(e).unwrap();
    log.evaluate_memory(e).unwrap();
    let source=log.cycle().unwrap();
    let e=log.extract(source,&[("elemento","gamma")],&[]).unwrap();
    log.close_selection_window(e).unwrap();
    log.evaluate_memory(e).unwrap();
    for _ in 3..=27 {log.cycle().unwrap();}
    let source=log.cycle().unwrap();
    let e=log.extract(source,&[("elemento","delta")],&[]).unwrap();
    log.close_selection_window(e).unwrap();
    log.evaluate_memory(e).unwrap();
    let archived:Vec<String>=log.events().iter().filter(|e| {
        e.kind=="AUX" && e.payload.get("level").and_then(|j|j.string()).ok()==Some("L3")
            && e.payload.get("kind").and_then(|j|j.string()).ok()==Some("archive")
    }).map(|e| {
        Json::parse(e.payload.get("data").unwrap().string().unwrap()).unwrap()
            .get("id").unwrap().string().unwrap().to_owned()
    }).collect();
    assert_eq!(archived,vec![V2Log::node_id("elemento","gamma"),V2Log::node_id("elemento","alfa")]);
    log.verify(&[]).unwrap();
}
