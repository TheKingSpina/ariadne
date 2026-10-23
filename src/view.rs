//! Vista L1 — proiezione derivata dal log L0 (SPEC L1 v0.1 §1–§5, addendum
//! umano D1–D13, decisioni operative D14–D21 del dispatch del 23/10/2026).
//!
//! - Input: SOLO il log L0, letto in ordine di seq di `&Log`, sola lettura.
//!   Il builder assume il log già verificato da L0 (D20): nessun re-hash,
//!   nessuna rivérifica della catena — le proprietà strutturali L0 restano
//!   di L0, qui non vengono reimplementate.
//! - Estrazione deterministica dalle chiavi del payload: `nodes` e `edges`
//!   sono stringhe che contengono ciascuna un array JSON canonico interno
//!   (UTF-8, chiavi lessicografiche, senza spazi, escape N3 severo — lo
//!   stesso emitted da `payload::write_escaped`). Gli elementi sono oggetti
//!   ESATTI con stringhe `kind`/`key` (nodi) o `src_kind`/`src_key`/
//!   `dst_kind`/`dst_key`/`rel` (archi): campi mancanti/extra, tipi
//!   sbagliati o forma non canonica sono errore. Nessuna estrazione implicita:
//!   qualunque altra chiave del payload, con qualunque altro valore,
//!   contribuisce zero. Payload L0 invariato: valori top-level solo
//!   i64/stringhe, `nodes`/`edges` restano stringhe.
//! - id nodo = SHA-256 minuscolo della serializzazione canonica di
//!   `{"key":…,"kind":…}` (chiavi in ordine canonico: "key" prima di "kind").
//!   origin_seq nodo = seq minimo degli eventi che lo dichiarano.
//! - rel regex `[a-z0-9_]+`; endpoint kind+key risolti in id durante il
//!   replay; riferimento a nodo non ancora dichiarato = errore (no skip,
//!   no autocreazione; l'ordine di replay è l'ordine di seq, e nel singolo
//!   evento l'array `nodes` è elaborato per intero prima di `edges`, D19).
//! - Arco unico per `(src_id,dst_id,rel)`; origin_seq arco = seq minimo
//!   degli eventi che provano lo stesso edge (D18).
//! - Vista (D14–D17): documento JSON canonico
//!   `{"edges":[…],"head_hash":"…","nodes":[…],"schema_version":"1","view_hash":"…"}`
//!   con chiavi in ordine lessicografico; `head_hash` = hash dell'ultimo
//!   evento del log (compresa la genesi-only); `schema_version` = stringa
//!   `"1"`; `view_hash` = SHA-256 minuscolo del documento canonico ESCLUSO
//!   il campo `view_hash`, nessun separatore 0x1F. Nodo serializzato
//!   `{"id":…,"key":…,"kind":…,"origin_seq":…}`; arco
//!   `{"dst_id":…,"origin_seq":…,"rel":…,"src_id":…}`; nodes ordinati per
//!   `id`, archi per `(src_id,dst_id,rel)`.
//! - Log "vuoto" ai fini L1 = genesi-only senza estrazioni: nodes/edges
//!   vuoti, head_hash della genesi, view_hash calcolato normalmente (D21).
//!
//! Determinismo: zero wall-clock, zero I/O non dichiarata, zero casualità.

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

use crate::log::Log;
use crate::payload::{write_escaped, Value};

/// Nodo della vista (SPEC §3 + D17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// SHA-256 minuscolo della serializzazione canonica di `{"key","kind"}`.
    pub id: String,
    pub key: String,
    pub kind: String,
    /// seq minimo di dichiarazione nel log.
    pub origin_seq: u64,
}

/// Arco della vista (SPEC §4 + D17): endpoint come id risolti.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub src_id: String,
    pub dst_id: String,
    pub rel: String,
    /// seq minimo degli eventi che provano lo stesso edge (D18).
    pub origin_seq: u64,
}

/// Vista L1 ancorata all'hash di testa (D14–D17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    /// "1" (D15).
    pub schema_version: String,
    /// hash dell'ultimo evento del log (D14).
    pub head_hash: String,
    /// ordinati per `id`.
    pub nodes: Vec<Node>,
    /// ordinati per `(src_id,dst_id,rel)`.
    pub edges: Vec<Edge>,
    /// SHA-256 minuscolo del documento canonico escluso `view_hash` (D16).
    pub view_hash: String,
}

impl View {
    /// Documento canonico completo, byte-per-byte: chiavi lessicografiche
    /// (edges, head_hash, nodes, schema_version, view_hash), nessuno spazio,
    /// escape N3 severo, campi di nodo/arco nell'ordine di D17.
    pub fn canonical(&self) -> String {
        let mut out = String::new();
        out.push_str("{\"edges\":[");
        for (i, e) in self.edges.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"dst_id\":");
            write_escaped(&mut out, &e.dst_id);
            out.push_str(",\"origin_seq\":");
            fmt_u64(&mut out, e.origin_seq);
            out.push_str(",\"rel\":");
            write_escaped(&mut out, &e.rel);
            out.push_str(",\"src_id\":");
            write_escaped(&mut out, &e.src_id);
            out.push('}');
        }
        out.push_str("],\"head_hash\":");
        write_escaped(&mut out, &self.head_hash);
        out.push_str(",\"nodes\":[");
        for (i, n) in self.nodes.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"id\":");
            write_escaped(&mut out, &n.id);
            out.push_str(",\"key\":");
            write_escaped(&mut out, &n.key);
            out.push_str(",\"kind\":");
            write_escaped(&mut out, &n.kind);
            out.push_str(",\"origin_seq\":");
            fmt_u64(&mut out, n.origin_seq);
            out.push('}');
        }
        out.push_str("],\"schema_version\":");
        write_escaped(&mut out, &self.schema_version);
        out.push_str(",\"view_hash\":");
        write_escaped(&mut out, &self.view_hash);
        out.push('}');
        out
    }

    /// Preimage di `view_hash` (D16): il documento canonico MINUS il campo
    /// `view_hash`. Interno: la prova è nella suite, che ricalcola
    /// indipendentemente l'SHA-256 di un documento costruito a parte.
    ///
    /// Il taglio è calcolato dalla lunghezza del valore ATTUALE, non da una
    /// costante: `build` chiama qui mentre `view_hash` è ancora vuoto, e un
    /// taglio fisso da 64 taglierebbe 63 caratteri di contenuto reale.
    /// L'hash è esa decimale minuscolo: `write_escaped` non aggiunge escape,
    /// quindi la lunghezza coincisa è `view_hash.len()`.
    fn minus_view_hash(&self) -> String {
        let full = self.canonical();
        let member = "\"view_hash\":";
        let cut = full.len() - 1 /* virgola */ - member.len() - 1 /* \" di apertura */
            - self.view_hash.len() - 2 /* \" di chiusura + } */;
        format!("{}}}", &full[..cut])
    }
}

fn fmt_u64(out: &mut String, v: u64) {
    use std::fmt::Write as _;
    let _ = write!(out, "{v}");
}

fn sha256_hex_lower(bytes: &[u8]) -> String {
    let dig = Sha256::digest(bytes);
    let mut out = String::with_capacity(dig.len() * 2);
    for b in dig {
        use std::fmt::Write as _;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Errori della ricostruzione della vista. La famiglia importa per i test:
/// `Schema` = payload estratto fuori forma, `Endpoint` = arco verso nodo
/// non ancora dichiarato, `Empty` è difensivo: il `Log` pubblico ha sempre
/// la genesi, mai zero eventi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Schema(String),
    Endpoint(String),
    Empty(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Schema(m) => write!(f, "schema vista: {m}"),
            Error::Endpoint(m) => write!(f, "endpoint non dichiarato: {m}"),
            Error::Empty(m) => write!(f, "log senza eventi: {m}"),
        }
    }
}

impl std::error::Error for Error {}

fn min_seq(prev: u64, seq: u64) -> u64 {
    if seq < prev {
        seq
    } else {
        prev
    }
}

/// id nodo: SHA-256 del JSON canonico di `{"key","kind"}` (ordine canonico
/// dei NOMI delle chiavi: "key" < "kind"), digest minuscolo — l'id è fermo
/// per contenuto, ovunque venga dichiarato.
fn node_id(kind: &str, key: &str) -> String {
    let mut preimage = String::new();
    preimage.push_str("{\"key\":");
    write_escaped(&mut preimage, key);
    preimage.push_str(",\"kind\":");
    write_escaped(&mut preimage, kind);
    preimage.push('}');
    sha256_hex_lower(preimage.as_bytes())
}

/// Ricostruzione della vista dal replay del log (SPEC §5): stesso log →
/// stesso grafo, byte per byte. Solo `&Log`: nessun mutamento, nessuna
/// scrittura da alcun lato del log.
pub fn build(log: &Log) -> Result<View, Error> {
    let head = log
        .events()
        .last()
        .ok_or_else(|| Error::Empty("nessun evento: il Log pubblico nasce sempre dalla genesi".to_string()))?
        .hash
        .clone();

    // (kind,key) → seq minimo di dichiarazione
    let mut nodes: BTreeMap<(String, String), u64> = BTreeMap::new();
    // trip di identità (src_id,dst_id,rel) → seq minimo di prova (D18)
    let mut edges: BTreeMap<(String, String, String), u64> = BTreeMap::new();

    for e in log.events() {
        // D19: nel singolo evento, l'array `nodes` è elaborato per intero
        // prima di `edges` — anche se il payload canonico elenca "edges"
        // prima di "nodes" (ordine lessicografico dei NOMI).
        if let Some(v) = e.payload.get("nodes") {
            for el in parse_node_array(v, e.seq)? {
                let seq = e.seq;
                nodes
                    .entry((el.kind, el.key))
                    .and_modify(|s| *s = min_seq(*s, seq))
                    .or_insert(seq);
            }
        }
        if let Some(v) = e.payload.get("edges") {
            for el in parse_edge_array(v, e.seq)? {
                // endpoint kind+key → id: solo nodi dichiarati PRIMA nel
                // replay (inclusive i nodi del medesimo evento, già inseriti
                // sopra); riferimento futuro = errore, no profezia.
                for (kind, key, role) in
                    [(&el.src_kind, &el.src_key, "src"), (&el.dst_kind, &el.dst_key, "dst")]
                {
                    if !nodes.contains_key(&(kind.clone(), key.clone())) {
                        return Err(Error::Endpoint(format!(
                            "endpoint {role} (kind {kind}, key {key}) dell'arco a seq {} non ancora dichiarato: nessun skip, nessuna autocreazione",
                            e.seq
                        )));
                    }
                }
                let seq = e.seq;
                edges
                    .entry((node_id(&el.src_kind, &el.src_key), node_id(&el.dst_kind, &el.dst_key), el.rel))
                    .and_modify(|s| *s = min_seq(*s, seq))
                    .or_insert(seq);
            }
        }
    }

    // uscita ordinata: nodi per id, archi per (src_id,dst_id,rel)
    let mut out_nodes: Vec<Node> = Vec::with_capacity(nodes.len());
    for ((kind, key), seq) in nodes {
        out_nodes.push(Node {
            id: node_id(&kind, &key),
            key,
            kind,
            origin_seq: seq,
        });
    }
    out_nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out_edges: Vec<Edge> = Vec::with_capacity(edges.len());
    for ((src_id, dst_id, rel), seq) in edges {
        out_edges.push(Edge {
            src_id,
            dst_id,
            rel,
            origin_seq: seq,
        });
    }
    out_edges.sort_by(|a, b| (&a.src_id, &a.dst_id, &a.rel).cmp(&(&b.src_id, &b.dst_id, &b.rel)));

    let view = View {
        schema_version: "1".to_string(),
        head_hash: head,
        nodes: out_nodes,
        edges: out_edges,
        view_hash: String::new(),
    };
    let view = View {
        view_hash: sha256_hex_lower(view.minus_view_hash().as_bytes()),
        ..view
    };
    Ok(view)
}

// ---------------------------------------------------------------------------
// Parser degli array JSON canonici interni (D1–D3).
//
// Strategia: parse rigorosamente tipato (oggetti esatti, solo valori
// stringa, chiavi senza duplicati fuori schema) + ricomposizione canonica
// della stringa INTERA byte-per-byte: chiavi ricomposte in ordine
// lessicografico, nessuno spazio, escape N3 severo (come `write_escaped`).
// La forma è un solo criterio misurato, come N3-bis in L0:
// `serialize(parse(src)) == src`.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct NodeEl {
    key: String,
    kind: String,
}

#[derive(Clone, Debug)]
struct EdgeEl {
    src_kind: String,
    src_key: String,
    dst_kind: String,
    dst_key: String,
    rel: String,
}

fn rel_valid(rel: &str) -> bool {
    !rel.is_empty() && rel.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// `nodes` nel payload L0: DEVE essere stringa con array canonico (D1);
/// qualunque altro tipo, incluso intero, è errore.
fn parse_node_array(value: &Value, seq: u64) -> Result<Vec<NodeEl>, Error> {
    match value {
        Value::Int(_) => Err(Error::Schema(format!(
            "nodes a seq {seq}: «nodes» deve essere una STRINGA che contiene l'array canonico (payload L0 invariato)"
        ))),
        Value::Str(s) => {
            let items = parse_array(s)?;
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                let (key, kind) = match item {
                    Elem::Node { key, kind } => (key, kind),
                    _ => unreachable!("parser di array di nodi, elementi di nodo"),
                };
                out.push(NodeEl { key, kind });
            }
            Ok(out)
        }
    }
}

/// `edges` nel payload L0: stesso contratto di `nodes`.
fn parse_edge_array(value: &Value, seq: u64) -> Result<Vec<EdgeEl>, Error> {
    match value {
        Value::Int(_) => Err(Error::Schema(format!(
            "edges a seq {seq}: «edges» deve essere una STRINGA che contiene l'array canonico (payload L0 invariato)"
        ))),
        Value::Str(s) => {
            let items = parse_array(s)?;
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                let Arc {
                    src_kind,
                    src_key,
                    dst_kind,
                    dst_key,
                    rel,
                } = match item {
                    Elem::Edge(el) => el,
                    _ => unreachable!("parser di array di archi, elementi di arco"),
                };
                let el = EdgeEl {
                    src_kind,
                    src_key,
                    dst_kind,
                    dst_key,
                    rel,
                };
                out.push(el);
            }
            Ok(out)
        }
    }
}

/// Elemento interno dopo il parse.
enum Elem {
    Node { key: String, kind: String },
    // renamed struct per il destructure
    // Edge { src_kind, src_key, dst_kind, dst_key, rel }
    Edge(Arc),
}

struct Arc {
    src_kind: String,
    src_key: String,
    dst_kind: String,
    dst_key: String,
    rel: String,
}

/// Parser dell'array interno con canonicità rigida dell'intera stringa.
fn parse_array(src: &str) -> Result<Vec<Elem>, Error> {
    let mut c = Cur { b: src.as_bytes(), i: 0 };
    c.expect(b'[')?;
    let mut items: Vec<Elem> = Vec::new();
    let mut first = true;
    loop {
        if c.peek() == Some(b']') {
            c.bump();
            break;
        }
        if !first {
            c.expect(b',')?;
        }
        first = false;
        let item = parse_elem(&mut c)?;
        if let Elem::Edge(ref e) = item {
            if !rel_valid(&e.rel) {
                return Err(Error::Schema(format!(
                    "rel fuori dominio: {} (regex [a-z0-9_]+)",
                    e.rel
                )));
            }
        }
        items.push(item);
    }
    if !c.at_end() {
        return Err(Error::Schema("residui oltre la chiusura dell'array".to_string()));
    }
    // Ricomposizione canonica dell'intera stringa (niente spazi, chiavi in
    // ordine lessicografico, escape N3 severo, ordine degli elementi dell'array).
    let mut canonical = String::new();
    canonical.push('[');
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            canonical.push(',');
        }
        match item {
            Elem::Node { key, kind } => {
                canonical.push_str("{\"key\":");
                write_escaped(&mut canonical, key);
                canonical.push_str(",\"kind\":");
                write_escaped(&mut canonical, kind);
                canonical.push('}');
            }
            Elem::Edge(e) => {
                canonical.push_str("{\"dst_key\":");
                write_escaped(&mut canonical, &e.dst_key);
                canonical.push_str(",\"dst_kind\":");
                write_escaped(&mut canonical, &e.dst_kind);
                canonical.push_str(",\"rel\":");
                write_escaped(&mut canonical, &e.rel);
                canonical.push_str(",\"src_key\":");
                write_escaped(&mut canonical, &e.src_key);
                canonical.push_str(",\"src_kind\":");
                write_escaped(&mut canonical, &e.src_kind);
                canonical.push('}');
            }
        }
    }
    canonical.push(']');
    if canonical.as_bytes() != c.b {
        return Err(Error::Schema(
            "array interno non canonico: la riserializzazione canonica non coincide byte-per-byte \
             con la stringa sorgente (ordine chiavi, spazi, ordine campi, escape non canonico)"
                .to_string(),
        ));
    }
    Ok(items)
}

/// Elemento dell'array interno: oggetto Esatto — le chiavi ammesse dipendono
/// dal fatto che l'array è interamente di nodi o interamente di archi.
/// La forma dell'arrivo non conta (parse in QUALSIASI ordine di chiavi): a
/// decidere è la ricomposizione byte-per-byte sopra.
fn parse_elem(c: &mut Cur) -> Result<Elem, Error> {
    c.expect(b'{')?;
    let mut first = true;
    let mut it_is_node_form: Option<bool> = None;
    let mut node_key: Option<String> = None;
    let mut node_kind: Option<String> = None;
    let mut src_kind: Option<String> = None;
    let mut src_key: Option<String> = None;
    let mut dst_kind: Option<String> = None;
    let mut dst_key: Option<String> = None;
    let mut rel: Option<String> = None;
    loop {
        if c.peek() == Some(b'}') {
            c.bump();
            break;
        }
        if !first {
            c.expect(b',')?;
        }
        first = false;
        let k = parse_string(c)?;
        c.expect(b':')?;
        match k.as_str() {
            "key" | "kind" => {
                if it_is_node_form == Some(false) {
                    return Err(Error::Schema(format!(
                        "elemento di arco con campo di nodo: {k}"
                    )));
                }
                it_is_node_form = Some(true);
                match parse_string(c)? {
                    v if k == "key" => {
                        if node_key.replace(v).is_some() {
                            return Err(Error::Schema(format!("campo duplicato: {k}")));
                        }
                    }
                    v if k == "kind" => {
                        if node_kind.replace(v).is_some() {
                            return Err(Error::Schema(format!("campo duplicato: {k}")));
                        }
                    }
                    _ => unreachable!(),
                }
            }
            "src_kind" | "src_key" | "dst_kind" | "dst_key" | "rel" => {
                if it_is_node_form == Some(true) {
                    return Err(Error::Schema(format!(
                        "elemento di nodo con campo di arco: {k}"
                    )));
                }
                it_is_node_form = Some(false);
                match parse_string(c)? {
                    v if k == "src_kind" => {
                        if src_kind.replace(v).is_some() {
                            return Err(Error::Schema(format!("campo duplicato: {k}")));
                        }
                    }
                    v if k == "src_key" => {
                        if src_key.replace(v).is_some() {
                            return Err(Error::Schema(format!("campo duplicato: {k}")));
                        }
                    }
                    v if k == "dst_kind" => {
                        if dst_kind.replace(v).is_some() {
                            return Err(Error::Schema(format!("campo duplicato: {k}")));
                        }
                    }
                    v if k == "dst_key" => {
                        if dst_key.replace(v).is_some() {
                            return Err(Error::Schema(format!("campo duplicato: {k}")));
                        }
                    }
                    v if k == "rel" => {
                        if rel.replace(v).is_some() {
                            return Err(Error::Schema(format!("campo duplicato: {k}")));
                        }
                    }
                    _ => unreachable!(),
                }
            }
            // campo extra: le chiavi ammesse anticipano la forma
            _ => return Err(Error::Schema(format!("campo non previsto: {k}"))),
        }
    }
    match it_is_node_form {
        Some(true) => {
            let (key, kind) = (node_key, node_kind);
            match (key, kind) {
                (Some(key), Some(kind)) => Ok(Elem::Node { key, kind }),
                _ => Err(Error::Schema(
                    "elemento di nodo incompleto: manca key o kind".to_string(),
                )),
            }
        }
        Some(false) => {
            if let (Some(src_kind), Some(src_key), Some(dst_kind), Some(dst_key), Some(rel)) =
                (src_kind, src_key, dst_kind, dst_key, rel)
            {
                Ok(Elem::Edge(Arc {
                    src_kind,
                    src_key,
                    dst_kind,
                    dst_key,
                    rel,
                }))
            } else {
                Err(Error::Schema(
                    "elemento di arco incompleto: manca uno di src_kind|src_key|dst_kind|dst_key|rel"
                        .to_string(),
                ))
            }
        }
        None => Err(Error::Schema("elemento senza campi: atteso key/kind per i nodi, src/dst per gli archi".to_string())),
    }
}

// ---------------------------------------------------------------------------
// Cursor simile a quello di L0 + stringa canonica N3 severa.
// ---------------------------------------------------------------------------

struct Cur<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Cur<'a> {
    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }
    fn bump(&mut self) -> Option<u8> {
        let c = self.peek();
        if c.is_some() {
            self.i += 1;
        }
        c
    }
    fn expect(&mut self, c: u8) -> Result<(), Error> {
        match self.bump() {
            Some(x) if x == c => Ok(()),
            Some(x) => Err(Error::Schema(format!(
                "atteso '{}', trovato '{}' nell'array interno",
                c as char, x as char
            ))),
            None => Err(Error::Schema("fine stringa inattesa nell'array interno".to_string())),
        }
    }
    fn at_end(&self) -> bool {
        self.i >= self.b.len()
    }
}

/// Stringa JSON canonica dentro l'array interno: STESSO criterio N3 di L0 —
/// è ammesso ESATTAMENTE ciò che `payload::write_escaped` emette (minimo
/// `\" \\ \b \t \n \f \r`, controlli <0x20 come \uXXXX minuscolo a 4 cifre
/// minimo importantissimo, tutto il resto UTF-8 diretto); il token sorgente
/// deve coincidere byte-per-byte con la ricomposizione canonica della stringa
/// decodificata (parse∘serialize = identità).
fn parse_string(c: &mut Cur) -> Result<String, Error> {
    let start = c.i;
    c.expect(b'"')?;
    let mut out: Vec<u8> = Vec::new();
    loop {
        match c.bump() {
            None => return Err(Error::Schema("stringa non chiusa nell'array interno".to_string())),
            Some(b'"') => break,
            Some(b'\\') => match c.bump() {
                Some(b'"') => out.push(b'"'),
                Some(b'\\') => out.push(b'\\'),
                Some(b'/') => out.push(b'/'),
                Some(b'b') => out.push(0x08),
                Some(b'f') => out.push(0x0C),
                Some(b'n') => out.push(b'\n'),
                Some(b'r') => out.push(b'\r'),
                Some(b't') => out.push(b'\t'),
                Some(b'u') => {
                    let code = parse_u4x(c)?;
                    if (0xD800..=0xDBFF).contains(&code) {
                        if c.peek() != Some(b'\\') {
                            return Err(Error::Schema("surrogate alta senza coppia".to_string()));
                        }
                        c.bump();
                        c.expect(b'u')?;
                        let low = parse_u4x(c)?;
                        if !(0xDC00..=0xDFFF).contains(&low) {
                            return Err(Error::Schema("coppia surrogate non valida".to_string()));
                        }
                        let combined = 0x10000u32 + ((code - 0xD800) << 10) + (low - 0xDC00);
                        push_rune(&mut out, combined)?;
                    } else if (0xDC00..=0xDFFF).contains(&code) {
                        return Err(Error::Schema("surrogate bassa isolata".to_string()));
                    } else {
                        push_rune(&mut out, code)?;
                    }
                }
                _ => return Err(Error::Schema("escape non valido nell'array interno".to_string())),
            },
            Some(raw) => {
                if raw < 0x20 {
                    return Err(Error::Schema(format!(
                        "carattere di controllo grezzo 0x{raw:02x} nell'array interno"
                    )));
                }
                out.push(raw);
            }
        }
    }
    let decoded = String::from_utf8(out).map_err(|_| Error::Schema("stringa interna non UTF-8".to_string()))?;
    let mut canonical = String::new();
    write_escaped(&mut canonical, &decoded);
    if canonical.as_bytes() != &c.b[start..c.i] {
        return Err(Error::Schema(
            "stringa interna non canonica: l'encoding non coincide con l'output del serializzatore (write_escaped)"
                .to_string(),
        ));
    }
    Ok(decoded)
}

fn push_rune(out: &mut Vec<u8>, code: u32) -> Result<(), Error> {
    let ch = char::from_u32(code).ok_or_else(|| Error::Schema("codepoint non valido".to_string()))?;
    let mut buf = [0u8; 4];
    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
    Ok(())
}

fn parse_u4x(c: &mut Cur) -> Result<u32, Error> {
    let mut v = 0u32;
    for _ in 0..4 {
        let d = match c.bump() {
            Some(b) if b.is_ascii_hexdigit() => (b as char).to_digit(16).unwrap(),
            _ => return Err(Error::Schema("escape \\u malformato".to_string())),
        };
        v = v * 16 + d;
    }
    Ok(v)
}
