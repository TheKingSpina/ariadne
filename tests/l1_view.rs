//! L1 — SPEC v0.1 §2–§6 + addendum umano D1–D13 + decisioni operative D14–D21
//! del dispatch dell'orchestratore (23/10/2026).
//!
//! Questo file è il commit TEST-ONLY: se resta solo qui, la suite è ROSSA
//! perché l'API `ariadne::view` non esiste ancora nel crate.
//!
//! Decisioni pinnate byte-per-byte dai test (da audit del reviewer; NON
//! pre-auditate):
//! - D14: `head_hash` = hash dell'ultimo evento del log (compresa la genesi-only);
//! - D15: `schema_version` = stringa `"1"`;
//! - D16: `view_hash` = SHA-256 minuscolo del documento canonico completo
//!   ESCLUSO il campo `view_hash` stesso; nessun separatore 0x1F;
//! - D17: nodo `{"id","key","kind","origin_seq"}`; arco `{"dst_id","origin_seq","rel","src_id"}`;
//! - D18: arco riprovato da più eventi → `origin_seq` minimo; unicità per (src_id,dst_id,rel);
//! - D19: nello stesso evento, l'array `nodes` è elaborato per intero prima di `edges`;
//! - D20: il builder NON rivérifica la catena L0 (nessun re-hash, nessun `verify`);
//! - D21: log "vuoto" = genesi-only senza estrazioni: nodes/edges vuoti,
//!   head_hash della genesi, view_hash calcolato normalmente.
//!
//! Da D1–D13 (addendum umano): payload L0 invariato (valori top-level solo
//! i64/stringhe; `nodes`/`edges` stringhe con array JSON canonico interno,
//! oggetti ESATTI `{kind,key}` e `{src_kind,src_key,dst_kind,dst_key,rel}`);
//! id nodo = SHA-256 minuscolo del JSON canonico `{"key":…,"kind":…}`;
//! origin_seq nodo = seq minimo di dichiarazione; rel `[a-z0-9_]+`; endpoint
//! risolti in id durante il replay, riferimento a nodo non ancora dichiarato
//! = errore (no skip, no autocreazione); cicli ammessi e richiesti (test
//! ≥4 nodi con A→B→C→A); builder su `&Log`, zero scritture (digest log
//! prima/dopo); vista JSON canonico, nodes ordinati per id, edges per
//! (src_id,dst_id,rel).

use ariadne::view::{self, Edge, Error, Node, View};
use ariadne::{EventType, Log, Payload, Value};

// ---------------------------------------------------------------------------
// Golden indipendenti: SHA-256 ricalcolati FUORI dal crate con `shasum -a 256`,
// comandi e output verbatim registrati nel messaggio del commit di test.
// ---------------------------------------------------------------------------

/// Hash di testa della genesi seed 1042 (già golden accettato in L0).
const GENESIS_1042_HASH: &str = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";

/// id di `{"key":"A","kind":"letter"}`:
///   printf '{"key":"A","kind":"letter"}' | shasum -a 256
const NODE_A_LETTER_ID: &str = "7a8edd9c537ac65f451833c61f37c940e0c49fd50f9705d2f8b6cf405d045d69";

/// id di `{"key":"B","kind":"letter"}`:
///   printf '{"key":"B","kind":"letter"}' | shasum -a 256
const NODE_B_LETTER_ID: &str = "230bab4a49e079fc2eafbbeffc801254745087addc3e51bcb76ff4a9bed9b4f7";

/// view_hash della vista genesi-only (documento minus-hash):
///   printf '{"edges":[],"head_hash":<gen>,"nodes":[],"schema_version":"1"}' | shasum -a 256
const GENESIS_ONLY_VIEW_HASH: &str =
    "60e804ca9c2d481f391e872674274d9492667be663be238897cc517caf8dc507";

// ---------------------------------------------------------------------------
// Helpers di test
// ---------------------------------------------------------------------------

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// id nodo: SHA-256 del JSON canonico di {key,kind} (chiavi in ordine
/// lessicografico: "key" prima di "kind"), digest minuscolo.
fn nid(key: &str, kind: &str) -> String {
    sha256_hex(format!("{{\"key\":\"{key}\",\"kind\":\"{kind}\"}}").as_bytes())
}

/// Elemento interno canonico di nodo: chiavi in ordine lessicografico.
fn n_el(key: &str, kind: &str) -> String {
    format!("{{\"key\":\"{key}\",\"kind\":\"{kind}\"}}")
}

/// Elemento interno canonico di arco: chiavi in ordine lessicografico
/// dst_key < dst_kind < rel < src_key < src_kind.
fn e_el(src_key: &str, src_kind: &str, dst_key: &str, dst_kind: &str, rel: &str) -> String {
    format!(
        "{{\"dst_key\":\"{dst_key}\",\"dst_kind\":\"{dst_kind}\",\"rel\":\"{rel}\",\"src_key\":\"{src_key}\",\"src_kind\":\"{src_kind}\"}}"
    )
}

/// Array JSON canonico interno (nessuno spazio, virgole senza spazi).
fn arr(items: &[String]) -> String {
    format!("[{}]", items.join(","))
}

fn app_payload(log: &mut Log, tick: u64, pairs: Vec<(&str, Value)>) -> u64 {
    log.append(tick, EventType::Action, Payload::from_pairs(pairs)).unwrap();
    log.events().last().unwrap().seq
}

fn app_nodes(log: &mut Log, tick: u64, items: &[(&str, &str)]) -> u64 {
    let nodes = arr(&items.iter().map(|(k, kind)| n_el(k, kind)).collect::<Vec<_>>());
    app_payload(log, tick, vec![("nodes", Value::Str(nodes))])
}

fn app_edges(log: &mut Log, tick: u64, items: &[(&str, &str, &str, &str, &str)]) -> u64 {
    let edges = arr(
        &items
            .iter()
            .map(|(sk, skind, dk, dkind, rel)| e_el(sk, skind, dk, dkind, rel))
            .collect::<Vec<_>>(),
    );
    app_payload(log, tick, vec![("edges", Value::Str(edges))])
}

fn app_both(
    log: &mut Log,
    tick: u64,
    nodes: &[(&str, &str)],
    edges: &[(&str, &str, &str, &str, &str)],
) -> u64 {
    let mut pairs = vec![];
    if !edges.is_empty() {
        let edges_json = arr(
            &edges
                .iter()
                .map(|(sk, skind, dk, dkind, rel)| e_el(sk, skind, dk, dkind, rel))
                .collect::<Vec<_>>(),
        );
        pairs.push(("edges", Value::Str(edges_json)));
    }
    if !nodes.is_empty() {
        let nodes_json = arr(&nodes.iter().map(|(k, kind)| n_el(k, kind)).collect::<Vec<_>>());
        pairs.push(("nodes", Value::Str(nodes_json)));
    }
    app_payload(log, tick, pairs)
}

/// Nodo e arco attesi, costruiti nel test con la struttura D17.
fn expects(nodes: &[(&str, &str, u64)], triples: &[(&str, &str, &str, &str, &str, u64)]) -> (Vec<Node>, Vec<Edge>) {
    let mut ns: Vec<Node> = nodes
        .iter()
        .map(|(k, kind, seq)| Node { id: nid(k, kind), key: k.to_string(), kind: kind.to_string(), origin_seq: *seq })
        .collect();
    ns.sort_by(|a, b| a.id.cmp(&b.id));
    let mut es: Vec<Edge> = triples
        .iter()
        .map(|(sk, skind, dk, dkind, rel, seq)| Edge {
            src_id: nid(sk, skind),
            dst_id: nid(dk, dkind),
            rel: rel.to_string(),
            origin_seq: *seq,
        })
        .collect();
    es.sort_by(|a, b| (&a.src_id, &a.dst_id, &a.rel).cmp(&(&b.src_id, &b.dst_id, &b.rel)));
    (ns, es)
}

/// Documento canonico MINUS view_hash, ricostruito indipendentemente (D17/D16).
fn minus_hash(head_hash: &str, ns: &[Node], es: &[Edge]) -> String {
    let nodes_json = ns
        .iter()
        .map(|n| format!("{{\"id\":\"{}\",\"key\":\"{}\",\"kind\":\"{}\",\"origin_seq\":{}}}", n.id, n.key, n.kind, n.origin_seq))
        .collect::<Vec<_>>()
        .join(",");
    let edges_json = es
        .iter()
        .map(|e| format!("{{\"dst_id\":\"{}\",\"origin_seq\":{},\"rel\":\"{}\",\"src_id\":\"{}\"}}", e.dst_id, e.origin_seq, e.rel, e.src_id))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"edges\":[{edges_json}],\"head_hash\":\"{head_hash}\",\"nodes\":[{nodes_json}],\"schema_version\":\"1\"}}")
}

/// Controllo d'ancoraggio e di formato della vista in un solo punto:
/// - head_hash == atteso (D14), schema_version == "1" (D15);
/// - struct nodes/edges == attesi (D17, ordine di output);
/// - canonical() == documento atteso + view_hash;
/// - view_hash == SHA-256 del documento minus view_hash (D16).
fn assert_view(view: &View, head_hash: &str, nodes: &[(&str, &str, u64)], triples: &[(&str, &str, &str, &str, &str, u64)]) {
    let (enodes, eedges) = expects(nodes, triples);
    assert_eq!(view.schema_version, "1");
    assert_eq!(view.head_hash, head_hash);
    assert_eq!(&view.nodes, &enodes, "nodi attesi (ordine: id)");
    assert_eq!(&view.edges, &eedges, "archi attesi (ordine: (src_id,dst_id,rel))");
    let doc = minus_hash(head_hash, &enodes, &eedges);
    let vh = sha256_hex(doc.as_bytes());
    assert_eq!(view.view_hash, vh, "view_hash = SHA-256 minuscolo del documento escluso view_hash (D16)");
    assert_eq!(view.canonical(), format!("{},\"view_hash\":\"{vh}\"}}", &doc[..doc.len() - 1]));
}

/// Cerca un arco per chiavi endpoint logiche (kind risolto dal nome).
fn find_edge(view: &View, sk: &str, skind: &str, dk: &str, dkind: &str, rel: &str) -> bool {
    view.edges.iter().any(|e| {
        e.src_id == nid(sk, skind) && e.dst_id == nid(dk, dkind) && e.rel == rel
    })
}

// ---------------------------------------------------------------------------
// D21 — log "vuoto": genesi-only senza estrazioni
// ---------------------------------------------------------------------------

#[test]
fn genesi_sola_vista_vuota_ancorata_head_hash_d21() {
    let log = Log::genesis(1042).unwrap();
    let v = view::build(&log).expect("genesi-only: vista vuota valida");
    assert_view(&v, GENESIS_1042_HASH, &[], &[]);
    // golden esterno del documento: view_hash della genesi-only è KNOWN.
    assert_eq!(
        v.canonical(),
        format!(
            "{{\"edges\":[],\"head_hash\":\"{GENESIS_1042_HASH}\",\"nodes\":[],\"schema_version\":\"1\",\"view_hash\":\"{GENESIS_ONLY_VIEW_HASH}\"}}"
        )
    );
}

// ---------------------------------------------------------------------------
// Provenienza e formato (SPEC §3, §4, §5; D17)
// ---------------------------------------------------------------------------

#[test]
fn nodo_golden_id_origin_seq_unico_evento() {
    let mut log = Log::genesis(1042).unwrap();
    app_nodes(&mut log, 1, &[("A", "letter"), ("B", "letter")]);
    let v = view::build(&log).unwrap();
    let head = log.events().last().unwrap().hash.clone();
    assert_view(&v, &head, &[("A", "letter", 1), ("B", "letter", 1)], &[]);
    // golden letterali dal `shasum` indipendente (ordine per id: 230b… < 7a8e…)
    assert_eq!(v.nodes[0].id, NODE_B_LETTER_ID);
    assert_eq!(v.nodes[1].id, NODE_A_LETTER_ID);
    assert_eq!(v.nodes[0].key, "B");
    assert_eq!(v.nodes[1].key, "A");
}

#[test]
fn stesso_key_kind_diversi_nodi_distinti_e_arco_cross_kind() {
    let mut log = Log::genesis(1042).unwrap();
    app_both(&mut log, 1, &[("A", "letter"), ("A", "digit")], &[("A", "digit", "A", "letter", "cross")]);
    let v = view::build(&log).unwrap();
    let head = log.events().last().unwrap().hash.clone();
    assert_view(&v, &head, &[("A", "digit", 1), ("A", "letter", 1)], &[("A", "digit", "A", "letter", "cross", 1)]);
    assert_ne!(nid("A", "digit"), nid("A", "letter"));
}

#[test]
fn arco_unicita_per_trip_e_origin_seq_minimo_d18() {
    let mut log = Log::genesis(1042).unwrap();
    app_nodes(&mut log, 1, &[("A", "letter"), ("B", "letter")]);
    app_edges(&mut log, 2, &[("A", "letter", "B", "letter", "next")]);
    let s3 = app_nodes(&mut log, 1, &[("A", "letter")]); // riprovato: origin_seq resta 1
    let s4 = app_edges(&mut log, 3, &[("A", "letter", "B", "letter", "next")]); // riprovato: origin_seq resta 2
    let v = view::build(&log).unwrap();
    let head = log.events().last().unwrap().hash.clone();
    assert_view(&v, &head, &[("A", "letter", 1), ("B", "letter", 1)], &[("A", "letter", "B", "letter", "next", 2)]);
    assert!(s4 > s3, "sanità del costruttore: seq cresce");
    // stessa coppia con rel diversa = arco DISTINTO, mappa `rel` base attr
    let s5 = app_edges(&mut log, 4, &[("A", "letter", "B", "letter", "alpha")]);
    let v2 = view::build(&log).unwrap();
    let head2 = log.events().last().unwrap().hash.clone();
    assert_view(
        &v2,
        &head2,
        &[("A", "letter", 1), ("B", "letter", 1)],
        &[("A", "letter", "B", "letter", "alpha", s5), ("A", "letter", "B", "letter", "next", 2)],
    );
}

// ---------------------------------------------------------------------------
// Ciclo (SPEC §1, addendum): almeno 4 nodi, A→B→C→A
// ---------------------------------------------------------------------------

#[test]
fn ciclo_quattro_nodi_determinismo_due_esecuzioni_byte_per_byte() {
    let build_log = || -> Log {
        let mut log = Log::genesis(1042).unwrap();
        app_nodes(&mut log, 1, &[("A", "letter"), ("B", "letter"), ("C", "letter"), ("D", "letter")]);
        app_edges(&mut log, 2, &[("A", "letter", "B", "letter", "next")]);
        app_edges(&mut log, 3, &[("B", "letter", "C", "letter", "next")]);
        app_edges(&mut log, 4, &[("C", "letter", "A", "letter", "next")]);
        app_edges(&mut log, 5, &[("A", "letter", "D", "letter", "next"), ("D", "letter", "C", "letter", "next")]);
        log
    };
    let log1 = build_log();
    let log2 = build_log();
    let v1 = view::build(&log1).unwrap();
    let v2 = view::build(&log2).unwrap();
    // due esecuzioni indipendenti → stesso buffer, byte per byte (checklist 1)
    assert_eq!(v1.canonical(), v2.canonical());
    let head = log1.events().last().unwrap().hash.clone();
    assert_view(
        &v1,
        &head,
        &[("A", "letter", 1), ("B", "letter", 1), ("C", "letter", 1), ("D", "letter", 1)],
        &[
            ("A", "letter", "B", "letter", "next", 2),
            ("A", "letter", "D", "letter", "next", 5),
            ("B", "letter", "C", "letter", "next", 3),
            ("C", "letter", "A", "letter", "next", 4),
            ("D", "letter", "C", "letter", "next", 5),
        ],
    );
    // il ciclo A→B→C→A è presente (nessun rifiuto)
    assert!(find_edge(&v1, "A", "letter", "B", "letter", "next"), "A→B");
    assert!(find_edge(&v1, "B", "letter", "C", "letter", "next"), "B→C");
    assert!(find_edge(&v1, "C", "letter", "A", "letter", "next"), "C→A");
    // replay dal byte: lo stesso log serializzato ricaricato dà la STESSA vista
    let ricaricato = Log::from_jsonl(&log1.to_jsonl()).unwrap();
    assert_eq!(view::build(&ricaricato).unwrap().canonical(), v1.canonical());
}

// ---------------------------------------------------------------------------
// D19 — nello stesso evento, nodes prima di edges
// ---------------------------------------------------------------------------

#[test]
fn nodi_prima_degli_archi_nello_stesso_evento_d19() {
    let mut log = Log::genesis(1042).unwrap();
    app_both(&mut log, 1, &[("A", "letter"), ("B", "letter")], &[("A", "letter", "B", "letter", "next")]);
    let v = view::build(&log).unwrap();
    let head = log.events().last().unwrap().hash.clone();
    assert_view(&v, &head, &[("A", "letter", 1), ("B", "letter", 1)], &[("A", "letter", "B", "letter", "next", 1)]);
}

// ---------------------------------------------------------------------------
// Endpoint non dichiarati = errore (no skip, no autocreazione, ordine di replay)
// ---------------------------------------------------------------------------

#[test]
fn endpoint_non_dichiarato_errore_no_skip_no_autocreazione() {
    // caso 1: Z non esiste da nessuna parte
    let mut log = Log::genesis(1042).unwrap();
    app_nodes(&mut log, 1, &[("A", "letter")]);
    app_edges(&mut log, 2, &[("A", "letter", "Z", "letter", "next")]);
    match view::build(&log) {
        Err(Error::Endpoint(_)) => {}
        other => panic!("atteso errore Endpoint, trovato: {other:?}"),
    }
    // caso 2: il nodo esiste ma è dichiarato DOPO l'arco (ordine di replay):
    // nessuna profezia, l'arco a seq 1 non è ancora provabile
    let mut log2 = Log::genesis(1042).unwrap();
    app_edges(&mut log2, 1, &[("A", "letter", "B", "letter", "next")]);
    let s2 = app_nodes(&mut log2, 2, &[("A", "letter"), ("B", "letter")]);
    assert_eq!(s2, 2);
    match view::build(&log2) {
        Err(Error::Endpoint(_)) => {}
        other => panic!("atteso errore Endpoint (dichiarazione futura ≠ presente), trovato: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Payload estratti non validi = errore (D1–D3: forma canonica severa)
// ---------------------------------------------------------------------------

#[test]
fn payload_estratti_non_validi_errore() {
    let mut log = Log::genesis(1042).unwrap();
    // base: nodi A,B dichiarati (seq 1) — così i casi di errore di FORMA non
    // collidono con l'errore d'endpoint (che è un'altra famiglia di test).
    app_nodes(&mut log, 1, &[("A", "letter"), ("B", "letter")]);

    // (a) `nodes` NON è una stringa (tipo sbagliato) → errore
    let l1 = { let mut l = log.clone(); app_payload(&mut l, 1, vec![("nodes", Value::Int(7))]); l };
    match view::build(&l1) {
        Err(Error::Schema(_)) => {}
        other => panic!("nodes non stringa: atteso errore Schema, trovato {other:?}"),
    }

    // (b) contenuto non è un array → errore
    let l2 = { let mut l = log.clone(); app_payload(&mut l, 1, vec![("nodes", Value::Str(n_el("A", "letter")))]) ; l };
    match view::build(&l2) {
        Err(Error::Schema(_)) => {}
        other => panic!("nodes object invece che array: atteso Schema, trovato {other:?}"),
    }

    // (c) campo extra nell'elemento → errore
    let l3 = { let mut l = log.clone();
        app_payload(&mut l, 1, vec![("nodes", Value::Str(arr(&[
            "{\"extra\":\"x\",\"key\":\"A\",\"kind\":\"letter\"}".to_string(),
        ])))]);
        l };
    match view::build(&l3) {
        Err(Error::Schema(_)) => {}
        other => panic!("campo extra: atteso Schema, trovato {other:?}"),
    }

    // (d) campo mancante nell'elemento → errore
    let l4 = { let mut l = log.clone();
        app_payload(&mut l, 1, vec![("nodes", Value::Str(arr(&[
            "{\"kind\":\"letter\"}".to_string(),
        ])))]);
        l };
    match view::build(&l4) {
        Err(Error::Schema(_)) => {}
        other => panic!("campo mancante: atteso Schema, trovato {other:?}"),
    }

    // (e) tipo sbagliato nel campo → errore (kind intero)
    let l5 = { let mut l = log.clone();
        app_payload(&mut l, 1, vec![("nodes", Value::Str(arr(&[
            "{\"key\":\"A\",\"kind\":1}".to_string(),
        ])))]);
        l };
    match view::build(&l5) {
        Err(Error::Schema(_)) => {}
        other => panic!("kind intero: atteso Schema, trovato {other:?}"),
    }

    // (f) rel fuori dominio `[a-z0-9_]+` → errore
    let l6 = { let mut l = log.clone();
        app_payload(&mut l, 1, vec![("edges", Value::Str(arr(&[
            e_el("A", "letter", "B", "letter", "Next").to_string(),
        ])))]);
        l };
    match view::build(&l6) {
        Err(Error::Schema(_)) => {}
        other => panic!("rel con maiuscola: atteso Schema, trovato {other:?}"),
    }

    // (g) ordine chiavi interno non canonico (kind prima di key) → errore
    let l7 = { let mut l = log.clone();
        app_payload(&mut l, 1, vec![("nodes", Value::Str(arr(&[
            "{\"kind\":\"letter\",\"key\":\"A\"}".to_string(),
        ])))]);
        l };
    match view::build(&l7) {
        Err(Error::Schema(_)) => {}
        other => panic!("ordine chiavi interno non canonico: atteso Schema, trovato {other:?}"),
    }

    // (h) spazi dentro l'array interno → non canonico → errore
    let l8 = { let mut l = log.clone();
        app_payload(&mut l, 1, vec![("nodes", Value::Str("[ {\"key\":\"A\",\"kind\":\"letter\"} ]".to_string()))]);
        l };
    match view::build(&l8) {
        Err(Error::Schema(_)) => {}
        other => panic!("spazi non canonici: atteso Schema, trovato {other:?}"),
    }

    // (i) arco con campo extra → errore
    let l9 = { let mut l = log.clone();
        app_payload(&mut l, 1, vec![("edges", Value::Str(arr(&[
            "{\"dst_id\":\"x\",\"dst_key\":\"B\",\"dst_kind\":\"letter\",\"rel\":\"next\",\"src_key\":\"A\",\"src_kind\":\"letter\"}".to_string(),
        ])))]);
        l };
    match view::build(&l9) {
        Err(Error::Schema(_)) => {}
        other => panic!("arco con dst_id grezzo: atteso Schema, trovato {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Eventi senza estrazioni contribuiscono zero (e array vuoto ammesso)
// ---------------------------------------------------------------------------

#[test]
fn eventi_senza_estrazioni_zero_e_array_vuoto() {
    let mut log = Log::genesis(1042).unwrap();
    let s1 = app_both(&mut log, 1, &[("A", "letter")], &[]); // nodes "[]" ammesso
    app_payload(&mut log, 1, vec![("gain", Value::Int(1))]);
    app_payload(&mut log, 2, vec![("it", Value::Str("testo che CONTIENE le parole nodes e edges nel valore".to_string()))]);
    let s4 = app_payload(&mut log, 3, vec![("nodes", Value::Str("[]".to_string()))]);
    let v = view::build(&log).unwrap();
    let head = log.events().last().unwrap().hash.clone();
    // solo il primo evento ha estratto; gli altri tre = zero contributi
    assert_view(&v, &head, &[("A", "letter", s1)], &[]);
    assert_eq!(s1, 1);
    assert!(s4 > 1);
    assert_eq!(v.nodes.len(), 1);
}

// ---------------------------------------------------------------------------
// D20 — il builder NON rivérifica la catena L0
// ---------------------------------------------------------------------------

#[test]
fn builder_non_riverifica_catena_d20() {
    let mut log = Log::genesis(1042).unwrap();
    app_nodes(&mut log, 1, &[("A", "letter"), ("B", "letter")]);
    app_edges(&mut log, 2, &[("A", "letter", "B", "letter", "next")]);
    // log con hash manomessi: la VALIDAZIONE di schema di from_jsonl passa
    // (i token restano esa decimali minuscoli di 64), la catena non è più
    // verificabile: il builder deve produrre la vista comunque (D20 — la
    // verifica è di L0, non di L1).
    let manomesso = log
        .to_jsonl()
        .lines()
        .map(|line| {
            let start = line.find("\"hash\":\"").unwrap() + 8;
            format!("{}{}{}", &line[..start], "a".repeat(64), &line[start + 64..])
        })
        .collect::<Vec<_>>()
        .join("\n");
    let manomesso = format!("{}\n", manomesso);
    let ricaricato = Log::from_jsonl(&manomesso).expect("schema valido, catena non verificata");
    assert_eq!(ricaricato.events().len(), 3);
    let v = view::build(&ricaricato).expect("il builder assume il log verificato da L0 (D20): non rivérifica");
    // la vista è comunque deterministica e ancorata all'ultimo hash REGISTRATO
    let head = ricaricato.events().last().unwrap().hash.clone();
    assert!(head.chars().all(|c| c == 'a'), "hash manomessi restano quelli registrati");
    assert_view(&v, &head, &[("A", "letter", 1), ("B", "letter", 1)], &[("A", "letter", "B", "letter", "next", 2)]);
}

// ---------------------------------------------------------------------------
// Zero scritture: digest del log prima e dopo il build (checklist 4)
// ---------------------------------------------------------------------------

#[test]
fn zero_scritture_digest_log_intatto() {
    let mut log = Log::genesis(1042).unwrap();
    app_nodes(&mut log, 1, &[("A", "letter"), ("B", "letter"), ("C", "letter")]);
    app_edges(&mut log, 2, &[("A", "letter", "B", "letter", "next"), ("B", "letter", "C", "letter", "next")]);
    let prima_jsonl = log.to_jsonl();
    let prima_digest = sha256_hex(prima_jsonl.as_bytes());
    let prima_len = log.events().len();
    let _v = view::build(&log).unwrap();
    let dopo_jsonl = log.to_jsonl();
    let dopo_digest = sha256_hex(dopo_jsonl.as_bytes());
    assert_eq!(prima_jsonl, dopo_jsonl, "il builder non muta il log");
    assert_eq!(prima_digest, dopo_digest, "digest del log invariato prima/dopo il build");
    assert_eq!(prima_len, log.events().len());
}
