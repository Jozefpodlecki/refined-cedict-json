mod api;
mod customReader;
mod models;
mod utils;
use crate::api::get_stroke_count_from_wiktionary;
use crate::customReader::customReader::BufReader;
use crate::models::*;
use crate::utils::get_pinyins;
use crate::utils::is_cjk;
use crate::utils::parse_ce_record;
use crate::utils::remove_duplicates;
use api::get_info_from_writtenchinese;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::Html;
use scraper::Selector;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::LineWriter;
use std::path::PathBuf;

fn get_stroke_order_map(file_path: PathBuf) -> Result<HashMap<String, u8>, Box<dyn Error>> {
    let lines = BufReader::open(file_path)?;
    let mut dict: HashMap<String, u8> = HashMap::new();

    for line in lines {
        let line = line?;

        let mut parts = line.split(" ");
        let simplified = parts.next().unwrap().trim().to_string();
        let stroke_count = parts.next().unwrap().trim().parse::<u8>()?;
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
        let file_size = fs::metadata(&cache_path)?.len();
        let mut bytes = Vec::with_capacity(usize::try_from(file_size)?);
        File::open(cache_path)?.read_to_end(&mut bytes)?;
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

fn get_from_file(file_path: PathBuf) -> Result<HashSet<Category>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashSet<Category> = HashSet::new();

    for line in reader {
        let line = line?;

        let mut parts = line.split(",").map(|pr| pr.trim().to_string());

        let record = Category {
            simplified: parts.next().unwrap(),
            pinyin: parts.next().unwrap(),
            meaning: parts.next().unwrap(),
        };

        list.insert(record);
    }

    Ok(list)
}

fn get_lines_from_file(file_path: PathBuf) -> Result<HashSet<String>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashSet<String> = HashSet::new();

    for line in reader {
        let line = line?;
        list.insert(line.trim().to_string());
    }

    Ok(list)
}

fn update_pinyins(pinyin_path: &str, output_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let pinyins = get_pinyins(pinyin_path);
    let mut pinyins_map: HashMap<String, PinyinMap> = HashMap::new();
    let mut updated: HashMap<String, String> = HashMap::new();

    for pinyinIt in pinyins {
        let cloned = pinyinIt.clone();
        let PinyinMap { pinyin, wade_giles } = pinyinIt;

        pinyins_map.insert(wade_giles.clone(), cloned);

        if wade_giles == pinyin {
            let default = pinyin.to_string();
            println!("{} {}", wade_giles, pinyin);
            let pinyin_from_api = get_info_from_writtenchinese(&wade_giles)?.unwrap_or(default);
            updated
                .entry(wade_giles.to_owned())
                .or_insert(pinyin_from_api.to_string());
        } else {
            updated.entry(wade_giles.to_owned()).or_insert(pinyin);
        }
    }

    let file = File::create(output_path)?;
    let mut line_writer = LineWriter::new(file);

    for (wade_giles, pinyin) in updated {
        let temp = format!("{} {}\n", pinyin, wade_giles);
        let line = temp.as_bytes();
        line_writer.write_all(line)?;
    }

    Ok(())
}

fn update_stroke_order(file_path: PathBuf, output_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::open(file_path.clone())?;
    let mut results: HashMap<String, u8> = HashMap::new();

    {
        let reader = BufReader::open(output_path.clone())?;
        for line in reader {
            let line = line?;
            let mut parts = line.split(" ").map(|s| s.trim());
            let character = parts.next().unwrap().to_owned();
            let stroke_count = parts.next().unwrap().parse::<u8>().unwrap();
            results.insert(character, stroke_count);
        }
    }

    let file = File::create(output_path)?;
    let mut line_writer = LineWriter::new(file);

    for line in reader {
        let line = line?;

        let mut parts = line.split(" ").map(|s| s.trim());
        let key = parts.next().unwrap();

        if results.contains_key(key) {
            println!("Skipping: {}", key);
            continue;
        }

        let stroke_count = get_stroke_count_from_wiktionary(key)?;

        if stroke_count.is_none() {
            println!("Could not process character: {}", line);
            continue;
        }

        //results.insert(key.to_string(), stroke_count);
        let line = format!("{} {}\n", key, stroke_count.unwrap());
        println!("{}", line);
        line_writer.write_all(line.as_bytes())?;
    }

    // for (character, stroke_count) in results {

    // }

    Ok(())
}

fn write_lines_to_file(file_path: PathBuf, items: HashSet<char>) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut line_writer = LineWriter::new(file);

    for character in items {
        let line = format!("{}\n", character);
        line_writer.write_all(line.as_bytes())?;
    }

    Ok(())
}

fn to_pinyin(wade_giles_pinyin: &str, pinyin_map: &HashMap<String, PinyinMap>) -> String {
    let breakdown = wade_giles_pinyin.split(" ").map(|s| s.trim());
    let mut result = "".to_string();

    for item in breakdown {
        let normalized = item.to_string().to_lowercase();
        let chars: Vec<char> = normalized.chars().collect();
        let first_char = chars.first().unwrap();

        if chars.len() == 1 && first_char.is_alphabetic() {
            result = result + " " + &first_char.to_string();
            continue;
        }

        if normalized == "·" || normalized == "," {
            result = result + " " + &normalized;
            continue;
        }

        let pinyin = pinyin_map.get(&normalized).unwrap();
        result = result + " " + &pinyin.pinyin;
    }

    result
}

fn get_group_ce_records_by_simplified(
    records: &[CERecord],
    cache_dict_path: PathBuf,
) -> Result<HashMap<String, Vec<CERecord>>, Box<dyn Error>> {
    let mut dict: HashMap<String, Vec<CERecord>> = HashMap::new();

    if cache_dict_path.exists() {
        let file_size = fs::metadata(&cache_dict_path)?.len();
        let mut bytes = Vec::with_capacity(usize::try_from(file_size)?);
        File::open(cache_dict_path)?.read_to_end(&mut bytes)?;

        dict = serde_json::from_slice(&bytes)?;
        return Ok(dict);
    }

    for record in records {
        let key = record.simplified.to_string();
        dict.entry(key).or_insert(Vec::new()).push(record.clone());
    }

    let file = File::create(cache_dict_path)?;
    serde_json::to_writer(file, &dict)?;

    Ok(dict)
}

fn get_single_characters(records: &[CERecord]) -> HashSet<char> {
    let mut single_characters: HashSet<char> = HashSet::new();

    for record in records {
        let mut key = &record.simplified;
        let mut chars: Vec<char> = key.chars().collect();

        if chars.len() == 1 {
            let character = chars[0];

            if is_cjk(&character) {
                single_characters.insert(character);
            }
        }

        key = &record.traditional;
        chars = key.chars().collect();

        if chars.len() == 1 {
            let character = chars[0];

            if is_cjk(&character) {
                single_characters.insert(character);
            }
        }

        //let pinyin = to_pinyin(&record.wade_giles_pinyin, &pinyins_map);
    }

    single_characters
}

fn main() -> Result<(), Box<dyn Error>> {
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
        let wade_giles = pinyin.wade_giles.clone();
        pinyins_map.insert(wade_giles.clone(), pinyin);
    }

    // update_stroke_order(
    //     current_directory.join("stroke-count.txt"),
    //     current_directory.join("stroke-count1.txt"),
    // )?;

    //update_pinyins(pinyin_path, assets_path.join("pinyin-updated.txt"))?;

    let stroke_order_map = get_stroke_order_map(assets_path.join("stroke-order.txt"))?;
    let list = try_get_ce_dict_records(file_path, current_directory.join("cache.json"))?;
    let dict =
        get_group_ce_records_by_simplified(&list, current_directory.join("cache-dict.json"))?;

    let dates_and_days_of_week = get_from_file(assets_path.join("dates-and-days-of-week.txt"))?;
    let chemical_elements = get_lines_from_file(assets_path.join("chemical-elements.txt"))?;
    let adjectives = get_lines_from_file(assets_path.join("adjectives.txt"))?;
    let adverbs = get_lines_from_file(assets_path.join("adverbs.txt"))?;
    let verbs = get_lines_from_file(assets_path.join("verbs.txt"))?;

    lazy_static! {
        static ref LITERATURE_REGEX: Regex =
            Regex::new(r"\/\(?lit\.\)?\s(.*)\s(\(idiom\))?",).unwrap();
        static ref FIGURATIVELY_REGEX: Regex = Regex::new(r"\/\(fig\.\)\s(.*)\/?",).unwrap();
        static ref COLLOQUIAL_REGEX: Regex = Regex::new(r"\/\(coll\.\)\s(.*)\/?",).unwrap();
        static ref EXTRACT_PINYIN_REGEX: Regex = Regex::new(r"[(?P<pinyin>.*?)\]",).unwrap();
    }

    for (key, records) in dict {
        let mut new_record = Group {
            simplified: key.clone(),
            simplified_stroke_count: stroke_order_map.get(&key).unwrap_or(&0).to_owned(),
            details: Vec::new(),
        };

        for record in records {
            let pinyin = to_pinyin(&record.wade_giles_pinyin, &pinyins_map);

            let mut item = Detail {
                meanings: Vec::new(),
                pronunciation: Vec::new(),
                simplified: record.simplified.clone(),
                simplified_stroke_count: new_record.simplified_stroke_count.clone(),
                traditional_stroke_count: stroke_order_map
                    .get(&record.traditional)
                    .unwrap_or(&0)
                    .to_owned(),
                tags: Vec::new(),
                classifiers: Vec::new(),
                traditional: record.traditional,
            };

            item.pronunciation.push(Pronunciation {
                pinyin: pinyin,
                wade_giles_pinyin: record.wade_giles_pinyin,
            });

            if adverbs.contains(&key) {}

            if adverbs.contains(&key) {}

            if verbs.contains(&key) {}

            let mut meanings = item.meanings;

            for meaning in record.meanings {
                let mut meaning_record = Meaning {
                    context: "".to_string(),
                    lexical_item: "".to_string(),
                    value: meaning.to_string(),
                };

                if meaning.contains("(honorific)") {}
                if meaning.contains("(dialect)") {}
                if meaning.contains("(polite)") {}

                if meaning.contains("Japanese") {}
                if meaning.contains("(Tw)") {
                    meaning_record.context = "taiwanese".to_string();
                }
                if meaning.contains("also pr.") {
                    let captures = EXTRACT_PINYIN_REGEX.captures(&meaning).unwrap();

                    let wade_giles_pinyin = captures.name("pinyin").unwrap().as_str().to_string();
                    let pinyin = to_pinyin(&wade_giles_pinyin, &pinyins_map);

                    item.pronunciation.push(Pronunciation {
                        pinyin: pinyin,
                        wade_giles_pinyin: wade_giles_pinyin,
                    });
                }

                if meaning.contains("(coll.) ") {
                    let captures = COLLOQUIAL_REGEX.captures(&meaning).unwrap();
                    let mut temp = captures.iter();
                    temp.next();
                    let text = temp.next().unwrap().unwrap().as_str();

                    meaning_record.lexical_item = "colloquial".to_string();
                    meaning_record.value = text.to_owned();
                }

                if meaning.contains("lit.") {
                    let captures = LITERATURE_REGEX.captures(&meaning).unwrap();
                    let mut temp = captures.iter();
                    temp.next();
                    let text = temp.next().unwrap().unwrap().as_str();

                    meaning_record.context = "literature".to_string();
                    meaning_record.lexical_item = "literature".to_string();
                    meaning_record.value = "literature".to_string();
                }

                if meaning.contains("(fig.)") {
                    let captures = FIGURATIVELY_REGEX.captures(&meaning).unwrap();
                    let mut temp = captures.iter();
                    temp.next();
                    let text = temp.next().unwrap().unwrap().as_str();

                    meaning_record.lexical_item = "figuratively".to_string();
                }
                if meaning.contains("(idiom)") {
                    meaning_record.lexical_item = "idiom".to_string();
                }
                if meaning.contains("see also") {}

                if meaning.contains("variant") {
                    meaning_record.lexical_item = "variant".to_string();
                }
                if meaning.contains("abbr.") {
                    meaning_record.lexical_item = "abbreviation".to_string();
                }
                if meaning.contains("CL") {
                    let classifier = Classifier {
                        description: "".to_string(),
                        value: "".to_string(),
                    };

                    item.classifiers.push(classifier);
                }

                meanings.push(meaning_record.clone());
            }

            item.meanings = meanings;

            new_record.details.push(item);
        }
    }

    Ok(())
}
