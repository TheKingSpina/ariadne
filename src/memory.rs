//! L3 — replay deterministico della memoria lunga (legge L3.1–L3.10).
use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::{view, EventType, Log, Value};
use crate::payload::write_escaped;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry { pub id: String, pub origin_seq: u64, pub status: Status, pub archived_at_seq: Option<u64> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status { Circuit, Long, Archived }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryView { pub archive_hash: String, pub config: Option<(i64,i64)>, pub entries: Vec<Entry>, pub log_len: usize, pub head_hash: String }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum L3Error { Config, Schema, HeadMatch, Recall, Transition }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision3 { Promote { at:u64,tick:u64,origin_seq:u64,id:String }, Archive { at:u64,tick:u64,origin_seq:u64,id:String } }

fn sha(bytes:&[u8])->String {
    Sha256::digest(bytes).iter().map(|b|format!("{b:02x}")).collect()
}
fn node_id(kind:&str,key:&str)->String {
    let mut s=String::from("{\"key\":"); write_escaped(&mut s,key); s.push_str(",\"kind\":"); write_escaped(&mut s,kind); s.push('}'); sha(s.as_bytes())
}
fn strv<'a>(e:&'a crate::Event,k:&str)->Result<&'a str,L3Error>{match e.payload.get(k){Some(Value::Str(s))=>Ok(s),_=>Err(L3Error::Schema)}}
fn intv(e:&crate::Event,k:&str)->Result<i64,L3Error>{match e.payload.get(k){Some(Value::Int(n))=>Ok(*n),_=>Err(L3Error::Schema)}}
fn exact(e:&crate::Event, allowed:&[&str])->Result<(),L3Error>{
    if e.payload.iter().all(|(k,_)|k=="l3"||allowed.contains(&k.as_str())) {Ok(())} else {Err(L3Error::Schema)}
}

// These strings have already passed the strict canonical parser in view::build.
fn strings_array(src:&str)->Result<Vec<Vec<String>>,L3Error>{
    if src=="[]" {return Ok(Vec::new())}
    let bytes=src.as_bytes(); let mut pos=1; let mut result=Vec::new();
    while pos+1<bytes.len() {
        if bytes[pos]!=b'{' {return Err(L3Error::Schema)} pos+=1;
        let mut fields=Vec::new();
        loop {
            if pos>=bytes.len() {return Err(L3Error::Schema)}
            if bytes[pos]==b'}' {pos+=1;break}
            if bytes[pos]==b',' {pos+=1}
            let (k,next)=read_json_string(src,pos)?; pos=next;
            if bytes.get(pos)!=Some(&b':'){return Err(L3Error::Schema)} pos+=1;
            let (v,next)=read_json_string(src,pos)?;pos=next;fields.push((k,v));
        }
        result.push(fields.into_iter().map(|(_,v)|v).collect());
        if bytes.get(pos)==Some(&b','){pos+=1}else{break}
    }
    if pos+1!=bytes.len() || bytes.get(pos)!=Some(&b']'){return Err(L3Error::Schema)}
    Ok(result)
}
fn read_json_string(s:&str,mut i:usize)->Result<(String,usize),L3Error>{
    let b=s.as_bytes();if b.get(i)!=Some(&b'"'){return Err(L3Error::Schema)} i+=1;let start=i;let mut escaped=false;
    while i<b.len(){if !escaped&&b[i]==b'"'{let token=&s[start-1..i+1];let decoded=decode_string(token)?;return Ok((decoded,i+1))}if !escaped&&b[i]==b'\\'{escaped=true}else{escaped=false}i+=1}
    Err(L3Error::Schema)
}
fn decode_string(token:&str)->Result<String,L3Error>{
    let mut out=String::new();let mut it=token[1..token.len()-1].chars();
    while let Some(c)=it.next(){if c!='\\'{out.push(c);continue}match it.next().ok_or(L3Error::Schema)?{'"'=>out.push('"'),'\\'=>out.push('\\'),'b'=>out.push('\u{8}'),'t'=>out.push('\t'),'n'=>out.push('\n'),'f'=>out.push('\u{c}'),'r'=>out.push('\r'),'u'=>{let mut hex=String::new();for _ in 0..4{hex.push(it.next().ok_or(L3Error::Schema)?)}let n=u32::from_str_radix(&hex,16).map_err(|_|L3Error::Schema)?;out.push(char::from_u32(n).ok_or(L3Error::Schema)?);},_=>return Err(L3Error::Schema)}}Ok(out)
}
fn activations(log:&Log, v:&view::View)->Result<BTreeMap<String,Vec<(u64,u64)>>,L3Error>{
    let mut out:BTreeMap<String,Vec<(u64,u64)>>=BTreeMap::new();
    let by_name:BTreeMap<(String,String),String>=v.nodes.iter().map(|n|((n.kind.clone(),n.key.clone()),n.id.clone())).collect();
    for e in log.events(){
        let mut ids=BTreeSet::new();
        if let Some(value)=e.payload.get("nodes") {let src=match value{Value::Str(s)=>s,_=>return Err(L3Error::Schema)};for fields in strings_array(src)?{if fields.len()!=2{return Err(L3Error::Schema)}let key=&fields[0];let kind=&fields[1];let id=by_name.get(&(kind.clone(),key.clone())).ok_or(L3Error::Schema)?;ids.insert(id.clone());}}
        if let Some(value)=e.payload.get("edges") {let src=match value{Value::Str(s)=>s,_=>return Err(L3Error::Schema)};for f in strings_array(src)?{if f.len()!=5{return Err(L3Error::Schema)}for (kind,key) in [(&f[4],&f[3]),(&f[1],&f[0])]{let id=by_name.get(&(kind.clone(),key.clone())).ok_or(L3Error::Schema)?;ids.insert(id.clone());}}}
        for id in ids{out.entry(id).or_default().push((e.seq,e.tick));}
    }
    Ok(out)
}
fn parse_config(e:&crate::Event)->Result<(i64,i64),L3Error>{
    exact(e,&["T","H"])?;let t=intv(e,"T").map_err(|_|L3Error::Config)?;let h=intv(e,"H").map_err(|_|L3Error::Config)?;
    if t<0||h<1 {Err(L3Error::Config)}else{Ok((t,h))}
}
fn replay(log:&Log)->Result<(Option<(i64,i64)>,BTreeMap<String,Entry>,BTreeMap<String,Vec<(u64,u64)>>),L3Error>{
    let v=view::build(log).map_err(|_|L3Error::Schema)?;
    if log.events().last().map(|e|e.hash.as_str())!=Some(v.head_hash.as_str()){return Err(L3Error::HeadMatch)}
    let acts=activations(log,&v)?;
    let mut entries:BTreeMap<String,Entry>=BTreeMap::new();
    for (id,_) in &acts {let n=v.nodes.iter().find(|n|&n.id==id).ok_or(L3Error::Schema)?;entries.insert(id.clone(),Entry{id:id.clone(),origin_seq:n.origin_seq,status:Status::Circuit,archived_at_seq:None});}
    let mut config=None;
    for e in log.events(){
        if e.r#type!=EventType::Action {if e.payload.get("l3").is_some(){return Err(L3Error::Schema)}continue}
        if e.payload.get("l2").is_some()&&e.payload.get("l3").is_some(){return Err(L3Error::Schema)}
        let tag=match e.payload.get("l3"){None=>continue,Some(Value::Str(s))=>s.as_str(),Some(_)=>return Err(L3Error::Schema)};
        match tag {
            "config"=>config=Some(parse_config(e)?),
            "promote"|"archive"|"recall"=>{
                exact(e,&["at","id","origin_seq"])?;
                let at=u64::try_from(intv(e,"at")?).map_err(|_|L3Error::Schema)?;
                if tag!="recall" && !log.events().iter().take(e.seq as usize).any(|prior|prior.seq==at&&(prior.payload.get("nodes").is_some()||prior.payload.get("edges").is_some())) {return Err(L3Error::Transition)}
                if tag=="recall" && at>=e.seq {return Err(L3Error::Recall)}
                let id=strv(e,"id")?.to_string();
                if id.len()!=64||!id.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Err(L3Error::Schema)}
                let origin=u64::try_from(intv(e,"origin_seq")?).map_err(|_|L3Error::Schema)?;
                let entry=entries.get_mut(&id).ok_or(L3Error::Transition)?;
                if entry.origin_seq!=origin{return Err(L3Error::Transition)}
                match tag {
                    "promote"=>if entry.status==Status::Circuit {entry.status=Status::Long}else{return Err(L3Error::Transition)},
                    "archive"=>if entry.status!=Status::Archived {entry.status=Status::Archived;entry.archived_at_seq=Some(e.seq)}else{return Err(L3Error::Transition)},
                    _=>{
                        if entry.status!=Status::Archived{return Err(L3Error::Transition)}
                        let archive=entry.archived_at_seq.ok_or(L3Error::Transition)?;
                        if at<=archive||!acts.get(&id).is_some_and(|xs|xs.iter().any(|(s,_)|*s==at)){return Err(L3Error::Recall)}
                        entry.status=Status::Circuit;entry.archived_at_seq=None;
                    }
                }
            },
            _=>return Err(L3Error::Schema),
        }
    }
    if config.is_none(){return Err(L3Error::Config)}
    Ok((config,entries,acts))
}
fn without_archive_hash(view:&MemoryView)->String{
    let mut s=String::from("{\"entries\":[");
    for (i,e) in view.entries.iter().enumerate(){if i>0{s.push(',')}s.push('{');let mut first=true;if let Some(seq)=e.archived_at_seq{s.push_str("\"archived_at_seq\":");s.push_str(&seq.to_string());first=false}if !first{s.push(',')}s.push_str("\"id\":");write_escaped(&mut s,&e.id);s.push_str(",\"origin_seq\":");s.push_str(&e.origin_seq.to_string());s.push_str(",\"status\":");write_escaped(&mut s,status_text(e.status));s.push('}')}
    s.push_str("],\"head_hash\":");write_escaped(&mut s,&view.head_hash);s.push_str(",\"schema_version\":\"1\"}");s
}
fn status_text(s:Status)->&'static str{match s{Status::Circuit=>"circuito",Status::Long=>"memoria_lunga",Status::Archived=>"archiviata"}}
pub fn canonical(view:&MemoryView)->String{
    let mut s=String::from("{\"archive_hash\":");write_escaped(&mut s,&view.archive_hash);s.push_str(",\"entries\":[");
    for(i,e)in view.entries.iter().enumerate(){if i>0{s.push(',')}s.push('{');let mut first=true;if let Some(seq)=e.archived_at_seq{s.push_str("\"archived_at_seq\":");s.push_str(&seq.to_string());first=false}if !first{s.push(',')}s.push_str("\"id\":");write_escaped(&mut s,&e.id);s.push_str(",\"origin_seq\":");s.push_str(&e.origin_seq.to_string());s.push_str(",\"status\":");write_escaped(&mut s,status_text(e.status));s.push('}')}
    s.push_str("],\"head_hash\":");write_escaped(&mut s,&view.head_hash);s.push_str(",\"schema_version\":\"1\"}");s
}
pub fn build_memory(log:&Log)->Result<MemoryView,L3Error>{
    let(config,entries,_)=replay(log)?;let mut v=MemoryView{archive_hash:String::new(),config,entries:entries.into_values().collect(),log_len:log.events().len(),head_hash:log.events().last().ok_or(L3Error::HeadMatch)?.hash.clone()};
    v.archive_hash=sha(without_archive_hash(&v).as_bytes());Ok(v)
}
fn score(id:&str,seq:u64,tick:u64,h:i64,acts:&BTreeMap<String,Vec<(u64,u64)>>)->u64{
    acts.get(id).into_iter().flatten().filter(|(activation_seq,_)|*activation_seq<=seq).fold(0u64,|sum,(_,at)|{let m=tick.saturating_sub(*at)/(h as u64);let c=if m>=25{0}else{(1u64<<24)>>(m as u32)};sum.saturating_add(c)})
}
pub fn evaluate(log:&Log,base:&MemoryView)->Result<Vec<Decision3>,L3Error>{
    if log.events().len()<base.log_len||base.log_len==0||log.events()[base.log_len-1].hash!=base.head_hash{return Err(L3Error::HeadMatch)}
    let(_config,mut entries,acts)=replay(log)?;
    let decision_cutoff=log.events().iter().filter(|e|matches!(e.payload.get("l3"),Some(Value::Str(x))if x=="promote"||x=="archive"||x=="recall")).map(|e|e.seq).max().unwrap_or(0).max(log.events()[base.log_len-1].seq);
    let v=view::build(log).map_err(|_|L3Error::Schema)?;let mut events: BTreeMap<u64,BTreeSet<String>>=BTreeMap::new();
    for e in log.events().iter().filter(|e|e.seq>decision_cutoff&&(e.payload.get("nodes").is_some()||e.payload.get("edges").is_some())){let mut ids=BTreeSet::new();if let Some(Value::Str(s))=e.payload.get("nodes"){for f in strings_array(s)?{if f.len()!=2{return Err(L3Error::Schema)}ids.insert(node_id(&f[1],&f[0]));}}if let Some(Value::Str(s))=e.payload.get("edges"){for f in strings_array(s)?{if f.len()!=5{return Err(L3Error::Schema)}ids.insert(node_id(&f[4],&f[3]));ids.insert(node_id(&f[1],&f[0]));}}events.insert(e.seq,ids);}
    let mut out=Vec::new();
    for e in log.events().iter().filter(|e|e.seq>decision_cutoff&&events.contains_key(&e.seq)){
        let mut config_at=None;for prior in log.events().iter().take(e.seq as usize+1){if matches!(prior.payload.get("l3"),Some(Value::Str(x))if x=="config"){config_at=Some(parse_config(prior)?);}}
        let(t,h)=config_at.ok_or(L3Error::Config)?;
        let mut archives:Vec<String>=entries.iter().filter(|(id,x)|x.status!=Status::Archived&&acts.get(*id).is_some_and(|xs|xs.iter().any(|(seq,_)|*seq<=e.seq))&&score(id,e.seq,e.tick,h,&acts)==0).map(|(id,_)|id.clone()).collect();
        archives.sort_by(|a,b|entries[b].origin_seq.cmp(&entries[a].origin_seq).then_with(||b.cmp(a)));
        let mut promotes:Vec<String>=entries.iter().filter(|(id,x)|x.status==Status::Circuit&&acts.get(*id).map_or(0,|a|a.iter().filter(|(s,_)|*s<=e.seq).count())>t as usize).map(|(id,_)|id.clone()).collect();
        promotes.sort_by(|a,b|entries[a].origin_seq.cmp(&entries[b].origin_seq).then_with(||a.cmp(b)));
        for id in archives{let en=entries.get_mut(&id).expect("identity collected from entries");out.push(Decision3::Archive{at:e.seq,tick:e.tick,origin_seq:en.origin_seq,id:id.clone()});en.status=Status::Archived;en.archived_at_seq=Some(e.seq)}
        for id in promotes{let en=entries.get_mut(&id).expect("identity collected from entries");if en.status==Status::Circuit{out.push(Decision3::Promote{at:e.seq,tick:e.tick,origin_seq:en.origin_seq,id:id.clone()});en.status=Status::Long}}
    }
    let _=v;Ok(out)
}
pub fn encode(d:&Decision3)->Vec<(String,Value)>{match d{Decision3::Promote{at,origin_seq,id,..}=>vec![("l3".into(),Value::Str("promote".into())),("at".into(),Value::Int(*at as i64)),("id".into(),Value::Str(id.clone())),("origin_seq".into(),Value::Int(*origin_seq as i64))],Decision3::Archive{at,origin_seq,id,..}=>vec![("l3".into(),Value::Str("archive".into())),("at".into(),Value::Int(*at as i64)),("id".into(),Value::Str(id.clone())),("origin_seq".into(),Value::Int(*origin_seq as i64))]}}
pub fn encode_recall(at:u64,origin_seq:u64,id:String)->Vec<(String,Value)>{vec![("l3".into(),Value::Str("recall".into())),("at".into(),Value::Int(at as i64)),("id".into(),Value::Str(id)),("origin_seq".into(),Value::Int(origin_seq as i64))]}
