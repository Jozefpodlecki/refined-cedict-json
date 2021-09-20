use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Meaning {
    type_: String,
    value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Detail {
    pub pinyin: String,
    pub traditional: String,
    pub meanings: Vec<Meaning>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EnhancedRecord {
    pub simplified: String,
    pub stroke_count: Option<u8>,
    pub details: Vec<Detail>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CERecord {
    pub simplified: String,
    pub traditional: String,
    pub wade_giles_pinyin: String,
    pub meanings: Vec<String>,
}

#[derive(Clone, Hash, Serialize, Deserialize)]
pub struct PinyinMap {
    pub pinyin: String,
    pub wade_giles: String,
}

impl PartialEq for PinyinMap {
    fn eq(&self, other: &Self) -> bool {
        self.pinyin == other.pinyin && self.wade_giles == other.wade_giles
    }
}

impl Ord for PinyinMap {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pinyin.cmp(&other.pinyin)
    }
}

impl PartialOrd for PinyinMap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PinyinMap {}
