//! Separate versioned protocol. v1 Log/EventType remain unchanged.
use std::collections::{BTreeMap, BTreeSet};
use sha2::{Digest, Sha256};
use super::Json;
use crate::{Log, prng::Pcg64};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error { Schema, Chain, Phase, Config, Reference, WindowState, IncompleteBatch, Mismatch, NoModel, DigestMismatch, Overflow }

impl std::fmt::Display for Error {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        write!(f,"AUX v2: {}", match self {
            Error::Schema=>"struttura o valore non valido",
            Error::Chain=>"catena hash o sequenza non valida",
            Error::Phase=>"operazione fuori dalla fase consentita",
            Error::Config=>"configurazione L2/L3 assente o non valida",
            Error::Reference=>"riferimento a evento, nodo o arco inesistente",
            Error::WindowState=>"finestra di selezione L2 in stato non valido",
            Error::IncompleteBatch=>"batch incompleto",
            Error::Mismatch=>"contenuto non corrispondente al replay",
            Error::NoModel=>"modello sociale assente",
            Error::DigestMismatch=>"digest fixture non corrispondente",
            Error::Overflow=>"valore fuori intervallo",
        })
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub seq: i64,
    pub tick: i64,
    pub kind: String,
    pub payload: Json,
    pub prev_hash: String,
    pub hash: String,
}
impl Record {
    fn seal(seq: i64, tick: i64, kind: &str, payload: Json, prev: &str) -> Self {
        let hash = digest(format!("{seq}\x1f{tick}\x1f{kind}\x1f{}\x1f{prev}", payload.canonical()).as_bytes());
        Self { seq, tick, kind: kind.into(), payload, prev_hash: prev.into(), hash }
    }
    pub fn canonical(&self) -> String {
        format!("{{\"seq\":{},\"tick\":{},\"type\":{},\"payload\":{},\"prev_hash\":{},\"hash\":{}}}", self.seq, self.tick, Json::from(self.kind.clone()).canonical(), self.payload.canonical(), Json::from(self.prev_hash.clone()).canonical(), Json::from(self.hash.clone()).canonical())
    }
}
pub(super) fn digest(b: &[u8]) -> String { Sha256::digest(b).iter().map(|b| format!("{b:02x}")).collect() }
fn num(j: &Json) -> Result<i64, Error> { let n = j.int()?; if n < 0 { Err(Error::Schema) } else { Ok(n) } }
fn ints(a: &[i64]) -> Json { Json::Array(a.iter().map(|n| Json::Int(*n)).collect()) }
fn list(j: &Json) -> Result<Vec<i64>, Error> {
    let a: Vec<i64> = j.array()?.iter().map(num).collect::<Result<_,_>>()?;
    if a.windows(2).any(|w| w[0] >= w[1]) { return Err(Error::Schema); } Ok(a)
}
fn empty_provenance(refs: &[i64]) -> Json { Json::object([("members", Json::Array(vec![])), ("refs", ints(refs))]) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node { pub id: String, pub kind: String, pub key: String, pub origin_seq: i64 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MemoryStatus { Circuit, Long, Archived { at: i64 } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialModel { pub counts: BTreeMap<i64,u64>, pub last: i64, pub sources: Vec<i64> }

/// Caller-supplied preregistered binding. Digest authenticates bytes, not truth.
pub struct Fixture<'a> { pub id: &'a str, pub sha256: &'a str, pub bytes: &'a str, pub anchor: i64 }

#[derive(Clone, Debug)]
struct Window { select: i64, ups: Vec<String>, downs: Vec<String>, sources: Vec<i64> }

#[derive(Clone, Debug)]
pub struct V2Log {
    events: Vec<Record>,
    rng: Pcg64,
    tick: i64,
    nodes: BTreeMap<String,Node>,
    edges: BTreeSet<(String,String,String)>,
    edge_origins: BTreeMap<(String,String,String),i64>,
    activations: BTreeMap<String,BTreeSet<i64>>,
    active: BTreeSet<String>,
    memory: BTreeMap<String,MemoryStatus>,
    l2: Option<(usize,i64)>,
    l3: Option<(u64,i64)>,
    window: Option<Window>,
    statements: Vec<Json>,
    social: BTreeMap<String,SocialModel>,
    pending: BTreeMap<String,(i64,i64)>,
    fixtures: BTreeMap<String,(String,i64)>,
}

impl V2Log {
    pub fn genesis(seed: u64) -> Result<Self, Error> {
        let old = Log::genesis(seed).map_err(|_| Error::Schema)?;
        let g = &old.events()[0];
        let payload = Json::parse(&g.payload.canonical())?;
        let event = Record::seal(0,0,"STATE",payload,&g.prev_hash);
        Ok(Self { events: vec![event], rng: Pcg64::new(seed), tick: 0,
            nodes: BTreeMap::new(), edges: BTreeSet::new(), edge_origins: BTreeMap::new(), activations: BTreeMap::new(), active: BTreeSet::new(), memory: BTreeMap::new(),
            l2: None, l3: None, window: None, statements: vec![], social: BTreeMap::new(), pending: BTreeMap::new(), fixtures: BTreeMap::new() })
    }
    pub fn events(&self) -> &[Record] { &self.events }
    pub fn nodes(&self) -> &BTreeMap<String,Node> { &self.nodes }
    pub fn edges(&self) -> &BTreeSet<(String,String,String)> { &self.edges }
    pub fn active(&self) -> &BTreeSet<String> { &self.active }
    pub fn memory(&self) -> &BTreeMap<String,MemoryStatus> { &self.memory }
    pub fn social(&self) -> &BTreeMap<String,SocialModel> { &self.social }
    pub fn window_is_open(&self) -> bool { self.window.is_some() }
    fn head(&self) -> &str { &self.events.last().expect("genesis always exists").hash }
    fn push(&mut self, kind: &str, tick: i64, payload: Json) -> Result<i64, Error> {
        let seq = i64::try_from(self.events.len()).map_err(|_| Error::Overflow)?;
        self.events.push(Record::seal(seq,tick,kind,payload,self.head())); Ok(seq)
    }
    fn aux(&mut self, level: &str, kind: &str, data: Json, sources: &[i64], provenance: Json) -> Result<i64, Error> {
        self.push("AUX",self.tick,Json::object([
            ("data",Json::from(data.canonical())),("kind",Json::from(kind)),("level",Json::from(level)),
            ("protocol_version",Json::from("2")),("provenance",Json::from(provenance.canonical())),("sources",Json::from(ints(sources).canonical())),
        ]))
    }
    /// Public mutators use clone-and-commit: any returned error leaves the log unchanged.
    fn transaction<T>(&mut self, f: impl FnOnce(&mut Self)->Result<T,Error>) -> Result<T,Error> {
        let mut next = self.clone(); let result = f(&mut next)?; *self = next; Ok(result)
    }
    fn outside(&self) -> Result<(),Error> { if self.window.is_some() { Err(Error::WindowState) } else { Ok(()) } }
    pub fn configure(&mut self, k: i64, tau: i64, threshold: i64, half_life: i64) -> Result<(),Error> {
        self.transaction(|s| {
            s.outside()?;
            if k<0 || tau<1 || threshold<0 || half_life<1 { return Err(Error::Config); }
            s.aux("L2","config",Json::object([("k",k.into()),("tau",tau.into())]),&[],empty_provenance(&[]))?;
            s.aux("L3","config",Json::object([("H",half_life.into()),("T",threshold.into())]),&[],empty_provenance(&[]))?;
            s.l2=Some((usize::try_from(k).map_err(|_| Error::Overflow)?,tau)); s.l3=Some((threshold as u64,half_life)); Ok(())
        })
    }
    pub fn cycle(&mut self) -> Result<i64,Error> {
        self.transaction(|s| {
            s.outside()?; let tick=s.tick.checked_add(1).ok_or(Error::Overflow)?;
            let draw=s.rng.next_u64();
            let at=s.push("SENSE",tick,Json::object([("stimolo",draw.to_string().into())]))?;
            s.push("ACTION",tick,Json::object([("mossa",((draw%3) as i64).into())]))?;
            s.push("STATE",tick,Json::object([("action",tick.into()),("sense",tick.into()),("state",s.rng.state_decimal().into()),("tick",tick.into())]))?;
            s.tick=tick; Ok(at)
        })
    }
    fn source(&self, seq: i64) -> Result<&Record,Error> {
        let event=self.events.get(usize::try_from(seq).map_err(|_| Error::Reference)?).ok_or(Error::Reference)?;
        if event.kind!="SENSE" || event.tick!=self.tick { return Err(Error::Reference); } Ok(event)
    }
    pub fn node_id(kind: &str,key: &str) -> String { digest(Json::object([("key",key.into()),("kind",kind.into())]).canonical().as_bytes()) }

    /// Preregisterable synthetic interpretation used by the integrated fixture.
    /// Labels are protocol conventions, not facts inferred from a random number.
    pub fn extract_synthetic(&mut self, source:i64)->Result<i64,Error> {
        let tick=self.source(source)?.tick;
        match tick {
            1=>self.extract(source,&[("elemento","alfa"),("elemento","beta"),("elemento","gamma"),("elemento","delta")],&[("elemento","alfa","elemento","beta","conosce")]),
            2|254=>self.extract(source,&[("elemento","alfa")],&[]),
            253=>self.extract(source,&[("elemento","beta")],&[]),
            _=>Err(Error::Reference),
        }
    }

    /// One source per extraction. ID arrays in v2 select disambiguate shared origins.
    /// L2 select is deferred until close_selection_window(), allowing only L2 AUX inside.
    pub fn extract(&mut self, source: i64, nodes: &[(&str,&str)], edges: &[(&str,&str,&str,&str,&str)]) -> Result<i64,Error> {
        self.transaction(|s| {
            s.outside()?; s.source(source)?;
            let mut declared=Vec::new(); let mut mentioned=BTreeSet::new();
            let mut node_data=Vec::new(); let mut edge_data=Vec::new();
            for &(kind,key) in nodes {
                let id=Self::node_id(kind,key); mentioned.insert(id.clone());
                if !s.nodes.contains_key(&id) { declared.push((kind.to_owned(),key.to_owned())); }
                s.nodes.entry(id.clone()).or_insert(Node{id:id.clone(),kind:kind.into(),key:key.into(),origin_seq:source});
                s.memory.entry(id.clone()).or_insert(MemoryStatus::Circuit);
                node_data.push(Json::object([("key",key.into()),("kind",kind.into())]));
            }
            let mut new_edges=Vec::new();
            for &(sk,sv,dk,dv,rel) in edges {
                if rel.is_empty() || !rel.bytes().all(|b| b.is_ascii_lowercase()||b.is_ascii_digit()||b==b'_') { return Err(Error::Schema); }
                let a=Self::node_id(sk,sv); let b=Self::node_id(dk,dv);
                if !s.nodes.contains_key(&a)||!s.nodes.contains_key(&b) { return Err(Error::Reference); }
                mentioned.insert(a.clone()); mentioned.insert(b.clone());
                let edge=(a,b,rel.to_owned());
                if s.edges.insert(edge.clone()) {
                    s.edge_origins.insert(edge,source);
                    new_edges.push((sk.to_owned(),sv.to_owned(),dk.to_owned(),dv.to_owned(),rel.to_owned()));
                }
                edge_data.push(Json::object([("dst_key",dv.into()),("dst_kind",dk.into()),("rel",rel.into()),("src_key",sv.into()),("src_kind",sk.into())]));
            }
            let members=Json::Array((0..node_data.len()+edge_data.len()).map(|i| Ok(Json::object([("index",i64::try_from(i).map_err(|_| Error::Overflow)?.into()),("sources",ints(&[source]))]))).collect::<Result<Vec<_>,Error>>()?);
            if node_data.is_empty()&&edge_data.is_empty() { return Err(Error::Schema); }
            let at=s.aux("L1","extract",Json::object([("edges",Json::Array(edge_data)),("nodes",Json::Array(node_data))]),&[source],Json::object([("members",members),("refs",ints(&[]))]))?;
            for id in mentioned { s.activations.entry(id).or_default().insert(source); }
            declared.sort(); declared.dedup();
            new_edges.sort_by_key(|(sk,sv,dk,dv,rel)|(rel.clone(),Self::node_id(sk,sv),Self::node_id(dk,dv)));
            let mut texts:Vec<String>=declared.iter().map(|(k,v)|format!("a seq {at} ho dichiarato l'elemento {k} \"{v}\"")).collect();
            for (sk,sv,dk,dv,rel) in new_edges { texts.push(format!("a seq {at} ho dichiarato l'arco {rel} da {sk} \"{sv}\" a {dk} \"{dv}\"")); }
            if !texts.is_empty() { s.narrate(at,&[at],texts.join("\n"),&[source])?; }
            Ok(at)
        })
    }
    fn score(&self,id:&str,at:i64,tau:i64)->u64 {
        self.activations.get(id).into_iter().flatten().filter(|a| **a<=at).fold(0u64,|sum,a|sum.saturating_add((1u64<<24)>>(((at-*a)/tau).min(24) as u32)))
    }
    /// Open, emit a recomputed select, then atomically close with mandatory summary.
    /// The entire operation, including subsequent L4 revisions, is transactional.
    pub fn close_selection_window(&mut self, extraction: i64) -> Result<(),Error> {
        self.transaction(|s| {s.open_selection_window(extraction)?;s.close_window()})
    }
    /// Start a window with exactly one recomputed select from a preceding extraction.
    /// No non-L2 operation is permitted until explicit close_window().
    pub fn open_selection_window(&mut self, extraction: i64) -> Result<(),Error> {
        self.transaction(|s| {
            s.outside()?; let (k,tau)=s.l2.ok_or(Error::Config)?;
            s.latest_extraction(extraction)?;
            if s.decision_already_recorded("L2", extraction)? { return Err(Error::Reference); }
            let e=s.events.get(usize::try_from(extraction).map_err(|_| Error::Reference)?).ok_or(Error::Reference)?.clone();
            if e.tick!=s.tick || e.kind!="AUX" || e.payload.get("level")?.string()?!="L1" { return Err(Error::Reference); }
            let sources=list(&Json::parse(e.payload.get("sources")?.string()?)?)?;
            let source=*sources.first().ok_or(Error::Reference)?;
            let mut ranked:BTreeSet<String>=s.active.clone();
            for (id,a) in &s.activations { if a.contains(&source) { ranked.insert(id.clone()); } }
            let mut ranked:Vec<_>=ranked.into_iter().collect();
            ranked.sort_by(|a,b|s.score(b,extraction,tau).cmp(&s.score(a,extraction,tau)).then_with(||s.nodes[a].origin_seq.cmp(&s.nodes[b].origin_seq)).then_with(||a.cmp(b)));
            let next:BTreeSet<_>=ranked.into_iter().take(k).collect();
            let ups:Vec<_>=next.difference(&s.active).cloned().collect();
            let downs:Vec<_>=s.active.difference(&next).cloned().collect();
            s.aux("L2","barrier",Json::object([("phase","open".into())]),&[],empty_provenance(&[]))?;
            let select=s.aux("L2","select",Json::object([("at",extraction.into()),("downs",strings(&downs)),("ups",strings(&ups))]),&sources,empty_provenance(&[extraction]))?;
            s.active=next;
            s.window=Some(Window {select,ups,downs,sources});
            Ok(())
        })
    }
    /// Atomic close/summary pair followed by L4 revisions after the window closes.
    pub fn close_window(&mut self) -> Result<(),Error> {
        self.transaction(|s| {
            let Window {select,ups,downs,sources}=s.window.clone().ok_or(Error::WindowState)?;
            s.aux("L2","barrier",Json::object([("phase","close".into())]),&[],empty_provenance(&[]))?;
            let expulsions:Vec<_>=downs.iter().map(|id|s.nodes[id].origin_seq).collect::<BTreeSet<_>>().into_iter().collect();
            let summary=s.aux("L2","summary",Json::object([("expulsions",ints(&expulsions)),("text",downs.join("|").into())]),&sources,empty_provenance(&[select]))?;
            s.window=None;
            let mut cites=vec![select]; cites.extend(ups.iter().chain(downs.iter()).map(|id|s.nodes[id].origin_seq)); cites.sort(); cites.dedup();
            let up_orig=origins(&s.nodes,&ups); let down_orig=origins(&s.nodes,&downs);
            s.narrate(select,&cites,format!("a seq {select} ho selezionato: entrate [{}], uscite [{}]",csv(&up_orig),csv(&down_orig)),&sources)?;
            let mut cites=expulsions.clone(); cites.push(summary); cites.sort(); cites.dedup();
            s.narrate(summary,&cites,format!("a seq {summary} ho consolidato: espulsi [{}]",csv(&expulsions)),&sources)?;
            Ok(())
        })
    }
    pub fn evaluate_memory(&mut self, extraction:i64) -> Result<(),Error> {
        self.transaction(|s| {
            s.outside()?; let (threshold,h)=s.l3.ok_or(Error::Config)?;
            s.latest_extraction(extraction)?;
            if s.decision_already_recorded("L3", extraction)? { return Err(Error::Reference); }
            let e=s.events.get(usize::try_from(extraction).map_err(|_| Error::Reference)?).ok_or(Error::Reference)?;
            if e.tick!=s.tick || e.kind!="AUX" || e.payload.get("level")?.string()?!="L1" { return Err(Error::Reference); }
            let sources=list(&Json::parse(e.payload.get("sources")?.string()?)?)?;
            let mut changes=Vec::new();
            for (id,activations) in &s.activations {
                let score=activations.iter().fold(0u64,|sum,seq| {
                    let age=(s.tick-s.events[*seq as usize].tick)/h;
                    sum.saturating_add(if age>=25 {0} else {(1u64<<24)>>(age as u32)})
                });
                let status=s.memory.get(id).ok_or(Error::Reference)?;
                let kind=match status {
                    MemoryStatus::Archived{at} if sources.iter().any(|a|a>at && activations.contains(a))=>Some("recall"),
                    MemoryStatus::Archived{..}=>None,
                    _ if score==0=>Some("archive"),
                    MemoryStatus::Circuit if activations.len() as u64>threshold=>Some("promote"),
                    _=>None,
                };
                if let Some(kind)=kind { changes.push((kind.to_owned(),id.clone())); }
            }
            changes.sort_by(|(ka,a),(kb,b)| {
                let priority=|k:&str|match k{"archive"=>0,"recall"=>1,_=>2};
                priority(ka).cmp(&priority(kb)).then_with(|| if ka=="archive" {s.nodes[b].origin_seq.cmp(&s.nodes[a].origin_seq).then_with(||b.cmp(a))} else {s.nodes[a].origin_seq.cmp(&s.nodes[b].origin_seq).then_with(||a.cmp(b))})
            });
            for (kind,id) in changes {
                let origin=s.nodes[&id].origin_seq;
                let at=s.aux("L3",&kind,Json::object([("at",extraction.into()),("id",id.clone().into()),("origin_seq",origin.into())]),&sources,empty_provenance(&[extraction]))?;
                s.memory.insert(id.clone(),match kind.as_str(){"archive"=>MemoryStatus::Archived{at},"recall"=>MemoryStatus::Circuit,_=>MemoryStatus::Long});
                let text=match kind.as_str(){"archive"=>format!("a seq {at} ho archiviato l'identità {id}"),"recall"=>format!("a seq {at} ho richiamato l'identità {id} (origin_seq {origin})"),_=>format!("a seq {at} ho promosso l'identità {id}")};
                let mut cites=vec![at]; if kind=="recall" {cites.push(origin);cites.sort();}
                s.narrate(at,&cites,text,&sources)?;
            }
            Ok(())
        })
    }
    fn narrate(&mut self,at:i64,cites:&[i64],text:String,sources:&[i64])->Result<(),Error> {
        self.outside()?;
        let data=Json::object([("at",at.into()),("sources",ints(cites)),("text",text.into())]);
        self.statements.push(data.clone());
        let narrate=self.aux("L4","narrate",data,sources,empty_provenance(&[at]))?;
        let hash=digest(self.self_preimage().canonical().as_bytes());
        self.aux("L4","snapshot",Json::object([("at",at.into()),("self_hash",hash.into())]),sources,empty_provenance(&[narrate]))?;
        Ok(())
    }
    fn self_preimage(&self)->Json { Json::object([("head_hash",self.head().into()),("schema_version","2".into()),("statements",Json::Array(self.statements.clone()))]) }
    pub fn self_document(&self)->Json {
        let mut doc=self.self_preimage(); let hash=digest(doc.canonical().as_bytes());
        if let Json::Object(ref mut o)=doc {o.insert("self_hash".into(),hash.into());} doc
    }
    fn decision_already_recorded(&self, level:&str, extraction:i64)->Result<bool,Error> {
        for event in &self.events {
            if event.kind=="AUX" && event.payload.get("level")?.string()?==level {
                let kind=event.payload.get("kind")?.string()?;
                if matches!(kind,"select"|"promote"|"archive"|"recall") {
                    let data=Json::parse(event.payload.get("data")?.string()?)?;
                    if num(data.get("at")?)? >= extraction {return Ok(true);}
                }
            }
        }
        Ok(false)
    }
    fn latest_extraction(&self, extraction:i64)->Result<(),Error> {
        for event in self.events.iter().rev() {
            if event.kind=="AUX" && event.payload.get("level")?.string()?=="L1" {
                return if event.seq==extraction {Ok(())} else {Err(Error::Reference)};
            }
        }
        Err(Error::Reference)
    }
    /// Canonical L1 projection. Hash includes the current chain head.
    pub fn graph_document(&self)->Json {
        let nodes=self.nodes.values().map(|n|Json::object([
            ("id",n.id.clone().into()),("key",n.key.clone().into()),("kind",n.kind.clone().into()),("origin_seq",n.origin_seq.into()),
        ])).collect();
        let edges=self.edge_origins.iter().map(|((a,b,r),origin)|Json::object([
            ("dst_id",b.clone().into()),("origin_seq",(*origin).into()),("rel",r.clone().into()),("src_id",a.clone().into()),
        ])).collect();
        hashed_document(Json::object([("edges",Json::Array(edges)),("head_hash",self.head().into()),("nodes",Json::Array(nodes)),("schema_version","2".into())]),"view_hash")
    }
    pub fn memory_document(&self)->Json {
        let entries=self.memory.iter().map(|(id,status)| {
            let mut fields=BTreeMap::from([
                ("id".into(),id.clone().into()),("origin_seq".into(),self.nodes[id].origin_seq.into()),
                ("status".into(),Json::from(match status {MemoryStatus::Circuit=>"circuito",MemoryStatus::Long=>"memoria_lunga",MemoryStatus::Archived{..}=>"archiviata"})),
            ]);
            if let MemoryStatus::Archived{at}=status {fields.insert("archived_at_seq".into(),(*at).into());}
            Json::Object(fields)
        }).collect();
        hashed_document(Json::object([("entries",Json::Array(entries)),("head_hash",self.head().into()),("schema_version","2".into())]),"archive_hash")
    }
    pub fn active_document(&self)->Json {
        Json::object([("head_hash",self.head().into()),("ids",strings(&self.active.iter().cloned().collect::<Vec<_>>())),("schema_version","2".into())])
    }
    /// Pure second-order identity composition, as in historical L5.8.
    /// Order-1 observation records do not establish distinct private beliefs.
    pub fn project_prediction(&self, agent:&str, target:&str)->Result<Json,Error> {
        if agent==target || !self.social.contains_key(agent) {return Err(Error::NoModel);}
        self.prediction_document(target)
    }
    pub fn prediction_document(&self,id:&str)->Result<Json,Error> {
        let m=self.social.get(id).ok_or(Error::NoModel)?;
        let class=m.counts.iter().max_by(|(a,na),(b,nb)|na.cmp(nb).then_with(||b.cmp(a))).map(|(a,_)|*a).ok_or(Error::NoModel)?;
        Ok(Json::object([("id",id.into()),("previsto",class.into()),("sources",ints(&m.sources))]))
    }
    pub fn to_jsonl(&self)->String { self.events.iter().map(|e|format!("{}\n",e.canonical())).collect() }

    pub fn observe(&mut self, fixture:&Fixture<'_>)->Result<i64,Error> {
        self.transaction(|s| {
            s.outside()?;
            if digest(fixture.bytes.as_bytes())!=fixture.sha256 {return Err(Error::DigestMismatch);}
            if fixture.id.is_empty() {return Err(Error::Schema);}
            let binding=(fixture.sha256.to_owned(),fixture.anchor);
            if s.fixtures.get(fixture.id).is_some_and(|previous|previous!=&binding) {return Err(Error::Mismatch);}
            let data=Json::parse(fixture.bytes)?; data.keys(&["observations"])?;
            let anchor=s.events.iter().rev().find(|e|e.kind=="STATE").ok_or(Error::Reference)?.seq;
            if anchor!=fixture.anchor {return Err(Error::Reference);}
            let mut observations=Vec::new();
            for member in data.get("observations")?.array()? {
                member.keys(&["agente","soggetto","ordine_tom","proposizione","confidenza","sources"])?;
                let agent=member.get("agente")?.string()?;
                if agent.is_empty()||member.get("soggetto")?.string()?.is_empty() {return Err(Error::Schema);}
                let order=num(member.get("ordine_tom")?)?;
                let confidence=num(member.get("confidenza")?)?;
                let proposition=member.get("proposizione")?.string()?;
                let class:i64=proposition.parse().map_err(|_|Error::Schema)?;
                if order>1||confidence>1000||class<0||class.to_string()!=proposition||list(member.get("sources")?)?!=vec![anchor] {return Err(Error::Schema);}
                if order==0 {observations.push((digest(Json::object([("altro",agent.into())]).canonical().as_bytes()),class));}
            }
            let at=s.aux("L5","observation",data,&[anchor],Json::object([
                ("external",Json::object([("fixture_id",fixture.id.into()),("sha256",fixture.sha256.into())])),
                ("members",Json::Array(vec![])),("refs",ints(&[]))]))?;
            s.fixtures.insert(fixture.id.into(),binding);
            // Each pending window is closed by the first matching observation.
            // Additional members in the same AUX increment counts but cannot reopen it.
            let mut updates=Vec::new();
            for (id,class) in observations {
                if let Some((predict,expected))=s.pending.remove(&id) {
                    if expected!=class {updates.push((id.clone(),predict,expected,class));}
                }
                let model=s.social.entry(id).or_insert(SocialModel{counts:BTreeMap::new(),last:class,sources:vec![]});
                let count=model.counts.entry(class).or_default(); *count=count.saturating_add(1);
                model.last=class; if model.sources.last()!=Some(&at) {model.sources.push(at);}
            }
            for (id,predict,expected,class) in updates {
                s.aux("L5","model_update",Json::object([("at",at.into()),("id",id.into()),("osservato",class.into()),("previsto",expected.into()),("sources",ints(&[predict,at]))]),&[anchor],empty_provenance(&[predict,at]))?;
            }
            Ok(at)
        })
    }
    pub fn predict(&mut self, id:&str)->Result<i64,Error> {
        self.transaction(|s| {
            s.outside()?; let model=s.social.get(id).ok_or(Error::NoModel)?;
            let class=model.counts.iter().max_by(|(a,na),(b,nb)|na.cmp(nb).then_with(||b.cmp(a))).map(|(a,_)|*a).ok_or(Error::NoModel)?;
            let sources=model.sources.clone(); let at=*sources.last().ok_or(Error::NoModel)?;
            let anchors:BTreeSet<i64>=sources.iter().map(|seq| {
                let text=s.events[*seq as usize].payload.get("sources")?.string()?;
                let a=list(&Json::parse(text)?)?; a.first().copied().ok_or(Error::Reference)
            }).collect::<Result<_,Error>>()?;
            let seq=s.aux("L5","predict",Json::object([("at",at.into()),("id",id.into()),("previsto",class.into()),("sources",ints(&sources))]),&anchors.into_iter().collect::<Vec<_>>(),empty_provenance(&sources))?;
            s.pending.insert(id.into(),(seq,class)); Ok(class)
        })
    }

    /// Strict structural and semantic replay by regenerating each operation from
    /// its recorded input. Fixture bytes must be supplied for every observation.
    /// It validates this module's closed profile, not arbitrary historical logs.
    pub fn from_jsonl(text:&str, version:&str, fixtures:&[Fixture<'_>])->Result<Self,Error> {
        if version!="2"||!text.ends_with('\n')||text.contains('\r') {return Err(Error::Schema);}
        let mut records=Vec::new();
        for line in text.strip_suffix('\n').ok_or(Error::Schema)?.split('\n') {
            let j=Json::decode(line)?; j.keys(&["seq","tick","type","payload","prev_hash","hash"])?;
            let record=Record {seq:num(j.get("seq")?)?,tick:num(j.get("tick")?)?,kind:j.get("type")?.string()?.into(),payload:j.get("payload")?.clone(),prev_hash:j.get("prev_hash")?.string()?.into(),hash:j.get("hash")?.string()?.into()};
            if record.canonical()!=line {return Err(Error::Schema);}
            let prev=records.last().map_or("0000000000000000000000000000000000000000000000000000000000000000",|r:&Record|r.hash.as_str());
            let expected=Record::seal(i64::try_from(records.len()).map_err(|_|Error::Overflow)?,record.tick,&record.kind,record.payload.clone(),prev);
            if record!=expected {return Err(Error::Chain);}
            records.push(record);
        }
        let first=records.first().ok_or(Error::Schema)?;
        let seed=u64::try_from(num(first.payload.get("seed")?)?).map_err(|_|Error::Schema)?;
        let mut result=Self::genesis(seed)?;
        if result.events[0]!=*first {return Err(Error::Mismatch);}
        // Duplicate fixture identifiers are forbidden even if their bytes match.
        let mut bindings=BTreeMap::new();
        for f in fixtures {if bindings.insert(f.id,f).is_some() {return Err(Error::Schema);}}
        while result.events.len()<records.len() {
            let start=result.events.len(); let e=&records[start];
            if e.kind=="SENSE" {result.cycle()?;} else if e.kind=="AUX" {
                e.payload.keys(&["data","kind","level","protocol_version","provenance","sources"])?;
                if e.payload.get("protocol_version")?.string()?!="2" {return Err(Error::Schema);}
                let level=e.payload.get("level")?.string()?; let kind=e.payload.get("kind")?.string()?;
                let data=Json::parse(e.payload.get("data")?.string()?)?;
                match (level,kind) {
                    ("L2","config")=>{
                        let next=records.get(start+1).ok_or(Error::Schema)?;
                        let l3=Json::parse(next.payload.get("data")?.string()?)?;
                        result.configure(num(data.get("k")?)?,num(data.get("tau")?)?,num(l3.get("T")?)?,num(l3.get("H")?)?)?;
                    },
                    ("L1","extract")=>{
                        let sources=list(&Json::parse(e.payload.get("sources")?.string()?)?)?;
                        if sources.len()!=1 {return Err(Error::Schema);}
                        let ns=data.get("nodes")?.array()?; let es=data.get("edges")?.array()?;
                        let nodes:Vec<_>=ns.iter().map(|n|Ok((n.get("kind")?.string()?,n.get("key")?.string()?))).collect::<Result<_,Error>>()?;
                        let edges:Vec<_>=es.iter().map(|a|Ok((a.get("src_kind")?.string()?,a.get("src_key")?.string()?,a.get("dst_kind")?.string()?,a.get("dst_key")?.string()?,a.get("rel")?.string()?))).collect::<Result<_,Error>>()?;
                        result.extract(sources[0],&nodes,&edges)?;
                    },
                    ("L2","barrier")=>{
                        match data.get("phase")?.string()? {
                            "open"=>{
                                let select=records.get(start+1).ok_or(Error::IncompleteBatch)?;
                                let select_data=Json::parse(select.payload.get("data")?.string()?)?;
                                result.open_selection_window(num(select_data.get("at")?)?)?;
                            },
                            "close"=>{result.close_window()?;},
                            _=>return Err(Error::WindowState),
                        }
                    },
                    ("L3","promote"|"archive"|"recall")=>{result.evaluate_memory(num(data.get("at")?)?)?;},
                    ("L5","predict")=>{result.predict(data.get("id")?.string()?)?;},
                    ("L5","observation")=>{
                        let p=Json::parse(e.payload.get("provenance")?.string()?)?;
                        let id=p.get("external")?.get("fixture_id")?.string()?;
                        result.observe(bindings.get(id).ok_or(Error::Reference)?)?;
                    },
                    _=>return Err(Error::Schema),
                }
            } else {return Err(Error::Phase);}
            let end=result.events.len();
            if end<=start||end>records.len()||result.events[start..end]!=records[start..end] {return Err(Error::Mismatch);}
        }
        Ok(result)
    }
    pub fn verify(&self, fixtures:&[Fixture<'_>])->Result<(),Error> {
        Self::from_jsonl(&self.to_jsonl(),"2",fixtures).map(|_|())
    }
    /// Read accessor for endpoint validation in the MCP backend: whether the
    /// replayed registry already contains a node declared as kind/key.
    pub fn has_node(&self,kind:&str,key:&str)->bool {
        self.nodes.contains_key(&Self::node_id(kind,key))
    }
}

/// Diagnostic only: first seq whose structural chain (prev_hash/hash/sequence)
/// breaks in canonical JSONL. Returns None for schema errors, stops before
/// emitting partial content, and does not alter replay semantics.
pub fn chain_break(text:&str)->Option<i64> {
    let mut previous=String::from("0000000000000000000000000000000000000000000000000000000000000000");
    let mut records=0usize;
    for line in text.split('\n') {
        if line.is_empty() {
            if records>0 {break;} else {return None;}
        }
        if line.ends_with('\r') {return None;}
        let j=Json::decode(line).ok()?;
        j.keys(&["seq","tick","type","payload","prev_hash","hash"]).ok()?;
        let record=Record {
            seq:num(j.get("seq").ok()?).ok()?,
            tick:num(j.get("tick").ok()?).ok()?,
            kind:j.get("type").ok()?.string().ok()?.into(),
            payload:j.get("payload").ok()?.clone(),
            prev_hash:j.get("prev_hash").ok()?.string().ok()?.into(),
            hash:j.get("hash").ok()?.string().ok()?.into(),
        };
        let seq=i64::try_from(records).ok()?;
        let expected=Record::seal(seq,record.tick,&record.kind,record.payload.clone(),&previous);
        if record!=expected {return Some(seq);}
        previous=record.hash;
        records+=1;
    }
    None
}
fn strings(v:&[String])->Json { Json::Array(v.iter().map(|s|s.clone().into()).collect()) }
fn origins(nodes:&BTreeMap<String,Node>,ids:&[String])->Vec<i64> { ids.iter().map(|id|nodes[id].origin_seq).collect::<BTreeSet<_>>().into_iter().collect() }
fn csv(v:&[i64])->String {v.iter().map(|n|n.to_string()).collect::<Vec<_>>().join(",")}
fn hashed_document(mut doc:Json,key:&str)->Json {
    let hash=digest(doc.canonical().as_bytes());
    if let Json::Object(ref mut fields)=doc {fields.insert(key.into(),hash.into());} doc
}
