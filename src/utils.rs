use crate::CERecord;
use crate::PinyinMap;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::LineWriter;
use std::io::{prelude::*, BufReader};
use scraper::Html;
use scraper::Selector;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use std::error::Error;

pub fn is_cjk(data: &char) -> bool {
    match *data {
        '\u{4E00}'..='\u{9FFF}' => true,
        '\u{3400}'..='\u{4DBF}' => true,
        '\u{20000}'..='\u{2A6DF}' => true,
        '\u{2A700}'..='\u{2B73F}' => true,
        '\u{2B740}'..='\u{2B81F}' => true,
        '\u{2B820}'..='\u{2CEAF}' => true,
        '\u{F900}'..='\u{FAFF}' => true,
        '\u{2F800}'..='\u{2FA1F}' => true,
        _ => false,
    }
}

pub fn get_pinyins(file_path: &str) -> BTreeSet<PinyinMap> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut pinyins: BTreeSet<PinyinMap> = BTreeSet::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let parts: Vec<&str> = line.split(" ").collect();

        let pinyin = PinyinMap {
            pinyin: parts[0].to_string(),
            wade_giles: parts[1].to_string(),
        };
        pinyins.insert(pinyin);
    }

    pinyins
}

pub fn remove_duplicates(file_path: &str) -> std::io::Result<()> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    //let mut pinyins: HashSet<PinyinMap> = HashSet::new();
    let mut pinyins: BTreeSet<PinyinMap> = BTreeSet::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let parts: Vec<&str> = line.split(" ").collect();

        let pinyin = PinyinMap {
            pinyin: parts[0].to_string(),
            wade_giles: parts[1].to_string(),
        };
        pinyins.insert(pinyin);
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .unwrap();
    let mut file = LineWriter::new(file);

    for pinyin in pinyins {
        let line = format!("{} {}\n", pinyin.pinyin, pinyin.wade_giles);
        file.write_all(line.as_bytes())?;
    }

    file.flush()
}

pub fn parse_ce_record(line: &str, line_number: u32) -> CERecord {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(
            r"(?P<traditional>.*?)\s(?P<simplified>.*?)\s\[(?P<pinyin>.*?)\]\s/(?P<meanings>.*)/",
        )
        .unwrap();
    }

    let captures = REGEX.captures(&line).unwrap();
    let traditional = captures.name("traditional").unwrap().as_str().to_string();
    let simplified = captures.name("simplified").unwrap().as_str().to_string();
    let wade_giles_pinyin = captures.name("pinyin").unwrap().as_str().to_string();
    let meanings = captures
        .name("meanings")
        .unwrap()
        .as_str()
        .split("/")
        .map(|s| s.to_string())
        .collect();

    let mut normalized_line = line.to_string();
    normalized_line.pop();
    normalized_line.pop();

    CERecord {
        line_number: line_number,
        line: normalized_line,
        simplified: simplified,
        traditional: traditional,
        wade_giles_pinyin: wade_giles_pinyin,
        meanings: meanings,
    }
}
