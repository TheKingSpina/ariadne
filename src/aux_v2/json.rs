//! Small canonical JSON codec for the closed v2 schemas. No floats or null.
use std::collections::BTreeMap;
use super::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Json {
    Int(i64),
    Str(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub fn object<const N: usize>(pairs: [(&str, Json); N]) -> Self {
        Self::Object(pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }
    pub fn canonical(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Str(s) => { let mut out = String::new(); crate::payload::write_escaped(&mut out, s); out },
            Self::Array(a) => format!("[{}]", a.iter().map(Self::canonical).collect::<Vec<_>>().join(",")),
            Self::Object(o) => format!("{{{}}}", o.iter().map(|(k,v)| format!("{}:{}", Self::Str(k.clone()).canonical(), v.canonical())).collect::<Vec<_>>().join(",")),
        }
    }
    pub fn get(&self, key: &str) -> Result<&Json, Error> {
        match self { Self::Object(o) => o.get(key).ok_or(Error::Schema), _ => Err(Error::Schema) }
    }
    pub fn keys(&self, expected: &[&str]) -> Result<(), Error> {
        match self { Self::Object(o) if o.len() == expected.len() && expected.iter().all(|k| o.contains_key(*k)) => Ok(()), _ => Err(Error::Schema) }
    }
    pub fn string(&self) -> Result<&str, Error> { match self { Self::Str(s) => Ok(s), _ => Err(Error::Schema) } }
    pub fn int(&self) -> Result<i64, Error> { match self { Self::Int(n) => Ok(*n), _ => Err(Error::Schema) } }
    pub fn array(&self) -> Result<&[Json], Error> { match self { Self::Array(a) => Ok(a), _ => Err(Error::Schema) } }
    pub fn parse(text: &str) -> Result<Json, Error> {
        let value = Self::decode(text)?;
        if value.canonical() != text { return Err(Error::Schema); }
        Ok(value)
    }
    pub(super) fn decode(text: &str) -> Result<Json, Error> {
        let mut p = Parser { text, pos: 0 };
        let value = p.value(0)?;
        if p.pos != text.len() { return Err(Error::Schema); }
        Ok(value)
    }
}
impl From<&str> for Json { fn from(s: &str) -> Self { Self::Str(s.to_owned()) } }
impl From<String> for Json { fn from(s: String) -> Self { Self::Str(s) } }
impl From<i64> for Json { fn from(n: i64) -> Self { Self::Int(n) } }

struct Parser<'a> { text: &'a str, pos: usize }
impl Parser<'_> {
    fn byte(&self) -> Option<u8> { self.text.as_bytes().get(self.pos).copied() }
    fn take(&mut self, b: u8) -> Result<(), Error> {
        if self.byte() != Some(b) { return Err(Error::Schema); } self.pos += 1; Ok(())
    }
    fn string(&mut self) -> Result<String, Error> {
        self.take(b'"')?;
        let mut out = String::new();
        loop {
            match self.byte().ok_or(Error::Schema)? {
                b'"' => { self.pos += 1; return Ok(out); },
                b'\\' => {
                    self.pos += 1;
                    let b = self.byte().ok_or(Error::Schema)?; self.pos += 1;
                    out.push(match b {
                        b'"' => '"', b'\\' => '\\', b'b' => '\u{8}', b'f' => '\u{c}',
                        b'n' => '\n', b'r' => '\r', b't' => '\t',
                        b'u' => {
                            let end = self.pos.checked_add(4).ok_or(Error::Schema)?;
                            let h = self.text.get(self.pos..end).ok_or(Error::Schema)?;
                            let code = u32::from_str_radix(h, 16).map_err(|_| Error::Schema)?;
                            self.pos = end; char::from_u32(code).ok_or(Error::Schema)?
                        },
                        _ => return Err(Error::Schema),
                    });
                },
                0..=31 => return Err(Error::Schema),
                _ => { let c = self.text[self.pos..].chars().next().ok_or(Error::Schema)?; self.pos += c.len_utf8(); out.push(c); },
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<Json, Error> {
        if depth > 32 { return Err(Error::Schema); }
        match self.byte().ok_or(Error::Schema)? {
            b'"' => Ok(Json::Str(self.string()?)),
            b'[' => {
                self.pos += 1; let mut a = Vec::new();
                if self.byte() != Some(b']') { loop {
                    a.push(self.value(depth + 1)?);
                    if self.byte() != Some(b',') { break; } self.pos += 1;
                } }
                self.take(b']')?; Ok(Json::Array(a))
            },
            b'{' => {
                self.pos += 1; let mut o = BTreeMap::new();
                if self.byte() != Some(b'}') { loop {
                    let key = self.string()?; self.take(b':')?;
                    if o.insert(key, self.value(depth + 1)?).is_some() { return Err(Error::Schema); }
                    if self.byte() != Some(b',') { break; } self.pos += 1;
                } }
                self.take(b'}')?; Ok(Json::Object(o))
            },
            b'-' | b'0'..=b'9' => {
                let start = self.pos; self.pos += 1;
                while matches!(self.byte(), Some(b'0'..=b'9')) { self.pos += 1; }
                Ok(Json::Int(self.text[start..self.pos].parse().map_err(|_| Error::Schema)?))
            },
            _ => Err(Error::Schema),
        }
    }
}
