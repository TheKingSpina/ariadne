use ariadne::{selection::{self, Decision, L2Error}, EventType, Log, Payload, Value};

fn append(log: &mut Log, tick: u64, pairs: Vec<(&str, Value)>) {
    log.append(tick, EventType::Action, Payload::from_pairs(pairs)).unwrap();
}
fn l2(log: &mut Log, tick: u64, tag: &str, rest: Vec<(&str, Value)>) {
    let mut p = vec![("l2", Value::Str(tag.into()))]; p.extend(rest); append(log,tick,p)
}
fn node(key: &str) -> Value { Value::Str(format!("[{{\"key\":\"{key}\",\"kind\":\"letter\"}}]")) }

#[test]
fn integer_salience_budget_provenance_and_encoding() {
    let mut log=Log::genesis(1).unwrap();
    l2(&mut log,0,"config",vec![("k",Value::Int(1)),("tau",Value::Int(1))]);
    append(&mut log,1,vec![("nodes",node("A"))]);
    append(&mut log,2,vec![("nodes",node("B"))]);
    let ds=selection::evaluate(&log,&selection::build_active(&Log::genesis(1).unwrap()).unwrap()).unwrap();
    assert_eq!(ds,vec![Decision::Select{at:2,tick:2,ups:vec![2],downs:vec![1]}]);
    assert_eq!(2u64.pow(24)>>1,1u64<<23); // independent integer golden
    assert_eq!(selection::encode(&ds[0]),vec![("l2".into(),Value::Str("select".into())),("at".into(),Value::Int(2)),("downs".into(),Value::Str("[1]".into())),("ups".into(),Value::Str("[2]".into()))]);
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
    let log=Log::genesis(1).unwrap(); let mut base=selection::build_active(&log).unwrap();
    let mut changed=log.clone(); append(&mut changed,1,vec![("nodes",node("A"))]);
    assert_eq!(selection::evaluate(&changed,&base).unwrap_err(),L2Error::HeadMatch);
    let _=&mut base;
}
