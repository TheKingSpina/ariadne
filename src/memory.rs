//! L3 test-first API stubs. No L3 behavior is implemented yet.
use crate::{Log, Value};

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

// TODO L3: stub test-only
pub fn build_memory(_log: &Log) -> Result<MemoryView,L3Error> { Err(L3Error::Config) }
// TODO L3: stub test-only
pub fn evaluate(_log:&Log,_base:&MemoryView)->Result<Vec<Decision3>,L3Error>{Ok(vec![])}
// TODO L3: stub test-only
pub fn encode(_d:&Decision3)->Vec<(String,Value)>{vec![]}
// TODO L3: stub test-only
pub fn encode_recall(_at:u64,_origin_seq:u64,_id:String)->Vec<(String,Value)>{vec![]}
// TODO L3: stub test-only
pub fn canonical(_view:&MemoryView)->String{String::new()}
