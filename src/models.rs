use hashbrown::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Meaning {
    value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Detail {
    pinyin: String,
    meanings: Vec<Meaning>,
    tags: String,
}

#[derive(Serialize, Deserialize)]
pub struct Record {
    simplified: String,
    traditional: String,
    details: Vec<Detail>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CERecord {
    pub simplified: String,
    pub traditional: String,
    pub wade_giles_pinyin: String,
    pub meanings: Vec<String>,
}

#[derive(Hash, Serialize, Deserialize)]
pub struct PinyinMap {
    pub pinyin: String,
    pub wade_giles: String,
}

impl PartialEq for PinyinMap {
    fn eq(&self, other: &Self) -> bool {
        self.pinyin == other.pinyin && self.wade_giles == other.wade_giles
    }
}
impl Eq for PinyinMap {}
