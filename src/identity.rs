//! L4 — replay deterministico del sé (legge `notes/L4_legge_identita.md`,
//! commit `2ec39a0`).
//!
//! - `build_self(log)` = replay autorevole a fine log (L4.7): gli eventi
//!   `l4` in log sono AUTOREVOLI (il testo apposto alimenta lo stato com'è
//!   scritto, L2.11); le coppie narrate+snapshot sono appaiate in modo
//!   stretto e ogni `snapshot.self_hash` è verificato duramente contro il
//!   documento cumulativo con `head_hash` = hash del narrate (L4.4).
//! - `evaluate_self(log, base)` = coerenza di prefisso (L4.8/L2.22) e
//!   decisioni L4.6 SOLO per eventi rilevanti oltre il cutoff (L4.3: prime
//!   dichiarazioni nodes/edges via vista L1, eventi `l2` select/summary,
//!   `l3` promote/archive/recall); per ogni revisione una coppia Narrate
//!   POI Snapshot, con apposizione simulata in coda (l'hash del narrate a
//!   decision-time esiste solo simulando l'apposizione secondo L4.9
//!   «sempre la coppia narrate+snapshot in ordine, per ogni revisione,
//!   senza intermissioni»).
//! - Le affermazioni leggono SOLO i campi necessari dagli eventi del log
//!   (`ups`/`downs`/`expulsions`/`id`/`origin_seq`), nessun re-parse per
//!   gli oggetti interni l2/l3, nessuna chiamata alle API L2/L3 (L4.2).
//! - Documento canonico L4.4: chiavi lessicografiche, `schema_version:"1"`,
//!   `self_hash` = SHA-256 minuscolo del preimage privo del campo.
//! - Zero I/O, zero wall-clock, zero RNG in questo modulo (L4.5/L4.10).

use sha2::{Digest, Sha256};

use crate::{Event, EventType, Log, Payload, Value};
use crate::payload::write_escaped;
use crate::view;

// ---------------------------------------------------------------------------
// API pubblica (L4.8)
// ---------------------------------------------------------------------------

/// Affermazione della revisione nel documento L4.4: `at` = seq dell'evento
/// rilevante generatore; `sources` = bibliografia (seq citate); `text` =
/// affermazioni della revisione congiunte con `\n` (L4.1/L4.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub at: u64,
    pub sources: Vec<u64>,
    pub text: String,
}

/// Documento del sé (L4.4): `head_hash` = hash dell'ultimo evento del log
/// al momento della ricostruzione; `statements` in ordine di `at`
/// crescente; `self_hash` = SHA-256 minuscolo del preimage privo del campo
/// `self_hash` (pattern D16 L1 / L3.8).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelfDocument {
    pub head_hash: String,
    pub statements: Vec<Statement>,
    pub self_hash: String,
}

/// Errori L4 (L4.8): `Schema` = payload fuori schema/dominio, incluso
/// l'interno degli eventi letti (pattern L2.21); `HeadMatch` = testa non
/// coincidente (L4.2/L4.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfError {
    Schema,
    HeadMatch,
}

/// Base con coerenza di prefisso (L4.8, stile L2.22): il log deve avere
/// almeno `log_len` eventi e l'hash dell'evento in posizione `log_len−1`
/// deve coincidere con `head_hash`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelfBase {
    pub head_hash: String,
    pub log_len: usize,
}

/// Decisioni L4 (L4.6): per ogni evento rilevante oltre il cutoff, in
/// ordine di seq, UNA coppia — prima Narrate, poi Snapshot — con `at` =
/// seq dell'evento rilevante e `tick` = tick dell'evento apposto (pattern
/// L2.18). Append al chiamante via API L0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision4 {
    Narrate { at: u64, tick: u64, text: String, sources: Vec<u64> },
    Snapshot { at: u64, tick: u64, self_hash: String },
}

// ---------------------------------------------------------------------------
// utility deterministiche
// ---------------------------------------------------------------------------

fn sha(bytes: &[u8]) -> String {
    let dig = Sha256::digest(bytes);
    dig.iter().map(|b| format!("{b:02x}")).collect()
}

fn fmt_u64(out: &mut String, v: u64) {
    use std::fmt::Write as _;
    let _ = write!(out, "{v}");
}

/// campo i64 non negativo del payload → u64; mancante/non-i64/negativo →
/// Schema.
fn u64_field(e: &Event, key: &str) -> Result<u64, SelfError> {
    match e.payload.get(key) {
        Some(Value::Int(n)) => u64::try_from(*n).map_err(|_| SelfError::Schema),
        _ => Err(SelfError::Schema),
    }
}

/// campo stringa del payload; mancante/non-stringa → Schema.
fn str_field<'a>(e: &'a Event, key: &str) -> Result<&'a str, SelfError> {
    match e.payload.get(key) {
        Some(Value::Str(s)) => Ok(s),
        _ => Err(SelfError::Schema),
    }
}

/// schema chiuso (pattern L2.21/L3.1): chiavi del payload ⊆ {tag} ∪ extra.
fn exact_keys(e: &Event, tag: &str, extra: &[&str]) -> Result<(), SelfError> {
    if e.payload
        .iter()
        .all(|(k, _)| k == tag || extra.contains(&k.as_str()))
    {
        Ok(())
    } else {
        Err(SelfError::Schema)
    }
}

/// hash hex minuscolo a 64 caratteri.
fn hex64(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// elementi di una lista di seq in forma JSON canonica di base: `[]` oppure
/// `[d(,d)*]` con interi decimali senza zero in testa; altro → Schema
/// (L4.7: «lista non canonica — Schema»).
fn seq_items(s: &str) -> Result<Vec<u64>, SelfError> {
    if s == "[]" {
        return Ok(Vec::new());
    }
    let inner = s
        .strip_prefix('[')
        .and_then(|x| x.strip_suffix(']'))
        .ok_or(SelfError::Schema)?;
    let mut out: Vec<u64> = Vec::new();
    for part in inner.split(',') {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(SelfError::Schema);
        }
        if part.len() > 1 && part.starts_with('0') {
            return Err(SelfError::Schema);
        }
        out.push(part.parse().map_err(|_| SelfError::Schema)?);
    }
    Ok(out)
}

/// bibliografia del payload `narrate` (L4.1): stringa JSON canonica array
/// di i64, DEDUP CRESCENTE — vincolo forte sul payload → Schema se non
/// conforme.
fn canonical_sources(s: &str) -> Result<Vec<u64>, SelfError> {
    let items = seq_items(s)?;
    if items.windows(2).any(|w| w[0] >= w[1]) {
        return Err(SelfError::Schema);
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
        fmt_u64(&mut out, *x);
    }
    out.push(']');
    out
}

// ---------------------------------------------------------------------------
// documento canonico (L4.4)
// ---------------------------------------------------------------------------

/// array `statements` canonico: ordine di `at` crescente (stabile), membro
/// `{"at":N,"sources":[…],"text":"…"}` con chiavi figlie lessicografiche,
/// nessuno spazio, escape canonico.
fn statements_array(statements: &[Statement]) -> String {
    let mut sorted: Vec<&Statement> = statements.iter().collect();
    sorted.sort_by_key(|s| s.at);
    let mut out = String::new();
    out.push('[');
    for (i, st) in sorted.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"at\":");
        fmt_u64(&mut out, st.at);
        out.push_str(",\"sources\":");
        out.push_str(&list_str(&st.sources));
        out.push_str(",\"text\":");
        write_escaped(&mut out, &st.text);
        out.push('}');
    }
    out.push(']');
    out
}

/// preimage del `self_hash` (L4.4/D16/L3.8): documento canonico privo del
/// campo `self_hash`.
fn doc_preimage(head_hash: &str, statements: &[Statement]) -> String {
    let mut out = String::new();
    out.push_str("{\"head_hash\":");
    write_escaped(&mut out, head_hash);
    out.push_str(",\"schema_version\":\"1\",\"statements\":");
    out.push_str(&statements_array(statements));
    out.push('}');
    out
}

fn doc_digest(head_hash: &str, statements: &[Statement]) -> String {
    sha(doc_preimage(head_hash, statements).as_bytes())
}

// ---------------------------------------------------------------------------
// valori di payload l2/l3 letti a campi limitati (L4.2/L4.3)
// ---------------------------------------------------------------------------

/// validazione a campi limitati di un evento `l2`/`l3` decisionale (il
/// parser legge SOLO i campi necessari: `ups`/`downs`/`expulsions`/`id`/
/// `origin_seq`; errore dentro un evento → Schema, L4.7/L2.21).
fn validate_decision_fields(e: &Event) -> Result<(), SelfError> {
    if e.payload.get("l2").is_some() && e.payload.get("l3").is_some() {
        // nessun dominio può accettare l'evento (L3.1 per l2+l3; L4.1 per
        // l4+l2/l3): errore interno
        return Err(SelfError::Schema);
    }
    if let Some(Value::Str(tag)) = e.payload.get("l2") {
        return match tag.as_str() {
            "select" => {
                seq_items(str_field(e, "ups")?)?;
                seq_items(str_field(e, "downs")?)?;
                Ok(())
            }
            "summary" => {
                seq_items(str_field(e, "expulsions")?)?;
                Ok(())
            }
            // barrier e config: controllo, estranei a L4 (L4.3(c))
            _ => Ok(()),
        };
    }
    if let Some(Value::Str(tag)) = e.payload.get("l3") {
        return match tag.as_str() {
            "promote" | "archive" | "recall" => {
                if !hex64(str_field(e, "id")?) {
                    return Err(SelfError::Schema);
                }
                u64_field(e, "origin_seq")?;
                Ok(())
            }
            // l3 config: controllo, escluso (L4.3(d))
            _ => Ok(()),
        };
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// rilevanza e affermazioni (L4.3)
// ---------------------------------------------------------------------------

/// discriminante `l4`: `None` se assente; valore fuori dominio → Schema.
fn l4_tag(e: &Event) -> Result<Option<&str>, SelfError> {
    match e.payload.get("l4") {
        None => Ok(None),
        Some(Value::Str(s)) => Ok(Some(s.as_str())),
        Some(_) => Err(SelfError::Schema),
    }
}

/// affermazioni L4.3(c)/(d) per un evento `l2`/`l3` decisionale: testo dei
/// template ESATTI e bibliografia = unione dedup-crescente delle seq
/// citate nel testo dell'affermazione (evento generatore compreso —
/// lettura dichiarata nel report: punto da firmare).
fn decision_statements(e: &Event) -> Result<(Vec<String>, Vec<u64>), SelfError> {
    let seq = e.seq;
    if let Some(Value::Str(tag)) = e.payload.get("l2") {
        return match tag.as_str() {
            "select" => {
                let ups = sort_dedup(seq_items(str_field(e, "ups")?)?);
                let downs = sort_dedup(seq_items(str_field(e, "downs")?)?);
                let mut cited = vec![seq];
                cited.extend_from_slice(&ups);
                cited.extend_from_slice(&downs);
                let text = format!(
                    "a seq {} ho selezionato: entrate [{}], uscite [{}]",
                    seq,
                    render(&ups),
                    render(&downs),
                );
                Ok((vec![text], sort_dedup(cited)))
            }
            "summary" => {
                let exp = sort_dedup(seq_items(str_field(e, "expulsions")?)?);
                let mut cited = vec![seq];
                cited.extend_from_slice(&exp);
                let text = format!("a seq {} ho consolidato: espulsi [{}]", seq, render(&exp));
                Ok((vec![text], sort_dedup(cited)))
            }
            _ => Ok((Vec::new(), Vec::new())),
        };
    }
    if let Some(Value::Str(tag)) = e.payload.get("l3") {
        return match tag.as_str() {
            "promote" | "archive" => {
                let id = str_field(e, "id")?;
                let origin = u64_field(e, "origin_seq")?;
                let _ = origin; // non citato nel template promote/archive
                let text = format!(
                    "a seq {} ho {} l'identità {}",
                    seq,
                    if tag == "promote" { "promosso" } else { "archiviato" },
                    id
                );
                Ok((vec![text], vec![seq]))
            }
            "recall" => {
                let id = str_field(e, "id")?;
                let origin = u64_field(e, "origin_seq")?;
                let text = format!(
                    "a seq {} ho richiamato l'identità {} (origin_seq {})",
                    seq, id, origin
                );
                Ok((vec![text], sort_dedup(vec![seq, origin])))
            }
            _ => Ok((Vec::new(), Vec::new())),
        };
    }
    Ok((Vec::new(), Vec::new()))
}

fn render(v: &[u64]) -> String {
    v.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
}

/// affermazioni L4.3(a)/(b) per un evento con estrazioni: SOLE prime
/// dichiarazioni (origin_seq nella vista == seq dell'evento, L1 D18);
/// ordine interno: nodi per `(kind,key)` poi archi per `(rel,src_id,
/// dst_id)` (nodi-primi D19); endpoint risolti a kind/key via id L1 della
/// vista (`id` e `origin_seq` da `view::build`, L4.3).
fn extraction_statements(seq: u64, v: &view::View) -> Result<(Vec<String>, Vec<u64>), SelfError> {
    let mut node_keys: Vec<(&String, &String)> = v
        .nodes
        .iter()
        .filter(|n| n.origin_seq == seq)
        .map(|n| (&n.kind, &n.key))
        .collect();
    node_keys.sort();
    let mut stmts: Vec<String> = Vec::new();
    for (kind, key) in &node_keys {
        stmts.push(format!(
            "a seq {} ho dichiarato l'elemento {} \"{}\"",
            seq, kind, key
        ));
    }
    let mut edge_keys: Vec<(&String, &String, &String)> = v
        .edges
        .iter()
        .filter(|x| x.origin_seq == seq)
        .map(|x| (&x.rel, &x.src_id, &x.dst_id))
        .collect();
    edge_keys.sort();
    for (rel, src_id, dst_id) in &edge_keys {
        let src = lookup_node(v, src_id)?;
        let dst = lookup_node(v, dst_id)?;
        stmts.push(format!(
            "a seq {} ho dichiarato l'arco {} da {} \"{}\" a {} \"{}\"",
            seq, rel, src.0, src.1, dst.0, dst.1
        ));
    }
    Ok((stmts, vec![seq]))
}

fn lookup_node<'a>(
    v: &'a view::View,
    id: &str,
) -> Result<(&'a String, &'a String), SelfError> {
    v.nodes
        .iter()
        .find(|n| n.id == id)
        .map(|n| (&n.kind, &n.key))
        .ok_or(SelfError::Schema)
}

/// affermazioni generabili dall'evento per la generazione L4.6 (L4.3):
/// evento senza domini propri e senza estrazioni → vuoto (non rilevante).
fn statements_for(e: &Event, v: &view::View) -> Result<(Vec<String>, Vec<u64>), SelfError> {
    if e.payload.get("l2").is_some() || e.payload.get("l3").is_some() {
        return decision_statements(e);
    }
    if e.payload.get("nodes").is_some() || e.payload.get("edges").is_some() {
        return extraction_statements(e.seq, v);
    }
    Ok((Vec::new(), Vec::new()))
}

// ---------------------------------------------------------------------------
// replay autorevole (L4.7)
// ---------------------------------------------------------------------------

/// iterazione a seq crescente su TUTTI gli eventi del log (L4.7):
/// - testa verificata con `view::build` (L4.2), errori mappati;
/// - `narrate`: autorevole, alimenta lo stato così com'è scritto; vincoli
///   su `at` (esistente, non-l4, storicamente anteriore) → Schema;
/// - `snapshot`: appaiato al narrate immediatamente precedente con lo
///   stesso `at`, `self_hash` verificato sul documento cumulativo (l'ancora
///   è dura) → Schema;
/// - narrate solitaria come ULTIMO evento del log: tollerata (nessun
///   errore, nessuna scrittura); qualunque evento successivo non appaiato
///   chiude una narrate pendente → Schema;
/// - errori interni a un evento (chiavi extra, domini, liste, campi) →
///   Schema;
/// - validazione a campi limitati degli eventi l2/l3 su TUTTO il log.
fn replay(log: &Log) -> Result<Vec<Statement>, SelfError> {
    let v = view::build(log).map_err(|_| SelfError::Schema)?;
    match log.events().last() {
        Some(e) if e.hash == v.head_hash => {}
        None => return Err(SelfError::Schema),
        Some(_) => return Err(SelfError::HeadMatch),
    }
    let mut statements: Vec<Statement> = Vec::new();
    // narrate pendente: (at, hash dell'evento narrate)
    let mut pending: Option<(u64, String)> = None;
    for e in log.events() {
        if let Some(tag) = l4_tag(e)? {
            // una scritta L4 è un evento ACTION (L4.1, pattern L3.1 secondo
            // capoverso)
            if e.r#type != EventType::Action {
                return Err(SelfError::Schema);
            }
            if e.payload.get("l2").is_some() || e.payload.get("l3").is_some() {
                // domini chiusi e mutualmente esclusivi (L4.1)
                return Err(SelfError::Schema);
            }
            match tag {
                "narrate" => {
                    if pending.is_some() {
                        // narrate chiusa da un evento non appaiato → Schema
                        return Err(SelfError::Schema);
                    }
                    exact_keys(e, "l4", &["at", "text", "sources"])?;
                    let at = u64_field(e, "at")?;
                    let text = str_field(e, "text")?.to_string();
                    let sources = canonical_sources(str_field(e, "sources")?)?;
                    if at >= e.seq {
                        // `at` deve riferirsi a un evento storicamente
                        // anteriore al narrate (L4.7)
                        return Err(SelfError::Schema);
                    }
                    let target = log.events().get(at as usize).ok_or(SelfError::Schema)?;
                    if target.seq != at || l4_tag(target)?.is_some() {
                        // evento inesistente o autoreferenziale → Schema
                        return Err(SelfError::Schema);
                    }
                    statements.push(Statement { at, sources, text });
                    pending = Some((at, e.hash.clone()));
                }
                "snapshot" => {
                    let (nat, nhash) = pending.ok_or(SelfError::Schema)?;
                    exact_keys(e, "l4", &["at", "self_hash"])?;
                    let at = u64_field(e, "at")?;
                    let self_hash = str_field(e, "self_hash")?;
                    if !hex64(self_hash) {
                        return Err(SelfError::Schema);
                    }
                    if at != nat {
                        // appaiamento stretto con lo stesso `at` (L4.7)
                        return Err(SelfError::Schema);
                    }
                    if doc_digest(&nhash, &statements) != self_hash {
                        // l'ancora d'integrità è dura (L4.7)
                        return Err(SelfError::Schema);
                    }
                    pending = None;
                }
                _ => return Err(SelfError::Schema), // valore fuori dominio
            }
            continue;
        }
        // evento non-l4:
        if pending.is_some() {
            // narrate solitaria chiusa da un eventi successivo non appaiato
            return Err(SelfError::Schema);
        }
        // validazione a campi limitati degli eventi l2/l3 (L4.7: su TUTTI
        // gli eventi; rilievo audit: vale per OGNI evento letto dal piano
        // L4, a prescindere dal tipo L0 — un payload decisionale su un
        // evento non-ACTION è scritta decisionale fuori dal proprio
        // dominio, L4.1/L4.2: Schema, uniforme con evaluate_self)
        validate_decision_fields(e)?;
        if (e.r#type != EventType::Action)
            && (e.payload.get("l2").is_some() || e.payload.get("l3").is_some())
        {
            return Err(SelfError::Schema);
        }
    }
    Ok(statements)
}

// ---------------------------------------------------------------------------
// API (L4.8)
// ---------------------------------------------------------------------------

/// replay autorevole a fine log (L4.7) → documento del sé (L4.4).
pub fn build_self(log: &Log) -> Result<SelfDocument, SelfError> {
    let mut statements = replay(log)?;
    statements.sort_by_key(|s| s.at);
    let head = log
        .events()
        .last()
        .map(|e| e.hash.clone())
        .ok_or(SelfError::Schema)?;
    let self_hash = doc_digest(&head, &statements);
    Ok(SelfDocument {
        head_hash: head,
        statements,
        self_hash,
    })
}

/// coerenza di prefisso (L4.8/L2.22) + decisioni L4.6 per eventi rilevanti
/// con seq > cutoff. Cutoff = max(seq dell'ultimo evento `l4` del log,
/// seq dell'evento in posizione `base.log_len−1`): nessuna decisione per
/// eventi nella base né per eventi già narrati.
pub fn evaluate_self(log: &Log, base: &SelfBase) -> Result<Vec<Decision4>, SelfError> {
    if log.events().len() < base.log_len
        || base.log_len == 0
        || log.events()[base.log_len - 1].hash != base.head_hash
    {
        return Err(SelfError::HeadMatch);
    }
    let cumulative: Vec<Statement> = replay(log)?;
    let cutoff = last_l4_seq(log).max(log.events()[base.log_len - 1].seq);
    let v = view::build(log).map_err(|_| SelfError::Schema)?;

    // apposizione SIMULATA in coda (L4.9): l'hash del narrate a
    // decision-time esiste solo simulando l'apposizione corretta — la firma
    // dell'evento simulato coincide con quella del chiamante che appende
    // esattamente le decisioni in ordine.
    let mut sim_seq = log.events().len() as u64;
    let mut sim_prev = log
        .events()
        .last()
        .map(|e| e.hash.clone())
        .ok_or(SelfError::Schema)?;

    let mut cumulative = cumulative;
    let mut out: Vec<Decision4> = Vec::new();
    for e in log.events() {
        if e.seq <= cutoff || l4_tag(e)?.is_some() {
            continue;
        }
        let (texts, cited) = statements_for(e, &v)?;
        if texts.is_empty() {
            continue; // evento non rilevante: nessuna decisione (L4.3)
        }
        let at = e.seq;
        let tick = e.tick;
        let sources = sort_dedup(cited);
        let text = texts.join("\n");
        let narrate = Decision4::Narrate {
            at,
            tick,
            text: text.clone(),
            sources: sources.clone(),
        };
        // simula l'apposizione del narrate: la sua firma è il `head_hash`
        // del documento cumulativo di questa revisione (L4.6)
        let payload = Payload::from_pairs(encode4(&narrate));
        let narrate_event =
            Event::seal(sim_seq, tick, EventType::Action, payload, &sim_prev);
        cumulative.push(Statement { at, sources, text });
        let self_hash = doc_digest(&narrate_event.hash, &cumulative);
        out.push(narrate);
        let snapshot = Decision4::Snapshot { at, tick, self_hash };
        // avanzamento della simulazione con lo snapshot apposto
        let payload = Payload::from_pairs(encode4(&snapshot));
        out.push(snapshot);
        let snapshot_event = Event::seal(
            sim_seq + 1,
            tick,
            EventType::Action,
            payload,
            &narrate_event.hash,
        );
        sim_prev = snapshot_event.hash.clone();
        sim_seq += 2;
    }
    Ok(out)
}

/// seq dell'ultimo evento `l4` del log (L4.6: narrate o snapshot, entrambi
/// decisionali; pattern L3.9). Nessun evento l4 → 0.
fn last_l4_seq(log: &Log) -> u64 {
    log.events()
        .iter()
        .filter(|e| e.payload.get("l4").is_some())
        .map(|e| e.seq)
        .max()
        .unwrap_or(0)
}

/// encoding payload L0 di una decisione L4 (L4.6): tuple nell'ordine della
/// legge (l4, at, text, sources) / (l4, at, self_hash); liste come
/// `Value::Str` con JSON canonico (bibliografia dedup crescente), `at` come
/// `Value::Int`. Append al chiamante via API L0.
pub fn encode4(d: &Decision4) -> Vec<(String, Value)> {
    match d {
        Decision4::Narrate { at, text, sources, .. } => vec![
            ("l4".into(), Value::Str("narrate".into())),
            ("at".into(), Value::Int(*at as i64)),
            ("text".into(), Value::Str(text.clone())),
            ("sources".into(), Value::Str(list_str(&sort_dedup(sources.clone())))),
        ],
        Decision4::Snapshot { at, self_hash, .. } => vec![
            ("l4".into(), Value::Str("snapshot".into())),
            ("at".into(), Value::Int(*at as i64)),
            ("self_hash".into(), Value::Str(self_hash.clone())),
        ],
    }
}
