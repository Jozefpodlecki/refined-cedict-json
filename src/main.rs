mod api;
mod customReader;
mod models;
mod utils;
use crate::api::get_stroke_count_from_wiktionary;
use crate::customReader::customReader::BufReader;
use crate::models::*;
use crate::utils::get_pinyins;
use crate::utils::get_pinyins_map;
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
use std::path::Path;
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
    file_path: &Path,
    cache_path: &Path,
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
        let mut normalized = item.to_string().to_lowercase();
        let chars: Vec<char> = normalized.chars().collect();
        let first_char = chars.first().unwrap();

        if chars.len() == 1 && first_char.is_alphabetic() {
            normalized = first_char.to_string();

            if result.is_empty() {
                result = result + " " + &normalized;
            } else {
                result = result + &normalized;
            }

            continue;
        }

        if normalized == "·" || normalized == "," {
            if result.is_empty() {
                result = result + &normalized;
            } else {
                result = result + " " + &normalized;
            }
            continue;
        }

        match normalized.as_str() {
            "lu:4" => normalized = "lv4".to_owned(),
            "lu:3" => normalized = "lv3".to_owned(),
            "lu:2" => normalized = "lv2".to_owned(),
            "lu:e4" => normalized = "lve4".to_owned(),
            "nu:e4" => normalized = "nve4".to_owned(),
            _ => {}
        }

        let pinyin = pinyin_map.get(&normalized);

        if pinyin.is_none() {
            return normalized;
        }

        normalized = pinyin.unwrap().pinyin.to_owned();

        if result.is_empty() {
            result = result + &normalized;
        } else {
            result = result + " " + &normalized;
        }
    }

    result
}

fn get_group_ce_records_by_simplified(
    records: &[CERecord],
    cache_dict_path: &Path,
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

fn basic_refine_meaning(
    mut meaning_record: Meaning,
    pattern: &str,
    lexical_item: Option<String>,
    context: Option<String>,
) -> Meaning {
    let mut processed = str::replace(&meaning_record.value, pattern, "");
    processed = processed.trim().to_owned();
    meaning_record.lexical_item = lexical_item;
    meaning_record.context = context;
    meaning_record.value = processed.to_string();

    meaning_record
}

fn refine_records(
    records: HashMap<String, Vec<CERecord>>,
    current_directory: &Path,
    public_directory: &Path,
    assets_directory: &Path,
) -> Result<Vec<Group>, Box<dyn Error>> {
    let temp = assets_directory.join("pinyin-map.txt");
    let pinyin_path = temp.to_str().unwrap();
    let pinyins_map = get_pinyins_map(pinyin_path)?;

    let stroke_order_map = get_stroke_order_map(assets_directory.join("stroke-order.txt"))?;

    let dates_and_days_of_week =
        get_from_file(assets_directory.join("dates-and-days-of-week.txt"))?;
    let chemical_elements = get_lines_from_file(assets_directory.join("chemical-elements.txt"))?;
    let adjectives = get_lines_from_file(assets_directory.join("adjectives.txt"))?;
    let adverbs = get_lines_from_file(assets_directory.join("adverbs.txt"))?;
    let verbs = get_lines_from_file(assets_directory.join("verbs.txt"))?;

    lazy_static! {
        static ref EXTRACT_CLASSIFIER_REGEX: Regex =
            Regex::new(r"(.*?[^|])\|?(.*?)\[(.*?)\]").unwrap();
        static ref EXTRACT_PINYIN_REGEX: Regex = Regex::new(r"\[(?P<pinyin>.*?)\]",).unwrap();
    }

    let mut index = 1;
    let mut output: Vec<Group> = Vec::with_capacity(116725);
    let mut meaning_list: Vec<String> = Vec::with_capacity(500000);

    for (key, records) in records {
        println!("{}: Processing: {}", index, key);
        index = index + 1;

        let mut new_record = Group {
            simplified: key.clone(),
            simplified_stroke_count: stroke_order_map.get(&key).map(|pr| pr.to_owned()),
            details: Vec::new(),
        };

        for record in records {
            let pinyin = to_pinyin(&record.wade_giles_pinyin, &pinyins_map);

            let mut detail = Detail {
                meanings: Vec::new(),
                pronunciation: Vec::new(),
                simplified: record.simplified.clone(),
                simplified_stroke_count: new_record.simplified_stroke_count.clone(),
                traditional_stroke_count: stroke_order_map
                    .get(&record.traditional)
                    .map(|pr| pr.to_owned()),
                tags: Vec::new(),
                classifiers: Vec::new(),
                traditional: record.traditional,
            };

            detail.pronunciation.push(Pronunciation {
                pinyin: pinyin,
                wade_giles_pinyin: record.wade_giles_pinyin,
            });

            if dates_and_days_of_week.iter().any(|pr| pr.simplified == key) {
                detail.tags.push("Dates and days of week".to_owned());
            }

            if chemical_elements.contains(&key) {
                detail.tags.push("chemical elements".to_owned());
            }

            if adjectives.contains(&key) {
                detail.tags.push("adjective".to_owned());
            }

            if adverbs.contains(&key) {
                detail.tags.push("adverb".to_owned());
            }

            if verbs.contains(&key) {
                detail.tags.push("verb".to_owned());
            }

            let mut meanings = detail.meanings;

            for meaning in record.meanings {
                let mut meaning_record = Meaning {
                    context: None,
                    lexical_item: None,
                    simplified: None,
                    traditional: None,
                    wade_giles_pinyin: None,
                    pinyin: None,
                    value: meaning.to_string(),
                };

                if meaning.contains("(old)") {
                    meaning_record =
                        basic_refine_meaning(meaning_record, "(old)", None, Some("old".to_owned()));
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(loanword)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(loanword)",
                        None,
                        Some("loanword".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(honorific)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(honorific)",
                        None,
                        Some("honorific".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(dialect)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(dialect)",
                        None,
                        Some("dialect".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(polite)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(polite)",
                        None,
                        Some("polite".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("Japanese") {
                    meaning_record.context = Some("Japanese".to_string());
                    meaning_record.value = meaning.to_string();
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(Tw)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(Tw)",
                        None,
                        Some("taiwan".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("also pr.") {
                    let captures = EXTRACT_PINYIN_REGEX.captures(&meaning);

                    if captures.is_none() {
                        meaning_record.value = meaning.to_string();
                        meanings.push(meaning_record.clone());
                    } else {
                        let wade_giles_pinyin = captures
                            .unwrap()
                            .name("pinyin")
                            .unwrap()
                            .as_str()
                            .to_string();
                        let pinyin = to_pinyin(&wade_giles_pinyin, &pinyins_map);

                        detail.pronunciation.push(Pronunciation {
                            pinyin: pinyin,
                            wade_giles_pinyin: wade_giles_pinyin,
                        });
                    }
                    continue;
                }

                if meaning.contains("(coll.) ") {
                    let mut processed = str::replace(&meaning, "(coll.)", "");
                    processed = processed.trim().to_owned();

                    if meaning.contains("(Tw)") {
                        processed = str::replace(&processed, "(Tw)", "");
                        processed = processed.trim().to_owned();
                    }

                    meaning_record.context = Some("colloquial".to_string());
                    meaning_record.value = processed.to_string();
                    meanings.push(meaning_record.clone());
                    meaning_list.push(processed.to_string());
                    continue;
                }

                if meaning.contains("lit.") {
                    let mut pattern = "lit.";
                    let mut processed = meaning.to_string();

                    if meaning.contains("(lit.)") {
                        pattern = "(lit.)";
                    }

                    if meaning.contains("(lit. and fig.)") {
                        pattern = "(lit. and fig.)";
                    }

                    if meaning.contains("(idiom)") {
                        meaning_record.lexical_item = Some("idiom".to_string());
                        processed = str::replace(&meaning, "(idiom)", "");
                    }

                    processed = str::replace(&processed, pattern, "");
                    let result = processed.trim();

                    meaning_record.context = Some("literature".to_string());
                    meaning_record.value = result.to_string();
                    meanings.push(meaning_record.clone());
                    meaning_list.push(processed.to_string());
                    continue;
                }

                if meaning.contains("(fig.)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(fig.)",
                        None,
                        Some("figuratively".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(idiom)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(idiom)",
                        Some("idiom".to_owned()),
                        None,
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("see also") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "see also",
                        None,
                        Some("see also".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);

                    continue;
                }

                if meaning.contains("variant") {
                    let mut pattern = "variant of";

                    if meaning.contains("old variant") {
                        pattern = "old variant of";
                        meaning_record.context = Some("old variant".to_string());
                    } else {
                        meaning_record.context = Some("variant".to_string());
                    }

                    let mut processed = str::replace(&meaning, pattern, "");
                    processed = processed.trim().to_owned();

                    let captures = EXTRACT_CLASSIFIER_REGEX.captures(&processed);

                    if captures.is_none() {
                        meaning_record.value = processed.to_string();
                    } else {
                        let captures = captures.unwrap();

                        meaning_record.simplified =
                            Some(captures.get(1).unwrap().as_str().to_owned());
                        meaning_record.traditional =
                            Some(captures.get(2).unwrap().as_str().to_owned());
                        meaning_record.wade_giles_pinyin =
                            Some(captures.get(3).unwrap().as_str().to_owned());
                    }

                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("abbr.") {
                    let mut pattern = "abbr. for";

                    if meaning.contains("(abbr.)") {
                        pattern = "(abbr.)";
                    }

                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        pattern,
                        None,
                        Some("abbreviation".to_owned()),
                    );

                    let captures = EXTRACT_CLASSIFIER_REGEX.captures(&meaning_record.value);

                    if captures.is_none() {
                    } else {
                        let captures = captures.unwrap();

                        meaning_record.simplified =
                            Some(captures.get(1).unwrap().as_str().to_owned());
                        meaning_record.traditional =
                            Some(captures.get(2).unwrap().as_str().to_owned());
                        meaning_record.wade_giles_pinyin =
                            Some(captures.get(3).unwrap().as_str().to_owned());
                    }

                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("CL:") {
                    let mut processed = str::replace(&meaning, "CL:", "");
                    processed = processed.trim().to_owned();

                    for item in processed.split(",") {
                        let captures = EXTRACT_CLASSIFIER_REGEX.captures(&item).unwrap();

                        let classifier = Classifier {
                            simplified: captures.get(1).unwrap().as_str().to_owned(),
                            traditional: captures.get(2).unwrap().as_str().to_owned(),
                            wade_giles_pinyin: captures.get(3).unwrap().as_str().to_owned(),
                        };

                        detail.classifiers.push(classifier);
                    }

                    continue;
                }

                meanings.push(meaning_record);
            }

            detail.meanings = meanings;

            new_record.details.push(detail);
            output.push(new_record.clone());

            let file =
                File::create(public_directory.join(format!("{}.json", &new_record.simplified)))?;
            serde_json::to_writer_pretty(file, &new_record)?;
        }
    }

    let file = File::create(public_directory.join("extracted-meanings.json"))?;
    serde_json::to_writer(file, &meaning_list)?;

    Ok(output)
}

fn main() -> Result<(), Box<dyn Error>> {
    let current_directory = env::current_dir()?;
    println!("The current directory is {}", current_directory.display());

    let assets_directory = current_directory.join("assets");
    let public_directory = current_directory.join("public");

    let file_name = "cedict_ts.u8";
    let file_path = assets_directory.join(file_name);
    let list = try_get_ce_dict_records(&file_path, &current_directory.join("cache.json"))?;

    let grouped_records =
        get_group_ce_records_by_simplified(&list, &current_directory.join("cache-dict.json"))?;

    let result = refine_records(
        grouped_records,
        &current_directory,
        &public_directory,
        &assets_directory,
    )?;

    let file = File::create(public_directory.join("result.json"))?;
    serde_json::to_writer(file, &result)?;

    Ok(())
}
