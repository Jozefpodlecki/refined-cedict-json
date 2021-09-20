mod models;
mod utils;
use crate::models::*;
use crate::utils::get_pinyins;
use crate::utils::parse_ce_record;
use crate::utils::remove_duplicates;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() {
    let mut refined_list: Vec<Record> = Vec::new();
    let mut list: Vec<CERecord> = Vec::new();
    let mut dict: HashMap<String, Vec<CERecord>> = HashMap::new();

    let current_directory = env::current_dir().unwrap();
    println!("The current directory is {}", current_directory.display());

    let assets_path = current_directory.join("assets");
    let file_name = "cedict_ts.u8";
    let file_path = assets_path.join(file_name);

    let temp = assets_path.join("pinyin-map.txt");
    let pinyin_path = temp.to_str().unwrap();
    //remove_duplicates(pinyin_path).unwrap();
    let pinyins = get_pinyins(pinyin_path);
    let mut pinyins_map: HashMap<String, PinyinMap> = HashMap::new();

    for pinyin in pinyins {
        pinyins_map.insert(pinyin.wade_giles.clone(), pinyin);
    }

    let file = File::open(file_path.as_path()).unwrap();

    let mut hashset: HashSet<String> = HashSet::new();

    if current_directory.join("list.json").exists() {
        list = serde_json::from_reader(file).unwrap();
    } else {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();
            if line.starts_with("#") {
                continue;
            }
            let record = parse_ce_record(&line);
            let key = record.simplified.clone();
            list.push(record.clone());

            let breakdown = record.wade_giles_pinyin.split(" ");

            for item in breakdown {
                if !pinyins_map.contains_key(item) {
                    hashset.insert(item.to_string().to_lowercase());
                }
            }
            dict.entry(key).or_insert(Vec::new()).push(record);
        }
    }

    for (key, value) in dict {
        let record = EnhancedRecord {
            simplified: key,
            stroke_count: 0,
            details: Vec::new(),
        };

        refined_list.insert();
    }

    for item in hashset {
        println!("{}", item);
    }

    let json = serde_json::to_string(&list).unwrap();
    fs::write("list.json", json).unwrap();

    //let json = serde_json::to_string(&dict).unwrap();
    //fs::write("grouped_by_simplified.json", json).unwrap();
}
