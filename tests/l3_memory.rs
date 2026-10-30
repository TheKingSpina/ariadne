use ariadne::{memory::{self, L3Error}, EventType, Log, Payload, Value};

fn action(log:&mut Log,tick:u64,tag:&str,fields:Vec<(&str,Value)>) {
    let mut pairs=vec![("l3",Value::Str(tag.into()))]; pairs.extend(fields);
    log.append(tick,EventType::Action,Payload::from_pairs(pairs)).unwrap();
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
    let a=Log::genesis(1042).unwrap();
    let b=Log::genesis(1043).unwrap();
    let base=memory::build_memory(&a).unwrap_or_else(|_| panic!("stub expected to be replaced"));
    assert_eq!(memory::evaluate(&b,&base).unwrap_err(),L3Error::HeadMatch);
}
