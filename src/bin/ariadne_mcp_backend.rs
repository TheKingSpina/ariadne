//! Stateless subprocess backend for the stdio MCP adapter.
//! Receives one canonical JSON object on stdin and returns one JSON object.
use std::collections::BTreeSet;
use std::io::{self, Read};
use ariadne::{aux_v2::{chain_break, Fixture, Json, V2Log}, Log};

fn fixtures(input: &Json) -> Result<Vec<Fixture<'_>>, String> {
    let fixture_json=input.get("fixtures").map_err(|_|"missing fixtures")?.array().map_err(|_|"invalid fixtures")?;
    if fixture_json.len()>64 {return Err("too many fixtures".into());}
    let mut fixtures=Vec::new();
    for value in fixture_json {
        value.keys(&["anchor","bytes","fixture_id","sha256"]).map_err(|_|"invalid fixture schema")?;
        let anchor=value.get("anchor").map_err(|_|"invalid anchor")?.int().map_err(|_|"invalid anchor")?;
        if anchor<0 {return Err("invalid anchor".into());}
        fixtures.push(Fixture {
            id:value.get("fixture_id").map_err(|_|"invalid fixture")?.string().map_err(|_|"invalid fixture")?,
            sha256:value.get("sha256").map_err(|_|"invalid digest")?.string().map_err(|_|"invalid digest")?,
            bytes:value.get("bytes").map_err(|_|"invalid fixture bytes")?.string().map_err(|_|"invalid fixture bytes")?, anchor,
        });
    }
    Ok(fixtures)
}

fn field<'a>(value:&'a Json,key:&str)->Result<&'a str,String> {
    let text=value.get(key).map_err(|_|"missing member field")?.string().map_err(|_|"invalid member field")?;
    if text.is_empty() || text.len()>512 {return Err("empty or oversized member field".into());}
    Ok(text)
}

fn run() -> Result<Json, String> {
    let mut text=String::new();
    io::stdin().take(2_000_001).read_to_string(&mut text).map_err(|_|"invalid input".to_owned())?;
    if text.len()>2_000_000 {return Err("input too large".into());}
    let input=Json::parse(&text).map_err(|_|"invalid canonical input".to_owned())?;
    let operation=input.get("operation").map_err(|_|"missing operation")?.string().map_err(|_|"invalid operation")?;
    if operation=="create_v2" {
        input.keys(&["H","T","k","operation","seed","tau"]).map_err(|_|"invalid input schema")?;
        let n=|key|input.get(key).map_err(|_|"invalid parameter")?.int().map_err(|_|"invalid parameter");
        let seed=u64::try_from(n("seed")?).map_err(|_|"invalid seed")?;
        let mut log=V2Log::genesis(seed).map_err(|e|format!("v2 genesis rejected: {e}"))?;
        log.configure(n("k")?,n("tau")?,n("T")?,n("H")?).map_err(|e|format!("v2 config rejected: {e}"))?;
        return Ok(Json::object([("version","2".into()),("log_jsonl",log.to_jsonl().into()),("head_hash",log.events().last().ok_or("empty log")?.hash.clone().into())]));
    }
    if operation=="extract_v2" {
        input.keys(&["edges","fixtures","log_jsonl","nodes","operation"]).map_err(|_|"invalid input schema")?;
        let bindings=fixtures(&input)?;
        let bytes=input.get("log_jsonl").map_err(|_|"missing log")?.string().map_err(|_|"invalid log")?;
        let mut log=V2Log::from_jsonl(bytes,"2",&bindings).map_err(|e|match chain_break(bytes) {
            Some(seq)=>format!("v2 replay rejected a seq {seq}: {e}"),
            None=>format!("v2 replay rejected: {e}"),
        })?;
        let ns=input.get("nodes").map_err(|_|"missing nodes")?.array().map_err(|_|"invalid nodes")?;
        let es=input.get("edges").map_err(|_|"missing edges")?.array().map_err(|_|"invalid edges")?;
        if ns.len()+es.len()>256 {return Err("too many members".into());}
        let nodes:Vec<_>=ns.iter().map(|n|{n.keys(&["kind","key"]).map_err(|_|"invalid node schema")?;Ok((field(n,"kind")?,field(n,"key")?))}).collect::<Result<_,String>>()?;
        let edges:Vec<_>=es.iter().map(|e|{e.keys(&["src_kind","src_key","dst_kind","dst_key","rel"]).map_err(|_|"invalid edge schema")?;Ok((field(e,"src_kind")?,field(e,"src_key")?,field(e,"dst_kind")?,field(e,"dst_key")?,field(e,"rel")?))}).collect::<Result<_,String>>()?;
        if nodes.is_empty()&&edges.is_empty() {return Err("extract: servono almeno un nodo o un arco".into());}
        let mut warnings:Vec<Json> = Vec::new();
        let mut seen_nodes=BTreeSet::new();
        for (kind,key) in &nodes {
            if !seen_nodes.insert((*kind,*key)) {
                warnings.push(Json::from(format!("nodo duplicato {kind}/{key} in nodes")));
            }
        }
        let declared:BTreeSet<_>=seen_nodes.clone();
        let mut seen_edges=BTreeSet::new();
        for (index,(sk,sv,dk,dv,rel)) in edges.iter().enumerate() {
            if rel.is_empty() || !rel.bytes().all(|b| b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'_') {
                return Err(format!("extract: arco #{index} rel \"{rel}\" non valida; ammessi solo a-z, 0-9 e _"));
            }
            for (_role,kind,key) in [("src",*sk,*sv),("dst",*dk,*dv)] {
                if !declared.contains(&(kind,key)) && !log.has_node(kind,key) {
                    return Err(format!("extract: arco #{index}: endpoint {kind}/{key} non dichiarato in nodes e assente nel registro"));
                }
            }
            if !seen_edges.insert((sk,sv,dk,dv,rel)) {
                warnings.push(Json::from(format!("arco duplicato {sk}/{sv}->{dk}/{dv} rel {rel} in edges")));
            }
        }
        let source=log.cycle().map_err(|e|format!("v2 cycle rejected: {e}"))?;
        let at=log.extract(source,&nodes,&edges).map_err(|e|format!("v2 extract rejected: {e}"))?;
        log.close_selection_window(at).map_err(|e|format!("v2 selection rejected: {e}"))?;
        log.evaluate_memory(at).map_err(|e|format!("v2 memory rejected: {e}"))?;
        return Ok(Json::object([("version","2".into()),("log_jsonl",log.to_jsonl().into()),("source_seq",source.into()),("extraction_seq",at.into()),("warnings",Json::Array(warnings)),("head_hash",log.events().last().ok_or("empty log")?.hash.clone().into())]));
    }
    input.keys(&["fixtures","log_jsonl","operation","version"]).map_err(|_|"invalid input schema".to_owned())?;
    let version=input.get("version").map_err(|_|"missing version")?.string().map_err(|_|"invalid version")?;
    let bytes=input.get("log_jsonl").map_err(|_|"missing log")?.string().map_err(|_|"invalid log")?;
    let fixtures=fixtures(&input)?;
    match version {
        "1" => {
            if !fixtures.is_empty() {return Err("v1 does not accept fixtures".into());}
            let log=Log::from_jsonl(bytes).map_err(|_|"v1 log schema invalid")?;
            log.verify().map_err(|_|"v1 chain verification failed")?;
            match operation {
                "verify"=>Ok(Json::object([("events",i64::try_from(log.events().len()).map_err(|_|"too many events")?.into()),("head_hash",log.events().last().ok_or("empty log")?.hash.clone().into()),("version","1".into())])),
                "graph"=>{
                    let view=ariadne::view::build(&log).map_err(|_|"v1 graph projection failed")?;
                    Json::parse(&view.canonical()).map_err(|_|"v1 graph serialization failed".into())
                },
                _=>Err("view unsupported for v1".into()),
            }
        },
        "2"=>{
            let log=V2Log::from_jsonl(bytes,"2",&fixtures).map_err(|e|match chain_break(bytes) {
                Some(seq)=>format!("v2 replay rejected a seq {seq}: {e}"),
                None=>format!("v2 replay rejected: {e}"),
            })?;
            match operation {
                "verify"=>Ok(Json::object([("events",i64::try_from(log.events().len()).map_err(|_|"too many events")?.into()),("head_hash",log.events().last().ok_or("empty log")?.hash.clone().into()),("version","2".into())])),
                "graph"=>Ok(log.graph_document()),
                "active"=>Ok(log.active_document()),
                "memory"=>Ok(log.memory_document()),
                "self"=>Ok(log.self_document()),
                _=>Err("unknown operation".into()),
            }
        },
        _=>Err("unsupported version".into()),
    }
}

fn main() {
    match run() {
        Ok(result)=>println!("{}",result.canonical()),
        Err(error)=>{eprintln!("{error}");std::process::exit(1);}
    }
}
