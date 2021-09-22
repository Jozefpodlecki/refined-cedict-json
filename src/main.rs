mod customReader;
mod models;
mod utils;
use crate::customReader::customReader::BufReader;
use crate::models::*;
use crate::utils::get_pinyins;
use crate::utils::parse_ce_record;
use crate::utils::remove_duplicates;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;
use std::path::PathBuf;

fn get_stroke_order_map(file_path: String) -> Result<HashMap<String, u8>, Box<dyn Error>> {
    let mut lines = BufReader::open(file_path)?;
    let mut dict: HashMap<String, u8> = HashMap::new();

    for line in lines {
        let mut parts = line?.split(" ");
        let simplified = parts.next().unwrap().to_string();
        let stroke_count = parts.next().unwrap().parse::<u8>()?;
        dict.insert(simplified, stroke_count);
    }

    Ok(dict)
}

fn try_get_ce_dict_records(
    file_path: String,
    cache_path: PathBuf,
) -> Result<Vec<CERecord>, Box<dyn Error>> {
    let lines = BufReader::open(file_path)?;
    let mut list: Vec<CERecord> = Vec::new();

    if cache_path.exists() {
        let file = File::open(file_path)?;
        list = serde_json::from_reader(file)?;
        return Ok(list);
    }

    for line in lines {
        let line = line?;

        if line.starts_with("#") {
            continue;
        }

        let record = parse_ce_record(&line);
        let key = record.simplified.clone();
        list.push(record.clone());
    }

    Ok(list)
}

fn get_lines_from_file(file_path: String) -> Result<HashSet<String>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashSet<String> = HashSet::new();

    for line in reader {
        let line = line?;
        list.insert(line.to_string());
    }

    Ok(list)
}

fn main() -> std::io::Result<()> {
    let mut refined_list: Vec<EnhancedRecord> = Vec::new();
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

    let chem_elem = File::create(current_directory.join("verbs.txt"))?;
    let mut chem_elem = LineWriter::new(chem_elem);

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

    for item in hashset {
        println!("{}", item);
    }

    return Ok({});

    for (key, value) in dict {
        let record = EnhancedRecord {
            simplified: key,
            stroke_count: Some(0),
            details: Vec::new(),
        };

        refined_list.push(record);
    }

    let json = serde_json::to_string(&list).unwrap();
    fs::write("list.json", json).unwrap();

    //let json = serde_json::to_string(&dict).unwrap();
    //fs::write("grouped_by_simplified.json", json).unwrap();
}
