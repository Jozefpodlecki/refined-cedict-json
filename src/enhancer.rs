use crate::api::download_cedict;
use crate::api::get_stroke_count_from_wiktionary;
use crate::customReader::customReader::BufReader;
use crate::models::*;
use crate::utils::get_descriptors_from_file;
use crate::utils::get_lines_from_file;
use crate::utils::get_pinyins;
use crate::utils::get_pinyins_map;
use crate::utils::get_row_from_file;
use crate::utils::get_single_characters;
use crate::utils::get_stroke_order_map;
use crate::utils::import_stroke_order;
use crate::utils::is_cjk;
use crate::utils::parse_ce_record;
use crate::utils::remove_duplicates;
use crate::utils::try_get_ce_dict_records;
use lazy_static::lazy_static;
use log::{debug, info};
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
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::LineWriter;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;

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

pub fn get_group_ce_records_by_simplified(
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

pub fn refine_records(
    records: HashMap<String, Vec<CERecord>>,
    current_directory: &Path,
    public_directory: &Path,
    assets_directory: &Path,
) -> Result<Vec<Group>, Box<dyn Error>> {
    let temp = assets_directory.join("pinyin-map.txt");
    let pinyin_path = temp.to_str().unwrap();
    let pinyins_map = get_pinyins_map(pinyin_path)?;

    let stroke_order_map = get_stroke_order_map(&assets_directory.join("stroke-order.txt"))?;

    let verbs = get_descriptors_from_file(&assets_directory.join("verbs.txt"))?;

    let countries = get_row_from_file(&assets_directory.join("countries.txt"), 1, ",")?;
    let cities = get_row_from_file(&assets_directory.join("cities.txt"), 1, ",")?;

    let hsk21 = get_lines_from_file(&assets_directory.join("hsk-version-2-1.txt"))?;
    let hsk22 = get_lines_from_file(&assets_directory.join("hsk-version-2-2.txt"))?;
    let hsk23 = get_lines_from_file(&assets_directory.join("hsk-version-2-3.txt"))?;
    let hsk24 = get_lines_from_file(&assets_directory.join("hsk-version-2-4.txt"))?;
    let hsk25 = get_lines_from_file(&assets_directory.join("hsk-version-2-5.txt"))?;
    let hsk26 = get_lines_from_file(&assets_directory.join("hsk-version-2-6.txt"))?;

    let hsk31 = get_row_from_file(&assets_directory.join("hsk-version-3-1.txt"), 1, " ")?;
    let hsk32 = get_row_from_file(&assets_directory.join("hsk-version-3-2.txt"), 1, " ")?;
    let hsk33 = get_row_from_file(&assets_directory.join("hsk-version-3-3.txt"), 1, " ")?;
    let hsk34 = get_row_from_file(&assets_directory.join("hsk-version-3-4.txt"), 1, " ")?;
    let hsk35 = get_row_from_file(&assets_directory.join("hsk-version-3-5.txt"), 1, " ")?;
    let hsk36 = get_row_from_file(&assets_directory.join("hsk-version-3-6.txt"), 1, " ")?;
    let hsk37 = get_row_from_file(&assets_directory.join("hsk-version-3-7.txt"), 1, " ")?;

    let dates_and_days_of_week =
        get_row_from_file(&assets_directory.join("dates-and-days-of-week.txt"), 0, ",")?;
    let chemical_elements = get_lines_from_file(&assets_directory.join("chemical-elements.txt"))?;
    let adjectives = get_lines_from_file(&assets_directory.join("adjectives.txt"))?;
    let adverbs = get_lines_from_file(&assets_directory.join("adverbs.txt"))?;

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

            let descriptor = verbs.get(&record.simplified);

            if descriptor.is_some() {
                let descriptor = descriptor.unwrap();
                for tag in &descriptor.tags {
                    detail.tags.push(tag.to_owned());
                }
            }

            if dates_and_days_of_week.contains(&key) {
                detail.tags.push("dates and days of week".to_owned());
            }

            if cities.contains(&key) {
                detail.tags.push("city".to_owned());
            }

            if countries.contains(&key) {
                detail.tags.push("country".to_owned());
            }

            if hsk21.contains(&key) {
                detail.tags.push("hsk-2-1".to_owned());
            }

            if hsk21.contains(&key) {
                detail.tags.push("hsk-2-1".to_owned());
            }

            if hsk22.contains(&key) {
                detail.tags.push("hsk-2-2".to_owned());
            }

            if hsk23.contains(&key) {
                detail.tags.push("hsk-2-3".to_owned());
            }

            if hsk24.contains(&key) {
                detail.tags.push("hsk-2-4".to_owned());
            }

            if hsk25.contains(&key) {
                detail.tags.push("hsk-2-5".to_owned());
            }

            if hsk26.contains(&key) {
                detail.tags.push("hsk-2-6".to_owned());
            }

            if hsk31.contains(&key) {
                detail.tags.push("hsk-3-1".to_owned());
            }

            if hsk32.contains(&key) {
                detail.tags.push("hsk-3-2".to_owned());
            }

            if hsk33.contains(&key) {
                detail.tags.push("hsk-3-3".to_owned());
            }

            if hsk34.contains(&key) {
                detail.tags.push("hsk-3-4".to_owned());
            }

            if hsk35.contains(&key) {
                detail.tags.push("hsk-3-5".to_owned());
            }

            if hsk36.contains(&key) {
                detail.tags.push("hsk-3-6".to_owned());
            }

            if hsk37.contains(&key) {
                detail.tags.push("hsk-3-7".to_owned());
            }

            if chemical_elements.contains(&key) {
                detail.tags.push("chemical element".to_owned());
            }

            if adjectives.contains(&key) {
                detail.tags.push("adjective".to_owned());
            }

            if adverbs.contains(&key) {
                detail.tags.push("adverb".to_owned());
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

                if descriptor.is_some() {
                    let descriptor = descriptor.unwrap();
                    if descriptor.meanings.contains(&meaning) {
                        meaning_record.lexical_item = Some(descriptor.lexical_item.to_owned());
                    }
                }

                if meaning.contains("(literary)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(literary)",
                        None,
                        Some("literary".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(chemistry)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(chemistry)",
                        None,
                        Some("chemistry".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(slang)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(slang)",
                        None,
                        Some("slang".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

                if meaning.contains("(vulgar)") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "(vulgar)",
                        None,
                        Some("vulgar".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
                    continue;
                }

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

                if meaning.contains("fig.") {
                    meaning_record = basic_refine_meaning(
                        meaning_record,
                        "fig.",
                        None,
                        Some("figuratively".to_owned()),
                    );
                    meaning_list.push(meaning_record.value.to_string());
                    meanings.push(meaning_record);
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
        }
    }

    Ok(output)
}
