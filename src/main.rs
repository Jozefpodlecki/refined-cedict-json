mod customReader;
mod models;
mod utils;
mod api;
use crate::api::get_stroke_count;
use crate::customReader::customReader::BufReader;
use crate::models::*;
use crate::utils::get_pinyins;
use crate::utils::is_cjk;
use crate::utils::parse_ce_record;
use crate::utils::remove_duplicates;
use scraper::Html;
use scraper::Selector;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;
use std::path::PathBuf;

fn enhance_record(record: CERecord) -> Result<(), Box<dyn Error>> {
    let record = EnhancedRecord {
        simplified: record.simplified,
        stroke_count: Some(0),
        details: Vec::new(),
    };

    Ok(())
}

fn get_stroke_order_map(file_path: PathBuf) -> Result<HashMap<String, u8>, Box<dyn Error>> {
    let lines = BufReader::open(file_path)?;
    let mut dict: HashMap<String, u8> = HashMap::new();

    for line in lines {
        let line = line?;

        let mut parts = line.split(" ");
        let simplified = parts.next().unwrap().to_string();
        let stroke_count = parts.next().unwrap().parse::<u8>()?;
        dict.insert(simplified, stroke_count);
    }

    Ok(dict)
}

fn try_get_ce_dict_records(
    file_path: PathBuf,
    cache_path: PathBuf,
) -> Result<Vec<CERecord>, Box<dyn Error>> {
    let lines = BufReader::open(file_path)?;
    let mut list: Vec<CERecord> = Vec::new();

    if cache_path.exists() {
        let mut bytes = Vec::new();
        File::open(cache_path)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        list = serde_json::from_slice(&bytes).unwrap();
        return Ok(list);
    }

    let mut index = 1;
    for line in lines {
        let line = line?;

        if line.starts_with("#") {
            continue;
        }

        let record = parse_ce_record(&line, index);
        list.push(record);
        index = index + 1;
    }

    let json = serde_json::to_string_pretty(&list)?;
    fs::write(cache_path, json)?;

    Ok(list)
}

fn get_lines_from_file(file_path: PathBuf) -> Result<HashSet<String>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashSet<String> = HashSet::new();

    for line in reader {
        let line = line?;
        list.insert(line.to_string());
    }

    Ok(list)
}

fn update_stroke_order(file_path: PathBuf, output_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::open(file_path.clone())?;
    let mut results: HashMap<String, u8> = HashMap::new();

    for line in reader {
        let line = line?;
        
        let parts: Vec<&str> = line.split(" ").collect();
        let key = parts[0];
        let stroke_count = get_stroke_count(key)?;

        results.insert(key.to_string(), stroke_count);
    }

    
    let file = File::create(output_path)?;
    let mut line_writer = LineWriter::new(file);

    for (character, stroke_count) in results {
        line_writer.write_all(format!("{} {}\n", character, stroke_count).as_bytes())?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {

    let mut refined_list: Vec<EnhancedRecord> = Vec::new();
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

    update_stroke_order(current_directory.join("stroke-count.txt"), current_directory.join("stroke-count1.txt"))?;

    let mut hashset: HashSet<String> = HashSet::new();
    
    let stroke_order_map: HashMap<String, u8> =
        get_stroke_order_map(assets_path.join("stroke-order.txt"))?;

    let mut list = try_get_ce_dict_records(file_path, current_directory.join("cache.json"))?;
    let mut single_characters: HashSet<char> = HashSet::new();

    for record in list {
        let mut new_record = EnhancedRecord {
            simplified: record.simplified.clone(),
            stroke_count: None,
            details: Vec::new(),
        };

        let key = &record.simplified.clone();

        if key.chars().count() == 1 {
            let character = key.chars().next().unwrap();

            if is_cjk(&character) {
                single_characters.insert(character);
            }
        }

        let breakdown = record.wade_giles_pinyin.split(" ").map(|s| s.trim());

        for item in breakdown {
            if !pinyins_map.contains_key(item) {
                hashset.insert(item.to_string().to_lowercase());
            }
        }

        match stroke_order_map.get(key) {
            Some(stroke_count) => {
                new_record.stroke_count = Some(*stroke_count);
            }
            None => {}
        }
    }

    for item in hashset {
        println!("{}", item);
    }

    // if current_directory.join("list.json").exists() {
    //     list = serde_json::from_reader(file).unwrap();
    // } else {
    //     let reader = BufReader::new(file);

    //     for line in reader.lines() {
    //         let line = line.unwrap();

    //         if line.starts_with("#") {
    //             continue;
    //         }
    //         let record = parse_ce_record(&line);
    //         let key = record.simplified.clone();
    //         list.push(record.clone());

    //         let breakdown = record.wade_giles_pinyin.split(" ");

    //         if line.contains("/to ") {
    //             chem_elem.write_all(format!("{}\n", record.simplified).as_bytes())?;
    //             continue;
    //         }

    //         for item in breakdown {
    //             if !pinyins_map.contains_key(item) {
    //                 hashset.insert(item.to_string().to_lowercase());
    //             }
    //         }
    //         dict.entry(key).or_insert(Vec::new()).push(record);
    //     }
    // }

    // let json = serde_json::to_string(&list).unwrap();
    // fs::write("list.json", json).unwrap();

    //let json = serde_json::to_string(&dict).unwrap();
    //fs::write("grouped_by_simplified.json", json).unwrap();
    Ok(())
}
