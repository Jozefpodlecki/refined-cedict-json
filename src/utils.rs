use crate::CERecord;
use crate::PinyinMap;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;
use std::io::{prelude::*, BufReader};

pub fn remove_duplicates(file_path: &str) -> std::io::Result<()> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut pinyins: HashSet<PinyinMap> = HashSet::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let parts: Vec<&str> = line.split(" ").collect();

        let pinyin = PinyinMap {
            pinyin: parts[0].to_string(),
            wade_giles: parts[1].to_string(),
        };
        pinyins.insert(pinyin);
    }

    let file = File::create(file_path).unwrap();
    let mut file = LineWriter::new(file);

    for pinyin in pinyins {
        let line = format!("{} {}", pinyin.pinyin, pinyin.wade_giles);
        file.write_all(line.as_bytes())?;
    }

    file.flush()
}

pub fn parse_ce_record(line: &str) -> CERecord {
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

    CERecord {
        simplified: simplified,
        traditional: traditional,
        wade_giles_pinyin: wade_giles_pinyin,
        meanings: meanings,
    }
}
