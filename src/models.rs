use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Meaning {
    pub context: String,
    pub lexical_item: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Classifier {
    pub value: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Pronunciation {
    pub pinyin: String,
    pub wade_giles_pinyin: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Detail {
    pub pronunciation: Vec<Pronunciation>,
    pub simplified: String,
    pub simplified_stroke_count: u8,
    pub traditional: String,
    pub traditional_stroke_count: u8,
    pub meanings: Vec<Meaning>,
    pub classifiers: Vec<Classifier>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub simplified: String,
    pub simplified_stroke_count: u8,
    pub details: Vec<Detail>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CERecord {
    pub line_number: u32,
    pub line: String,
    pub simplified: String,
    pub traditional: String,
    pub wade_giles_pinyin: String,
    pub meanings: Vec<String>,
}

#[derive(Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Category {
    pub simplified: String,
    pub pinyin: String,
    pub meaning: String,
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.simplified == other.simplified
            && self.pinyin == other.pinyin
            && self.meaning == other.meaning
    }
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
