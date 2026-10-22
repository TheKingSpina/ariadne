//! Payload dell'evento — JSON canonico (SPEC L0 v0.2 §2).
//!
//! Decisioni di formato (da governance, progetto `opencode:Ariadne`):
//! - chiavi in ordine alfabetico di byte (BTreeMap, lexicografico = ordine UTF-8);
//! - serializzazione compatta, nessun spazio;
//! - valori ammessi: SOLO interi `i64` e stringhe UTF-8 — niente float, bool, null,
//!   array, object annidati (rifiutati a costruzione e in parser);
//! - escape JSON standard: `"` `\` `\b` `\t` `\n` `\f` `\r`; altri caratteri di controllo
//!   < 0x20 come `\uxxxx` con quattro cifre esadecimali minuscole; nulla di più viene
//!   escapato UTF-8 diretto.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Valore di payload: solo intero `i64` o stringa UTF-8.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Str(v.to_string())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Str(v)
    }
}

/// Payload dell'evento: insieme ordinato alfabeticamente di chiavi → valori canonici.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Payload {
    map: BTreeMap<String, Value>,
}

impl Payload {
    pub fn empty() -> Self {
        Payload {
            map: BTreeMap::new(),
        }
    }

    /// Costruisce un payload da coppie (chiave, valore) in qualunque ordine;
    /// l'ordine alfabetico è imposto dalla struttura, non dall'input.
    pub fn from_pairs<I, K>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, Value)>,
        K: Into<String>,
    {
        let mut payload = Payload::empty();
        for (key, value) in pairs {
            payload.map.insert(key.into(), value);
        }
        payload
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        self.map.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// JSON canonico: chiavi ordinate alfabeticamente, compatto, solo interi e stringhe.
    pub fn canonical(&self) -> String {
        let mut out = String::with_capacity(self.map.len() * 16 + 2);
        out.push('{');
        let mut first = true;
        for (key, value) in &self.map {
            if !first {
                out.push(',');
            }
            first = false;
            write_escaped(&mut out, key);
            out.push(':');
            match value {
                Value::Int(v) => {
                    let _ = write!(out, "{v}");
                }
                Value::Str(s) => write_escaped(&mut out, s),
            }
        }
        out.push('}');
        out
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.map.iter()
    }
}

/// Scrive una stringa JSON con escape standard e UTF-8 diretto.
pub(crate) fn write_escaped(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{9}' => out.push_str("\\t"),
            '\u{a}' => out.push_str("\\n"),
            '\u{c}' => out.push_str("\\f"),
            '\u{d}' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
