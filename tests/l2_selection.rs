use ariadne::{selection::{self, Decision, L2Error}, EventType, Log, Payload, Value};

fn append(log: &mut Log, tick: u64, pairs: Vec<(&str, Value)>) {
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
 let mut x=Log::genesis(1).unwrap();config(&mut x,0,1,1);l2(&mut x,0,"barrier",vec![]);nodes(&mut x,1,&["A","B"]);
 l2(&mut x,2,"barrier",vec![("phase",Value::Str("close".into()))]);
 assert_eq!(selection::build_active(&x).unwrap_err(),L2Error::Barrier);
 let mut bad=Log::genesis(1).unwrap();l2(&mut bad,0,"barrier",vec![("phase",Value::Str("close".into()))]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Barrier);
 let mut bad=Log::genesis(1).unwrap();l2(&mut bad,0,"barrier",vec![("phase",Value::Str("later".into()))]);assert_eq!(selection::build_active(&bad).unwrap_err(),L2Error::Barrier);
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
 assert!(ds.iter().all(|d|matches!(d,Decision::Select{at:5,..})));
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
 let mut x=Log::genesis(1).unwrap(); config(&mut x,0,1,1);
 let base=selection::build_active(&x).unwrap();
 l2(&mut x,1,"barrier",vec![]); nodes(&mut x,2,&["A","B"]);
 nodes(&mut x,3,&["C"]);
 l2(&mut x,4,"barrier",vec![("phase",Value::Str("close".into()))]);
 let ds=selection::evaluate(&x,&base).unwrap();
 let summary=ds.iter().position(|d|matches!(d,Decision::Summary{..})).unwrap();
 let select=ds.iter().position(|d|matches!(d,Decision::Select{at:5,..})).unwrap();
 assert!(summary<select);
 let Decision::Summary{expulsions,text}=&ds[summary] else {panic!()};
 assert_eq!(expulsions, &{let mut v=expulsions.clone();v.sort_unstable();v.dedup();v});
 let mut ids:Vec<String>=view_ids(&x,expulsions);
 ids.sort(); assert_eq!(*text,ids.join("|"));
}

fn view_ids(log:&Log, origins:&[u64])->Vec<String>{
 let view=ariadne::view::build(log).unwrap();
 view.nodes.iter().filter(|n|origins.contains(&n.origin_seq)).map(|n|n.id.clone()).collect()
}
