use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Meaning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lexical_item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_meaning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simplified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traditional: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wade_giles_pinyin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinyin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Classifier {
    pub simplified: String,
    pub traditional: String,
    pub wade_giles_pinyin: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Abbreviation {
    pub key: String,
    pub value: Option<String>,
    pub simplified: Option<String>,
    pub traditional: Option<String>,
    pub wade_giles_pinyin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Pronunciation {
    pub pinyin: String,
    pub wade_giles_pinyin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Radical {
    pub value: String,
    pub stroke_count: u8,
    pub meaning: String,
    pub pinyin: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Decomposition {
    pub radical: Vec<Radical>,
    pub graphical: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Variant {
    pub simplified: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub traditional: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wade_giles_pinyin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Detail {
    pub pronunciation: Vec<Pronunciation>,

    pub simplified: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub simplified_stroke_count: Option<u8>,

    pub traditional: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub traditional_stroke_count: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<Variant>,

    pub meanings: Vec<Meaning>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub classifiers: Option<Vec<Classifier>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub decomposition: Option<Decomposition>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Group {
    pub simplified: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simplified_stroke_count: Option<u8>,
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

#[derive(Clone, Hash, Serialize, Deserialize)]
pub struct Descriptor {
    pub simplified: String,
    pub pinyin: String,
    pub meaning: String,
    pub lexical_item: Option<String>,
    pub tags: Option<Vec<String>>,
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
