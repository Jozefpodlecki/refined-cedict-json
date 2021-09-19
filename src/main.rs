mod models;
mod utils;
use crate::models::*;
use crate::utils::parse_ce_record;
use crate::utils::remove_duplicates;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};

fn main() {
    let mut list: Vec<CERecord> = Vec::new();
    let mut dict: HashMap<String, Vec<CERecord>> = HashMap::new();

    let current_directory = env::current_dir().unwrap();
    println!("The current directory is {}", current_directory.display());

    let assets_path = current_directory.join("assets");
    let file_name = "cedict_ts.u8";
    let file_path = assets_path.join(file_name);

    remove_duplicates(assets_path.join("pinyin-map.txt").to_str().unwrap());

    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();

        if line.starts_with("#") {
            continue;
        }

        let record = parse_ce_record(&line);
        let key = record.simplified.clone();
        list.push(record.clone());
        dict.entry(key).or_insert(Vec::new()).push(record);
    }

    // let json = serde_json::to_string(&list).unwrap();
    // fs::write("simple.json", json).unwrap();

    let json = serde_json::to_string(&dict).unwrap();
    fs::write("grouped_by_simplified.json", json).unwrap();
}
