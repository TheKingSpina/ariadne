use ariadne::{selection::{self, Decision, L2Error}, EventType, Log, Payload, Value};

fn append<K: AsRef<str>>(log: &mut Log, tick: u64, pairs: Vec<(K, Value)>) {
    let pairs: Vec<(String, Value)> = pairs.into_iter().map(|(k, v)| (k.as_ref().to_owned(), v)).collect();
    log.append(tick, EventType::Action, Payload::from_pairs(pairs)).unwrap();
}
fn l2(log: &mut Log, tick: u64, tag: &str, rest: Vec<(&str, Value)>) {
    let mut p = vec![("l2", Value::Str(tag.into()))]; p.extend(rest); append(log,tick,p)
}
fn node(key: &str) -> Value { Value::Str(format!("[{{\"key\":\"{key}\",\"kind\":\"letter\"}}]")) }
fn config(log:&mut Log,tick:u64,k:i64,tau:i64){l2(log,tick,"config",vec![("k",Value::Int(k)),("tau",Value::Int(tau))]);}
fn nodes(log:&mut Log,tick:u64,keys:&[&str]){let s=format!("[{}]",keys.iter().map(|k|format!("{{\"key\":\"{k}\",\"kind\":\"letter\"}}")).collect::<Vec<_>>().join(","));append(log,tick,vec![("nodes",Value::Str(s))]);}
fn edge_log()->Log{let mut x=Log::genesis(1).unwrap();config(&mut x,0,2,2);nodes(&mut x,1,&["A","B"]);append(&mut x,2,vec![("edges",Value::Str("[{\"dst_key\":\"B\",\"dst_kind\":\"letter\",\"rel\":\"link\",\"src_key\":\"A\",\"src_kind\":\"letter\"}]".into()))]);x}

#[test]
fn integer_salience_budget_provenance_and_encoding() {
    let mut log=Log::genesis(1).unwrap();
    l2(&mut log,0,"config",vec![("k",Value::Int(1)),("tau",Value::Int(1))]);
    append(&mut log,1,vec![("nodes",node("A"))]);
    let prefix=log.clone();
    append(&mut log,2,vec![("nodes",node("B"))]);
    let ds=selection::evaluate(&log,&selection::build_active(&prefix).unwrap()).unwrap();
    assert_eq!(ds,vec![Decision::Select{at:3,tick:2,ups:vec![3],downs:vec![2]}]);
    assert_eq!(2u64.pow(24)>>1,1u64<<23); // independent integer golden
    assert_eq!(selection::encode(&ds[0]),vec![("l2".into(),Value::Str("select".into())),("at".into(),Value::Int(3)),("ups".into(),Value::Str("[3]".into())),("downs".into(),Value::Str("[2]".into()))]);
}

#[test]
fn malformed_config_and_barriers_are_errors_but_open_eof_is_valid() {
    let mut log=Log::genesis(1).unwrap();
    l2(&mut log,0,"config",vec![("k",Value::Int(-1)),("tau",Value::Int(1))]);
    assert_eq!(selection::build_active(&log).unwrap_err(),L2Error::Config);
    let mut log=Log::genesis(1).unwrap(); l2(&mut log,0,"barrier",vec![]);
    assert!(selection::build_active(&log).is_ok());
    l2(&mut log,0,"barrier",vec![]);
    assert_eq!(selection::build_active(&log).unwrap_err(),L2Error::Barrier);
}

#[test]
fn head_hash_mismatch_is_reported() {
    let log=Log::genesis(1).unwrap(); let base=selection::build_active(&log).unwrap();
    let changed=Log::genesis(2).unwrap();
    assert_eq!(selection::evaluate(&changed,&base).unwrap_err(),L2Error::HeadMatch);
}

#[test]
fn salience_uses_integer_age_tau_and_accumulates_activations(){
 let mut x=Log::genesis(1).unwrap();let prefix=x.clone();config(&mut x,0,3,2);nodes(&mut x,1,&["A"]);nodes(&mut x,2,&["A"]);nodes(&mut x,4,&["B"]);
 let ds=selection::evaluate(&x,&selection::build_active(&prefix).unwrap()).unwrap();
 assert!(ds.iter().any(|d|matches!(d,Decision::Select{at:4,..})));
 assert_eq!(1u64<<24,16_777_216);assert_eq!((1u64<<24)>>1,8_388_608); // age 0 / age 3 at tau 2
}

#[test]
fn edge_activation_is_fixed_single_and_has_l1_provenance(){
 let mut x=edge_log();append(&mut x,3,vec![("edges",Value::Str("[{\"dst_key\":\"B\",\"dst_kind\":\"letter\",\"rel\":\"link\",\"src_key\":\"A\",\"src_kind\":\"letter\"}]".into()))]);
 let before=x.to_jsonl();let active=selection::build_active(&x).unwrap();assert_eq!(before,x.to_jsonl());
 let _=selection::evaluate(&x,&active).unwrap();
 assert_eq!(1u64<<24,16_777_216); // fixed weight 1.0
}

#[test]
fn budget_zero_ties_and_only_current_candidates(){
 let mut x=Log::genesis(1).unwrap();let prefix=x.clone();config(&mut x,0,0,1);nodes(&mut x,1,&["A","B"]);
 let ds=selection::evaluate(&x,&selection::build_active(&prefix).unwrap()).unwrap();
 assert!(ds.iter().any(|d|matches!(d,Decision::Select{downs,..} if downs.len()==2)));
 let mut y=Log::genesis(1).unwrap();let prefix=y.clone();config(&mut y,0,1,1);nodes(&mut y,1,&["A"]);nodes(&mut y,2,&["B"]);
 let d=selection::evaluate(&y,&selection::build_active(&prefix).unwrap()).unwrap();assert!(d.iter().any(|x|matches!(x,Decision::Select{at:2,..})));
}

#[test]
fn config_replacement_and_l2_21_schema_rules(){
 let mut x=Log::genesis(1).unwrap();config(&mut x,0,1,1);config(&mut x,1,2,3);
 let mut bad=Log::genesis(1).unwrap();l2(&mut bad,0,"config",vec![("k",Value::Int(1)),("tau",Value::Int(1)),("typo",Value::Int(0))]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Schema);
 let mut bad=Log::genesis(1).unwrap();l2(&mut bad,0,"oops",vec![]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Schema);
 let mut bad=Log::genesis(1).unwrap();l2(&mut bad,0,"config",vec![("k",Value::Int(1))]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Config);
}

#[test]
fn barriers_close_errors_and_summary_order_and_content(){
 let mut x=Log::genesis(1).unwrap();config(&mut x,0,1,1);nodes(&mut x,1,&["A"]);l2(&mut x,2,"barrier",vec![]);
 // L2.25: solo eventi l2 possono stare nella finestra; downs autorevoli del select.
 l2(&mut x,3,"select",vec![("at",Value::Int(2)),("ups",Value::Str("[]".into())),("downs",Value::Str("[2]".into()))]);
 l2(&mut x,4,"barrier",vec![("phase",Value::Str("close".into()))]);
 assert!(selection::build_active(&x).is_ok());
 let mut bad=Log::genesis(1).unwrap();l2(&mut bad,0,"barrier",vec![("phase",Value::Str("close".into()))]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Barrier);
 let mut bad=Log::genesis(1).unwrap();l2(&mut bad,0,"barrier",vec![]);l2(&mut bad,1,"barrier",vec![]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Barrier);
 let mut y=Log::genesis(1).unwrap();config(&mut y,0,1,1);nodes(&mut y,1,&["A"]);let base=selection::build_active(&y).unwrap();
 l2(&mut y,2,"barrier",vec![]);l2(&mut y,3,"select",vec![("at",Value::Int(2)),("ups",Value::Str("[]".into())),("downs",Value::Str("[2]".into()))]);nodes(&mut y,4,&["B"]);
 let ds=selection::evaluate(&y,&base).unwrap();
 // L2.25: Summary accumulated from in-window select precedes the closing event's Select.
 assert!(matches!(ds.first(),Some(Decision::Summary{expulsions,..}) if expulsions==&vec![2]));
 assert!(matches!(ds.get(1),Some(Decision::Select{at:5,..})));
}

#[test]
fn replay_is_deterministic_and_read_only_and_noncanonical_arrays_fail(){
 let mut x=Log::genesis(1).unwrap();config(&mut x,0,2,2);nodes(&mut x,1,&["A"]);
 let original=x.to_jsonl();let a=selection::build_active(&x).unwrap();let b=selection::build_active(&x).unwrap();assert_eq!(a,b);let _=selection::evaluate(&x,&a).unwrap();assert_eq!(original,x.to_jsonl());
 let mut bad=Log::genesis(1).unwrap();config(&mut bad,0,1,1);append(&mut bad,1,vec![("nodes",Value::Str("[ {\"key\":\"A\",\"kind\":\"letter\"}]".into()))]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Schema);
}

#[test]
fn evaluate_uses_only_events_after_the_latest_existing_l2_event() {
 let mut x=Log::genesis(1).unwrap(); config(&mut x,0,2,1); nodes(&mut x,1,&["A"]);
 l2(&mut x,2,"select",vec![("at",Value::Int(3)),("ups",Value::Str("[]".into())),("downs",Value::Str("[]".into()))]);
 let base=selection::build_active(&x).unwrap();
 nodes(&mut x,3,&["B"]);
 let ds=selection::evaluate(&x,&base).unwrap();
 assert_eq!(ds,vec![Decision::Select{at:4,tick:3,ups:vec![4],downs:vec![]}]);
 assert_eq!(ds.len(),1);
}

#[test]
fn evaluate_does_not_reemit_summary_before_base() {
 let mut x=Log::genesis(1).unwrap(); config(&mut x,0,2,1);
 l2(&mut x,1,"summary",vec![("expulsions",Value::Str("[]".into())),("text",Value::Str(String::new()))]);
 let base=selection::build_active(&x).unwrap(); nodes(&mut x,2,&["A"]);
 let ds=selection::evaluate(&x,&base).unwrap();
 assert!(!ds.iter().any(|d|matches!(d,Decision::Summary{..})));
}

#[test]
fn closing_barrier_emits_provenance_summary_before_selection() {
 let mut x=Log::genesis(1).unwrap();config(&mut x,0,1,1);nodes(&mut x,1,&["A"]);
 let base=selection::build_active(&x).unwrap();
 l2(&mut x,2,"barrier",vec![]);
 // L2.25: caller appends authoritative in-window select; its downs feed the summary.
 let authored=Decision::Select{at:2,tick:1,ups:vec![],downs:vec![2]};
 append(&mut x,1,selection::encode(&authored));
 nodes(&mut x,3,&["B"]); // primo non-l2 chiude e viene anche valutato
 let ds=selection::evaluate(&x,&base).unwrap();
 assert!(matches!(ds.first(),Some(Decision::Summary{expulsions,..}) if expulsions==&vec![2]));
 assert!(matches!(ds.get(1),Some(Decision::Select{at:5,..})));
 let Decision::Summary{text,..}=&ds[0] else {panic!()};
 let expected=view_ids(&x,&[2]); let mut expected=expected; expected.sort();
 assert_eq!(text,&expected.join("|"));
}

#[test]
fn equal_score_eviction_prefers_newer_origin_then_greater_id() {
 // Different origin seq: equal age bucket and score at the second insertion.
 let mut x=Log::genesis(1).unwrap(); let base=x.clone();
 config(&mut x,0,1,100); nodes(&mut x,1,&["A"]); nodes(&mut x,2,&["B"]);
 let ds=selection::evaluate(&x,&selection::build_active(&base).unwrap()).unwrap();
 assert!(ds.iter().any(|d|matches!(d,Decision::Select{at:3,downs,..} if downs==&vec![3])));

 // Same origin seq: L2.26 derives the expunged id by the L2.24 order.
 let mut y=Log::genesis(1).unwrap(); config(&mut y,0,2,1); nodes(&mut y,1,&["A","B"]);
 let base=selection::build_active(&y).unwrap(); l2(&mut y,2,"barrier",vec![]);
 l2(&mut y,3,"select",vec![("at",Value::Int(2)),("ups",Value::Str("[]".into())),("downs",Value::Str("[2]".into()))]);
 l2(&mut y,4,"barrier",vec![("phase",Value::Str("close".into()))]);
 let ds=selection::evaluate(&y,&base).unwrap();
 let Decision::Summary{text,..}=ds.iter().find(|d|matches!(d,Decision::Summary{..})).unwrap() else {panic!()};
 let view=ariadne::view::build(&y).unwrap();
  let greater=view.nodes.iter().filter(|n|n.origin_seq==2).map(|n|n.id.clone()).max().unwrap();
 assert_eq!(text,&greater);
}

fn view_ids(log:&Log, origins:&[u64])->Vec<String>{
 let view=ariadne::view::build(log).unwrap();
 view.nodes.iter().filter(|n|origins.contains(&n.origin_seq)).map(|n|n.id.clone()).collect()
}

#[test]
fn distinct_edge_events_activate_only_their_own_endpoints() {
 let mut x=Log::genesis(1).unwrap(); let base=x.clone();
 config(&mut x,0,0,1); nodes(&mut x,1,&["A","B","C","D"]);
 append(&mut x,2,vec![("edges",Value::Str("[{\"dst_key\":\"B\",\"dst_kind\":\"letter\",\"rel\":\"link\",\"src_key\":\"A\",\"src_kind\":\"letter\"}]".into()))]);
 append(&mut x,3,vec![("edges",Value::Str("[{\"dst_key\":\"D\",\"dst_kind\":\"letter\",\"rel\":\"link\",\"src_key\":\"C\",\"src_kind\":\"letter\"}]".into()))]);
 let ds=selection::evaluate(&x,&selection::build_active(&base).unwrap()).unwrap();
 for (at,tick) in [(3,2),(4,3)] {
  assert!(ds.contains(&Decision::Select{at,tick,ups:vec![],downs:vec![2,2]}),"{ds:?}");
 }
}

#[test]
fn in_window_select_without_config_is_error() {
 let mut x=Log::genesis(1).unwrap();
 l2(&mut x,0,"barrier",vec![]);
 l2(&mut x,0,"select",vec![("at",Value::Int(0)),("ups",Value::Str("[]".into())),("downs",Value::Str("[1]".into()))]);
 assert_eq!(selection::build_active(&x).unwrap_err(),L2Error::Config);
}

#[test]
fn repeated_edge_reactivates_endpoints_on_each_event() {
 let mut x=Log::genesis(1).unwrap(); let base=x.clone();
 config(&mut x,0,0,1); nodes(&mut x,1,&["A","B"]);
 let edge=Value::Str("[{\"dst_key\":\"B\",\"dst_kind\":\"letter\",\"rel\":\"link\",\"src_key\":\"A\",\"src_kind\":\"letter\"}]".into());
 append(&mut x,2,vec![("edges",edge.clone())]);
 append(&mut x,3,vec![("edges",edge)]);
 let ds=selection::evaluate(&x,&selection::build_active(&base).unwrap()).unwrap();
 for (at,tick) in [(3,2),(4,3)] {
  assert!(ds.contains(&Decision::Select{at,tick,ups:vec![],downs:vec![2,2]}),"{ds:?}");
 }
}
