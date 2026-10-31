//! Deterministic L2 selection replay. The log and L1 view are read-only inputs.
use std::collections::{BTreeMap,BTreeSet};
use crate::{Log,Value};
use crate::view;

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum L2Error{Config,Schema,Barrier,HeadMatch}
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Decision{Select{at:u64,tick:u64,ups:Vec<u64>,downs:Vec<u64>},Summary{expulsions:Vec<u64>,text:String}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ActiveSet{head_hash:String, log_len:usize, active:Vec<u64>}

fn strv<'a>(e:&'a crate::Event,k:&str)->Result<&'a str,L2Error>{match e.payload.get(k){Some(Value::Str(s))=>Ok(s),_=>Err(L2Error::Schema)}}
fn intv(e:&crate::Event,k:&str)->Result<i64,L2Error>{match e.payload.get(k){Some(Value::Int(n))=>Ok(*n),_=>Err(L2Error::Config)}}
fn arr_seq(s:&str)->Result<Vec<u64>,L2Error>{if s=="[]"{return Ok(vec![])} if !s.starts_with('[')||!s.ends_with(']'){return Err(L2Error::Schema)} s[1..s.len()-1].split(',').map(|x|x.parse::<u64>().map_err(|_|L2Error::Schema)).collect()}
fn allowed(e:&crate::Event,tag:&str)->Result<(),L2Error>{let canonical=e.payload.canonical();let extras:Vec<&str>=canonical.trim_matches(|c|c=='{'||c=='}').split(',').filter_map(|m|m.split_once(':').map(|x|x.0.trim_matches('"'))).filter(|k|*k!="l2"&&match tag{"config"=>!matches!(*k,"k"|"tau"),"select"=>!matches!(*k,"at"|"ups"|"downs"),"summary"=>!matches!(*k,"expulsions"|"text"),"barrier"=>*k!="phase",_=>true}).collect();if extras.is_empty(){Ok(())}else{Err(L2Error::Schema)}}
fn parse_nodes(v:&Value)->Result<Vec<(String,String)>,L2Error>{let s=match v{Value::Str(s)=>s,_=>return Err(L2Error::Schema)};if s=="[]"{return Ok(vec![])} let mut out=vec![];for x in s.trim_matches(|c|c=='['||c==']').split("},{"){let x=x.trim_matches(|c|c=='{'||c=='}');let mut k=None;let mut kind=None;for p in x.split(','){let(a,b)=p.split_once(':').ok_or(L2Error::Schema)?;let val=b.trim_matches('"').to_string();match a{"\"key\""=>k=Some(val),"\"kind\""=>kind=Some(val),_=>return Err(L2Error::Schema)}}out.push((kind.ok_or(L2Error::Schema)?,k.ok_or(L2Error::Schema)?));}Ok(out)}
fn id(kind:&str,key:&str)->String{use sha2::{Digest,Sha256};let src=format!("{{\"key\":\"{key}\",\"kind\":\"{kind}\"}}");Sha256::digest(src.as_bytes()).iter().map(|b|format!("{b:02x}")).collect()}
// L1 has already validated canonical F1. Keep JSON string tokens intact so
// escaped characters are compared using the same canonical encoder as L1.
fn edge_mentions(v:&Value,view:&view::View)->Result<Vec<(String,u64)>,L2Error>{
 let s=match v{Value::Str(s)=>s,_=>return Err(L2Error::Schema)};
 let bytes=s.as_bytes();let mut tokens=Vec::new();let mut pos=0;
 while pos<bytes.len(){
  if bytes[pos]!=b'"'{pos+=1;continue}
  let start=pos;pos+=1;
  while pos<bytes.len() && bytes[pos]!=b'"'{if bytes[pos]==b'\\'{pos+=1;}pos+=1;}
  if pos>=bytes.len(){return Err(L2Error::Schema)}
  pos+=1;tokens.push(&s[start..pos]);
 }
 if tokens.len()%10!=0{return Err(L2Error::Schema)}
 let mut out=Vec::new();
 for edge in tokens.chunks_exact(10){
  for (key,kind) in [(edge[1],edge[3]),(edge[7],edge[9])]{
   let n=view.nodes.iter().find(|n|{
    let mut k=String::new();let mut t=String::new();
    crate::payload::write_escaped(&mut k,&n.key);crate::payload::write_escaped(&mut t,&n.kind);
    k==key && t==kind
   }).ok_or(L2Error::Schema)?;
   out.push((n.id.clone(),n.origin_seq));
  }
 }
 Ok(out)
}
fn replay(log:&Log,emit:bool,cutoff:u64)->Result<(ActiveSet,Vec<Decision>),L2Error>{let view=view::build(log).map_err(|_|L2Error::Schema)?;if view.head_hash!=log.events().last().ok_or(L2Error::HeadMatch)?.hash{return Err(L2Error::HeadMatch)}
  let mut config:Option<(u64,u64)>=None;let mut activations:BTreeMap<String,Vec<u64>>=BTreeMap::new();let mut origin_map:BTreeMap<String,u64>=BTreeMap::new();let mut active=BTreeSet::new();let mut decisions=vec![];let mut barrier=false;let mut expulsions=BTreeSet::new();let mut expelled_ids=BTreeSet::new();let mut window_has_summary=false;
 for e in log.events(){let lv=e.payload.get("l2");if let Some(v)=lv{let tag=match v{Value::Str(s)=>s.as_str(),_=>return Err(L2Error::Schema)};if !matches!(tag,"config"|"select"|"summary"|"barrier"){return Err(L2Error::Schema)}allowed(e,tag)?;match tag{"config"=>{let k=intv(e,"k").map_err(|_|L2Error::Config)?;let tau=intv(e,"tau").map_err(|_|L2Error::Config)?;if k<0||tau<1{return Err(L2Error::Config)}config=Some((k as u64,tau as u64));},"select"=>{
   let downs=arr_seq(strv(e,"downs")?)?;
   if barrier {
    let at=match e.payload.get("at"){Some(Value::Int(n))=>u64::try_from(*n).map_err(|_|L2Error::Schema)?,_=>return Err(L2Error::Schema)};
     let tau=config.ok_or(L2Error::Config)?.1;
    let score=|i:&String|->u64{activations.get(i).into_iter().flatten().map(|seq|(1u64<<24)>>((at.saturating_sub(*seq)/tau).min(24))).sum()};
    let mut ranked:Vec<String>=active.iter().cloned().collect();
    ranked.sort_by(|a,b|score(a).cmp(&score(b)).then_with(||origin_map[b].cmp(&origin_map[a])).then_with(||b.cmp(a)));
     for origin in downs {if let Some(pos)=ranked.iter().position(|i|origin_map.get(i)==Some(&origin)){let i=ranked.remove(pos);expulsions.insert(origin);expelled_ids.insert(i.clone());active.remove(&i);}}
   }
   let ups=arr_seq(strv(e,"ups")?)?;
   for origin in ups {for n in view.nodes.iter().filter(|n|n.origin_seq==origin){active.insert(n.id.clone());origin_map.insert(n.id.clone(),origin);}}
  },"summary"=>{if barrier {window_has_summary=true;}},"barrier"=>{let phase=e.payload.get("phase");if let Some(v)=phase{if v!=&Value::Str("close".into()){return Err(L2Error::Barrier)}if !barrier{return Err(L2Error::Barrier)}barrier=false;if !window_has_summary && !expulsions.is_empty() && e.seq > cutoff {let mut ids_sorted:Vec<String>=expelled_ids.iter().cloned().collect();ids_sorted.sort();decisions.push(Decision::Summary{expulsions:expulsions.iter().copied().collect(),text:ids_sorted.join("|")});}expulsions.clear();expelled_ids.clear();window_has_summary=false;}else{if barrier{return Err(L2Error::Barrier)}barrier=true;expulsions.clear();expelled_ids.clear();window_has_summary=false;}}_=>unreachable!()};continue}

  if barrier {
   barrier=false;
   if !window_has_summary && !expulsions.is_empty() && e.seq>cutoff {let mut ids_sorted:Vec<String>=expelled_ids.iter().cloned().collect();ids_sorted.sort();decisions.push(Decision::Summary{expulsions:expulsions.iter().copied().collect(),text:ids_sorted.join("|")});}
   expulsions.clear();expelled_ids.clear();window_has_summary=false;
  }
  let mut mentioned=vec![];if let Some(v)=e.payload.get("nodes"){for(kind,key)in parse_nodes(v)?{let i=id(&kind,&key);let origin=view.nodes.iter().find(|n|n.id==i).map(|n|n.origin_seq).unwrap_or(e.seq);origin_map.insert(i.clone(),origin);mentioned.push((i,origin));}}
  if let Some(v)=e.payload.get("edges"){mentioned.extend(edge_mentions(v,&view)?);}
 mentioned.sort();mentioned.dedup_by(|a,b|a.0==b.0);for(i,_)in &mentioned{activations.entry(i.clone()).or_default().push(e.seq)}
 if !mentioned.is_empty(){let(k,tau)=config.ok_or(L2Error::Config)?;let old=active.clone();for(i,origin)in &mentioned{origin_map.insert(i.clone(),*origin);active.insert(i.clone());}let score=|id:&String|->u64{activations.get(id).into_iter().flatten().map(|s|(1u64<<24)>>(((e.seq-*s)/tau).min(24))).sum()};let mut downs=Vec::new();while active.len()>k as usize{let loser=active.iter().min_by(|a,b|score(a).cmp(&score(b)).then_with(||origin_map[*b].cmp(&origin_map[*a])).then_with(||b.cmp(a))).unwrap().clone();active.remove(&loser);let origin=origin_map[&loser];downs.push(origin);if barrier && (!emit || e.seq>cutoff){expulsions.insert(origin);expelled_ids.insert(loser);}}let ups:Vec<u64>=active.difference(&old).filter_map(|i|origin_map.get(i).copied()).collect();downs.sort_unstable();if !ups.is_empty()||!downs.is_empty(){decisions.push(Decision::Select{at:e.seq,tick:e.tick,ups,downs});}}
 }
 Ok((ActiveSet{head_hash:log.events().last().ok_or(L2Error::HeadMatch)?.hash.clone(),log_len:log.events().len(),active:active.iter().filter_map(|i|origin_map.get(i).copied()).collect()},decisions))}
pub fn build_active(log:&Log)->Result<ActiveSet,L2Error>{Ok(replay(log,false,0)?.0)}
pub fn evaluate(log:&Log,base:&ActiveSet)->Result<Vec<Decision>,L2Error>{if log.events().len()<base.log_len||base.log_len==0||log.events()[base.log_len-1].hash!=base.head_hash{return Err(L2Error::HeadMatch)}let decision_cutoff=log.events().iter().filter(|e|matches!(e.payload.get("l2"),Some(Value::Str(t)) if t=="select"||t=="summary")).map(|e|e.seq).max().unwrap_or(0);let cutoff=decision_cutoff.max(log.events()[base.log_len-1].seq);Ok(replay(log,true,cutoff)?.1.into_iter().filter(|d|match d{Decision::Select{at,..}=>*at>cutoff,Decision::Summary{..}=>true}).collect())}
pub fn encode(d:&Decision)->Vec<(String,Value)>{match d{Decision::Select{at,ups,downs,..}=>vec![("l2".into(),Value::Str("select".into())),("at".into(),Value::Int(*at as i64)),("ups".into(),Value::Str(format!("[{}]",ups.iter().map(u64::to_string).collect::<Vec<_>>().join(",")))),("downs".into(),Value::Str(format!("[{}]",downs.iter().map(u64::to_string).collect::<Vec<_>>().join(","))))],Decision::Summary{expulsions,text}=>vec![("l2".into(),Value::Str("summary".into())),("expulsions".into(),Value::Str(format!("[{}]",expulsions.iter().map(u64::to_string).collect::<Vec<_>>().join(",")))),("text".into(),Value::Str(text.clone()))]}}
