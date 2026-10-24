use ariadne::{selection::{self, Decision}, view, EventType, Log, Payload, Value};

fn append<K: AsRef<str>>(log: &mut Log, tick: u64, pairs: Vec<(K, Value)>) {
    let pairs: Vec<(String, Value)> = pairs.into_iter().map(|(k, v)| (k.as_ref().to_owned(), v)).collect();
    log.append(tick, EventType::Action, Payload::from_pairs(pairs)).unwrap();
}
fn l2(log: &mut Log, tick: u64, tag: &str, pairs: Vec<(&str, Value)>) {
    let mut p = vec![("l2", Value::Str(tag.into()))]; p.extend(pairs); append(log, tick, p);
}
fn nodes(keys: &[&str]) -> Value {
    Value::Str(format!("[{}]", keys.iter().map(|k| format!("{{\"key\":\"{k}\",\"kind\":\"letter\"}}")).collect::<Vec<_>>().join(",")))
}

#[test]
fn l0_l1_l2_pipeline_is_replayable_and_provenanced() {
    let mut log = Log::genesis(41).unwrap();
    l2(&mut log, 0, "config", vec![("k", Value::Int(1)), ("tau", Value::Int(1))]);
    append(&mut log, 1, vec![("nodes", nodes(&["A", "B", "C"])), ("edges", Value::Str("[{\"dst_key\":\"C\",\"dst_kind\":\"letter\",\"rel\":\"rel\",\"src_key\":\"A\",\"src_kind\":\"letter\"}]".into()))]);
    let base = selection::build_active(&log).unwrap();
    let before = log.to_jsonl();
    let v = view::build(&log).unwrap();
    assert_eq!(log.to_jsonl(), before);
    assert_eq!(v.head_hash, log.events().last().unwrap().hash);
    let a = v.nodes.iter().find(|n| n.key == "A").unwrap();
    let b = v.nodes.iter().find(|n| n.key == "B").unwrap();
    let c = v.nodes.iter().find(|n| n.key == "C").unwrap();
    assert_eq!((a.origin_seq, b.origin_seq, c.origin_seq), (2, 2, 2));
    assert_eq!(v.edges.len(), 1);
    assert_eq!(v.edges[0].origin_seq, 2);

    l2(&mut log, 3, "barrier", vec![]);
    let window_select = Decision::Select { at: 2, tick: 1, ups: vec![], downs: vec![2] };
    append(&mut log, 3, selection::encode(&window_select));
    append(&mut log, 4, vec![("nodes", nodes(&["D"])), ("edges", Value::Str("[{\"dst_key\":\"A\",\"dst_kind\":\"letter\",\"rel\":\"cycle\",\"src_key\":\"D\",\"src_kind\":\"letter\"}]".into()))]);
    let before_eval = log.to_jsonl();
    let decisions = selection::evaluate(&log, &base).unwrap();
    let summary_at = decisions.iter().position(|d| matches!(d, Decision::Summary { .. })).unwrap();
    let close_select_at = decisions.iter().position(|d| matches!(d, Decision::Select { at: 5, .. })).unwrap();
    assert!(summary_at < close_select_at);
    assert!(matches!(&decisions[summary_at], Decision::Summary { expulsions, .. } if expulsions == &vec![2]));
    let again = selection::evaluate(&log, &base).unwrap();
    assert_eq!(decisions, again);
    assert_eq!(before_eval, log.to_jsonl());
    log.verify().unwrap();
    let active_a = selection::build_active(&log).unwrap();
    let active_b = selection::build_active(&log).unwrap();
    assert_eq!(active_a, active_b);
    let events = log.events();
    for (i, e) in events.iter().enumerate() {
        assert_eq!(e.seq, i as u64);
        if i > 0 { assert_eq!(e.prev_hash, events[i-1].hash); }
    }
}
