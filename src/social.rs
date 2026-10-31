//! L5 — legge sociale (modelli degli altri e teoria della mente).
//!
//! Implementazione della legge `notes/L5_legge_sociale.md` (commit `23a7104`,
//! delega umana esplicita 31/10/2026 su tutti i default del triage
//! `notes/L5_domande_sociale.md` + Q4(a)).
//!
//! Articoli realizzati: L5.1 (dominio chiuso `predict | model_update`),
//! L5.2 (`osservazioni` F1 su SENSE solo), L5.3 (id canonico), L5.4
//! (cutoff/interazioni), L5.5 (modello con contatori saturati), L5.6
//! (argmax, tie-break classe minore, `NoModel`, `sources` dedup crescente),
//! L5.7 (finestra/un update, replay autorevole), L5.8 (composizione
//! identità, solo costruzione), L5.9 (zero scritture L5 nel log), L5.10
//! (API pubblica), L5.11 (zero draw).

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::payload::write_escaped;
use crate::{Event, EventType, Log, Value};

// ---------------------------------------------------------------------------
// utility deterministiche
// ---------------------------------------------------------------------------

fn sha(bytes: &[u8]) -> String {
    let dig = Sha256::digest(bytes);
    dig.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// campo stringa del payload; mancante/non-stringa → Schema.
fn str_field<'a>(e: &'a Event, key: &str) -> Result<&'a str, SocialError> {
    match e.payload.get(key) {
        Some(Value::Str(s)) => Ok(s),
        _ => Err(SocialError::Schema),
    }
}

/// campo i64 ≥ 0 del payload; mancante/non-i64/negativo → Schema.
fn nonneg_field(e: &Event, key: &str) -> Result<u64, SocialError> {
    match e.payload.get(key) {
        Some(Value::Int(n)) if *n >= 0 => Ok(*n as u64),
        _ => Err(SocialError::Schema),
    }
}

/// schema chiuso (pattern L2.21/L3.1/L4.1): chiavi del payload ⊆ dominio.
fn exact_keys(e: &Event, extra: &[&str]) -> Result<(), SocialError> {
    if e.payload.iter().all(|(k, _)| k == "l5" || extra.contains(&k.as_str())) {
        Ok(())
    } else {
        Err(SocialError::Schema)
    }
}

/// elementi di una lista di seq F1: `[]` oppure `[d(,d)*]` con interi
/// decimali senza zero in testa; altro → Schema (pattern L4.7).
fn seq_items(s: &str) -> Result<Vec<u64>, SocialError> {
    if s == "[]" {
        return Ok(Vec::new());
    }
    let inner = s
        .strip_prefix('[')
        .and_then(|x| x.strip_suffix(']'))
        .ok_or(SocialError::Schema)?;
    let mut out: Vec<u64> = Vec::new();
    for part in inner.split(',') {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(SocialError::Schema);
        }
        if part.len() > 1 && part.starts_with('0') {
            return Err(SocialError::Schema);
        }
        out.push(part.parse().map_err(|_| SocialError::Schema)?);
    }
    Ok(out)
}

/// bibliografia canonica (L5.6/L5.7): array F1, DEDUP CRESCENTE — violazione
/// → Schema.
fn canonical_sources(s: &str) -> Result<Vec<u64>, SocialError> {
    let items = seq_items(s)?;
    if items.windows(2).any(|w| w[0] >= w[1]) {
        return Err(SocialError::Schema);
    }
    Ok(items)
}

fn sort_dedup(mut v: Vec<u64>) -> Vec<u64> {
    v.sort_unstable();
    v.dedup();
    v
}

fn list_str(v: &[u64]) -> String {
    let mut out = String::from("[");
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        {
            use std::fmt::Write as _;
            let _ = write!(out, "{x}");
        }
    }
    out.push(']');
    out
}

// ---------------------------------------------------------------------------
// parser canonico di `osservazioni` (L5.2)
// ---------------------------------------------------------------------------

/// Stringa JSON canonica RIGIDA: la ri-serializzazione canonica del
/// decodificato deve coincidere byte per byte col token letto.
fn parse_json_string(s: &str, i: usize) -> Result<(String, usize), SocialError> {
    let b = s.as_bytes();
    if b.get(i) != Some(&b'"') {
        return Err(SocialError::Schema);
    }
    let mut j = i + 1;
    while j < b.len() {
        if b[j] == b'"' {
            let token = &s[i..j + 1];
            let decoded = decode_json_string(token)?;
            // canone: il token deve essere la serializzazione canonica del decodificato
            let mut canon = String::new();
            write_escaped(&mut canon, &decoded);
            if canon != token {
                return Err(SocialError::Schema);
            }
            return Ok((decoded, j + 1));
        }
        if b[j] == b'\\' {
            j += 2; // salta la coppia d'escape; il canone del decodificato la verifica
        } else {
            j += 1;
        }
    }
    Err(SocialError::Schema)
}

fn decode_json_string(token: &str) -> Result<String, SocialError> {
    let mut out = String::new();
    let mut it = token[1..token.len() - 1].chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next().ok_or(SocialError::Schema)? {
            '"' => out.push('"'),
            '\\' => out.push('\\'),
            'b' => out.push('\u{8}'),
            't' => out.push('\t'),
            'n' => out.push('\n'),
            'f' => out.push('\u{c}'),
            'r' => out.push('\r'),
            'u' => {
                let mut hex = String::new();
                for _ in 0..4 {
                    hex.push(it.next().ok_or(SocialError::Schema)?);
                }
                let n = u32::from_str_radix(&hex, 16).map_err(|_| SocialError::Schema)?;
                out.push(char::from_u32(n).ok_or(SocialError::Schema)?);
            }
            _ => return Err(SocialError::Schema),
        }
    }
    Ok(out)
}

/// decimale canonico senza segno: `0` o `d(,d)*` senza zero in testa.
fn parse_u64_canonical(s: &str) -> Result<u64, SocialError> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(SocialError::Schema);
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(SocialError::Schema);
    }
    s.parse().map_err(|_| SocialError::Schema)
}

/// Membri delle `osservazioni` (L5.2): array F1 di oggetti canonici
/// `{"altro":"<label>","comportamento":<i64>}` in ordine lessicografico.
/// Array vuoto ammesso; doppi membri ammessi e contano ognuno; forma
/// malformata (JSON non canonico, chiavi extra, shape errata, label vuota,
/// comportamento negativo) → Schema.
fn parse_osservazioni(s: &str) -> Result<Vec<(String, i64)>, SocialError> {
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'[') {
        return Err(SocialError::Schema);
    }
    if s == "[]" {
        return Ok(Vec::new());
    }
    let mut i = 1;
    let mut out: Vec<(String, i64)> = Vec::new();
    loop {
        // membro: {"altro":"<label>","comportamento":<n>}
        if bytes.get(i) != Some(&b'{') {
            return Err(SocialError::Schema);
        }
        i += 1;
        if bytes.get(i) != Some(&b'"') {
            return Err(SocialError::Schema);
        }
        let (key, next) = parse_json_string(s, i)?;
        if key != "altro" {
            return Err(SocialError::Schema);
        }
        i = next;
        if bytes.get(i) != Some(&b':') {
            return Err(SocialError::Schema);
        }
        i += 1;
        let (label, next) = parse_json_string(s, i)?;
        i = next;
        if bytes.get(i) != Some(&b',') {
            return Err(SocialError::Schema);
        }
        i += 1;
        let (key, next) = parse_json_string(s, i)?;
        if key != "comportamento" {
            return Err(SocialError::Schema);
        }
        i = next;
        if bytes.get(i) != Some(&b':') {
            return Err(SocialError::Schema);
        }
        i += 1;
        let (num_txt, next) = {
            // intero: solo cifre (negativo → Schema, L5.2)
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j == i {
                return Err(SocialError::Schema);
            }
            (s[i..j].to_string(), j)
        };
        i = next;
        if bytes.get(i) != Some(&b'}') {
            return Err(SocialError::Schema);
        }
        i += 1;
        if label.is_empty() {
            return Err(SocialError::Schema);
        }
        let n = parse_u64_canonical(&num_txt)?;
        out.push((label, n as i64));
        // separatore o chiusura array
        match bytes.get(i) {
            Some(&b',') => i += 1,
            Some(&b']') => {
                i += 1;
                break;
            }
            _ => return Err(SocialError::Schema),
        }
    }
    if i != bytes.len() {
        return Err(SocialError::Schema);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// tipi pubblici (L5.10)
// ---------------------------------------------------------------------------

/// Modello statistico di UN altro (L5.5): vista derivata pura a sola
/// lettura; contatori per classe in ordine di classe crescente (BTreeMap),
/// incrementi saturati.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OtherModel {
    pub id: String,
    pub n: u64,
    pub ultimo: Option<i64>,
    pub contatori: BTreeMap<i64, u64>,
}

/// Vista sociale a fine log (L5.5): `head_hash` = hash dell'ultimo evento
/// del log; `models` in ordine di id lessicografico.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialView {
    pub head_hash: String,
    pub models: BTreeMap<String, OtherModel>,
}

/// Previsione (L5.6): classe argmax del modello + bibliografia delle seq
/// osservate nel prefisso (dedup crescente).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prediction {
    pub id: String,
    pub previsto: i64,
    pub sources: Vec<u64>,
}

/// Errori L5 (L5.10, pattern L4.8): `Schema` = payload fuori schema/dominio
/// (incluso l'interno delle `osservazioni`); `HeadMatch` = base incoerente;
/// `NoModel` = previsione su altro con zero osservazioni nel prefisso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialError {
    Schema,
    HeadMatch,
    NoModel,
}

/// Base con coerenza di prefisso (L5.10, stile L2.22/L4.8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SocialBase {
    pub head_hash: String,
    pub log_len: usize,
}

/// Decisione L5 (L5.7): UNA `ModelUpdate` per finestra predict→osservazione
/// con disaccordo; `at` = seq dell'osservazione, `tick` = tick dell'evento
/// osservazione (L2.18); `sources` = la coppia di seq citate dal trigger
/// `[p, o]` (dedup crescente). Append al chiamante via API L0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision5 {
    ModelUpdate {
        at: u64,
        tick: u64,
        id: String,
        previsto: i64,
        osservato: i64,
        sources: Vec<u64>,
    },
}

// ---------------------------------------------------------------------------
// accumulatori di replay
// ---------------------------------------------------------------------------

struct Accum {
    n: u64,
    ultimo: Option<i64>,
    contatori: BTreeMap<i64, u64>,
    seqs: Vec<u64>,
}

impl Accum {
    fn new() -> Self {
        Accum { n: 0, ultimo: None, contatori: BTreeMap::new(), seqs: Vec::new() }
    }

    fn osserva(&mut self, seq: u64, comportamento: i64) {
        // incrementi SATURATI (L5.5, stile L2)
        self.n = self.n.saturating_add(1);
        let c = self.contatori.entry(comportamento).or_insert(0);
        *c = c.saturating_add(1);
        self.ultimo = Some(comportamento);
        self.seqs.push(seq);
    }
}

/// Replay della statistica (L5.5): scorre gli eventi `log` fino a
/// `up_to` (esclusivo) validando TUTTE le forme del proprio dominio L5
/// (L5.1/L5.2/L5.6/L5.7: gli errori di forma sono Schema a prescindere
/// dalla rilevanza statistica). Restituisce gli accumulatori per id; la
/// semantica delle finestre (ancore dei predict) è applicata in
/// `evaluate_social` (L5.7) sul cammino in ordine di seq.
struct Replay {
    models: BTreeMap<String, Accum>,
}

fn replay_upto(log: &Log, up_to: usize) -> Result<Replay, SocialError> {
    let mut models: BTreeMap<String, Accum> = BTreeMap::new();

    for e in log.events().iter().take(up_to) {
        // [L5.2] la chiave `osservazioni` vive SOLO nel canale percettivo:
        // su un evento NON-SENSE è scritta fuori dal proprio dominio → Schema
        // (specchio della regola di patch L4).
        if e.payload.get("osservazioni").is_some() && e.r#type != EventType::Sense {
            return Err(SocialError::Schema);
        }
        // [L5.2] `osservazioni` è SOLO stringa con array F1 canonico:
        // chiave presente con valore NON-stringa → Schema (patch R1,
        // rilievo audit_l5_impl; specchi selection.rs:47, identity.rs:283).
        if let Some(value) = e.payload.get("osservazioni") {
            let raw = match value {
                Value::Str(raw) => raw.as_str(),
                // [patch R1 / L5.2] forma malformata: NON-stringa → Schema
                _ => return Err(SocialError::Schema),
            };
            let members = parse_osservazioni(raw)?;
            if !members.is_empty() {
                // il gruppo della stessa SENSE concorre in ordine di membro;
                // l'ultimo membro dell'ultima SENSE resta `ultimo` (L5.5)
                for (label, comportamento) in members {
                    let id = id_altro(&label);
                    let acc = models.entry(id).or_insert_with(Accum::new);
                    acc.osserva(e.seq, comportamento);
                }
            }
        }

        // [L5.1] `l5` è SOLO un ACTION, dominio chiuso, mutualmente esclusivo
        // con `l2`/`l3`/`l4`; eventi senza `l5` sono fuori da L5.
        if e.payload.get("l5").is_some() {
            if e.r#type != EventType::Action {
                return Err(SocialError::Schema);
            }
            for (x, _) in e.payload.iter() {
                if x == "l2" || x == "l3" || x == "l4" {
                    return Err(SocialError::Schema);
                }
            }
            let tag = str_field(e, "l5")?;
            match tag {
                "predict" => {
                    exact_keys(e, &["at", "id", "previsto", "sources"])?;
                    let id = str_field(e, "id")?;
                    if !hex64(id) {
                        return Err(SocialError::Schema);
                    }
                    let at = nonneg_field(e, "at")?;
                    if at as usize >= up_to {
                        return Err(SocialError::Schema); // `at` che non riferisce seq esistente
                    }
                    match e.payload.get("previsto") {
                        Some(Value::Int(_)) => {}
                        _ => return Err(SocialError::Schema),
                    }
                    let sources = canonical_sources(str_field(e, "sources")?)?;
                    let _ = sources; // forma canonica solo: contenuto autorevole (L2.11)
                }
                "model_update" => {
                    exact_keys(e, &["at", "id", "previsto", "osservato", "sources"])?;
                    let id = str_field(e, "id")?;
                    if !hex64(id) {
                        return Err(SocialError::Schema);
                    }
                    let at = nonneg_field(e, "at")?;
                    if at as usize >= up_to {
                        return Err(SocialError::Schema); // `at` che non riferisce seq esistente
                    }
                    match e.payload.get("previsto") {
                        Some(Value::Int(_)) => {}
                        _ => return Err(SocialError::Schema),
                    }
                    match e.payload.get("osservato") {
                        Some(Value::Int(_)) => {}
                        _ => return Err(SocialError::Schema),
                    }
                    // [L5.7] `sources` = della coppia citata dal trigger: canonica [p, o]
                    let sources = canonical_sources(str_field(e, "sources")?)?;
                    if sources.len() != 2 {
                        return Err(SocialError::Schema);
                    }
                    let _ = sources;
                }
                _ => return Err(SocialError::Schema), // fuori dominio chiuso L5.1
            }
        }
    }
    Ok(Replay { models })
}

/// campo `previsto` del payload predict (i64 qualsiasi: contenuto
/// autorevole; la FORMA è Int).
fn previsto_value(e: &Event) -> Result<i64, SocialError> {
    match e.payload.get("previsto") {
        Some(Value::Int(n)) => Ok(*n),
        _ => Err(SocialError::Schema),
    }
}

/// id canonico dell'altro (L5.3): SHA-256 minuscolo del canone
/// `{"altro":"<label>"}` — scope SEPARATO dai nodi L1; una label non crea
/// mai nodi L1. NON parte dell'elenco pubblico L5.10 (pattern L4: `node_id`
/// resta interno); i test calcolano l'id in modo indipendente.
fn id_altro(label: &str) -> String {
    let mut canone = String::from("{\"altro\":");
    write_escaped(&mut canone, label);
    canone.push('}');
    sha(canone.as_bytes())
}

// ---------------------------------------------------------------------------
// API (L5.10)
// ---------------------------------------------------------------------------

/// replay a fine log (L5.5) → vista sociale.
pub fn build_social(log: &Log) -> Result<SocialView, SocialError> {
    let replay = replay_upto(log, log.events().len())?;
    let mut models = BTreeMap::new();
    for (id, acc) in replay.models {
        models.insert(
            id.clone(),
            OtherModel { id, n: acc.n, ultimo: acc.ultimo, contatori: acc.contatori },
        );
    }
    let head_hash = log
        .events()
        .last()
        .map(|e| e.hash.clone())
        .ok_or(SocialError::Schema)?;
    Ok(SocialView { head_hash, models })
}

/// coerenza di prefisso (L5.10/L2.22/L4.8): il log deve avere almeno
/// `log_len` eventi e l'hash dell'evento in posizione `log_len−1` deve
/// coincidere con `head_hash`.
fn check_base(log: &Log, base: &SocialBase) -> Result<(), SocialError> {
    if log.events().len() < base.log_len
        || base.log_len == 0
        || log.events()[base.log_len - 1].hash != base.head_hash
    {
        return Err(SocialError::HeadMatch);
    }
    Ok(())
}

/// previsione (L5.6): funzione pura del PREFISSO della base. Il modello è
/// costruito SOLO dagli eventi fino a `base.log_len` (la previsione deve
/// restare autorevole in replay qualunque sia la coda del log); `NoModel`
/// se l'id ha zero osservazioni nel prefisso.
pub fn predict_social(log: &Log, base: &SocialBase, id: &str) -> Result<Prediction, SocialError> {
    check_base(log, base)?;
    if !hex64(id) {
        return Err(SocialError::Schema);
    }
    let replay = replay_upto(log, base.log_len)?;
    let acc = replay.models.get(id).ok_or(SocialError::NoModel)?;
    let previsto = argmax(&acc.contatori);
    Ok(Prediction { id: id.to_string(), previsto, sources: sort_dedup(acc.seqs.clone()) })
}

/// classe argmax dei contatori; parità → la classe MINORE vince (L5.6,
/// tie-break firmato: attraversamento in ordine di classe crescente con
/// confronto stretto).
fn argmax(contatori: &BTreeMap<i64, u64>) -> i64 {
    let mut best: Option<(i64, u64)> = None;
    for (classe, conteggio) in contatori.iter() {
        match best {
            None => best = Some((*classe, *conteggio)),
            Some((_, c)) if *conteggio > c => best = Some((*classe, *conteggio)),
            _ => {}
        }
    }
    best.map(|(k, _)| k).unwrap_or(0)
}

/// decisioni L5 (L5.4/L5.7): coerenza di prefisso; cutoff = max(seq
/// dell'ultimo evento `l5` del log, seq dell'evento in posizione
/// `base.log_len−1`); UNA decisione per finestra, in ordine di seq,
/// SOLO per finestre con osservazione `o` > cutoff. In replay gli eventi
/// apposti sono AUTOREVOLI (L2.11): il ricalcolo divergente è mismatch
/// dichiarato, non errore runtime.
pub fn evaluate_social(log: &Log, base: &SocialBase) -> Result<Vec<Decision5>, SocialError> {
    check_base(log, base)?;
    // validazione di forma dell'INTERO log (gli errori sono Schema a
    // prescindere dalla rilevanza delle decisioni, pattern L4.7)
    replay_upto(log, log.events().len())?;
    let last_l5 = log
        .events()
        .iter()
        .filter(|e| e.payload.get("l5").is_some())
        .map(|e| e.seq)
        .max()
        .unwrap_or(0);
    let cutoff = last_l5.max(log.events()[base.log_len - 1].seq);

    // [L5.7] le finestre sono definite dal predict più recente per id:
    // un secondo predict senza osservazioni in mezzo SOSTITUISCE il primo;
    // il cammino qui sotto chiude ogni finestra con la prima osservazione
    // successiva dello stesso id.
    let mut anchors: BTreeMap<String, (u64, i64)> = BTreeMap::new();
    let mut out: Vec<Decision5> = Vec::new();
    for e in log.events() {
        if e.r#type == EventType::Action {
            if let Some(Value::Str(tag)) = e.payload.get("l5") {
                if tag == "predict" {
                    // anchor aggiornato: il predict più recente sostituisce
                    let id = str_field(e, "id")?;
                    let previsto = previsto_value(e)?;
                    anchors.insert(id.to_string(), (e.seq, previsto));
                    continue;
                }
            }
        }
        // [L5.2 patch R1] `osservazioni`: stessa validazione di replay_upto —
        // chiave presente con valore NON-stringa F1 → Schema (rilievo R1,
        // audit_l5_impl; specchi selection.rs:47, identity.rs:283).
        if let Some(value) = e.payload.get("osservazioni") {
            let raw = match value {
                Value::Str(raw) => raw.as_str(),
                _ => return Err(SocialError::Schema),
            };
            let members = parse_osservazioni(raw)?;
            for (label, comportamento) in members {
                let id = id_altro(&label);
                let (p, previsto) = match anchors.remove(&id) {
                    Some((p, prev)) => (p, prev),
                    None => continue, // nessuna finestra aperta per questo id
                };
                // osservazione di chiusura: UNA decisione per finestra, SOLO post-cutoff
                if e.seq > cutoff && previsto != comportamento {
                    out.push(Decision5::ModelUpdate {
                        at: e.seq,
                        tick: e.tick,
                        id: id.clone(),
                        previsto,
                        osservato: comportamento,
                        sources: sort_dedup(vec![p, e.seq]),
                    });
                }
            }
        }
    }
    Ok(out)
}

/// encoding payload L0 di una decisione L5 (L5.7/L5.10): chiavi esatte
/// `l5, at, id, previsto, osservato, sources` (chiuso), `sources` = la
/// coppia citata dal trigger in dedup crescente. Append al chiamante.
pub fn encode5(d: &Decision5) -> Vec<(String, Value)> {
    match d {
        Decision5::ModelUpdate { at, id, previsto, osservato, sources, .. } => vec![
            ("l5".into(), Value::Str("model_update".into())),
            ("at".into(), Value::Int(*at as i64)),
            ("id".into(), Value::Str(id.clone())),
            ("previsto".into(), Value::Int(*previsto)),
            ("osservato".into(), Value::Int(*osservato)),
            ("sources".into(), Value::Str(list_str(&sort_dedup(sources.clone())))),
        ],
    }
}

/// encoding payload L0 di un predict (L5.6/L5.10): chiavi esatte
/// `l5, at, id, previsto, sources` (chiuso), `sources` dedup crescente.
/// `at` = seq dell'ANCORA (ultima osservazione usata, anchoring L2.18):
/// passato dal chiamante. Append al chiamante via API L0.
pub fn encode5_predict(id: &str, at: u64, previsto: i64, sources: &[u64]) -> Vec<(String, Value)> {
    vec![
        ("l5".into(), Value::Str("predict".into())),
        ("at".into(), Value::Int(at as i64)),
        ("id".into(), Value::Str(id.to_string())),
        ("previsto".into(), Value::Int(previsto)),
        ("sources".into(), Value::Str(list_str(&sort_dedup(sources.to_vec())))),
    ]
}
