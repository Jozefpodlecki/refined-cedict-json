use crate::models::*;
use crate::refiner::refine_meaning_record::refine_meaning_record;
use crate::refiner::to_pinyin::to_pinyin;
use crate::utils::*;
use crate::CERecord;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

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

    let verbs = get_descriptors_from_file(&assets_directory.join("verbs.txt"))?;
    let dates_and_days_of_week =
        get_row_from_file(&assets_directory.join("dates-and-days-of-week.txt"), 0, ",")?;
    let chemical_elements = get_lines_from_file(&assets_directory.join("chemical-elements.txt"))?;
    let adjectives = get_lines_from_file(&assets_directory.join("adjectives.txt"))?;
    let adverbs = get_lines_from_file(&assets_directory.join("adverbs.txt"))?;

    let mut index = 1;
    let mut grouped_records: Vec<Group> = Vec::with_capacity(116725);

    lazy_static! {
        static ref EXTRACT_CLASSIFIER_REGEX: Regex =
            Regex::new(r"(.*?[^|])\|?(.*?)\[(.*?)\]").unwrap();
        static ref EXTRACT_PINYIN_REGEX: Regex = Regex::new(r"\[(?P<pinyin>.*?)\]").unwrap();
        static ref EXTRACT_IDIOM_REGEX: Regex = Regex::new(r"\(idiom,?[^\)].*\)").unwrap();
    }

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
                other: None,
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
                if meaning.contains("also pr.") {
                    let captures = EXTRACT_PINYIN_REGEX.captures(&meaning);

                    if captures.is_none() {
                        let mut processed = str::replace(&meaning, "also pr. ", "");
                        processed = processed.trim().to_owned();

                        detail.pronunciation.push(Pronunciation {
                            pinyin: "".to_string(),
                            wade_giles_pinyin: "".to_string(),
                            other: Some(processed),
                        });
                        continue;
                    }

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
                        other: None,
                    });
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

                let meaning_record = refine_meaning_record(&meaning);

                match meaning_record {
                    Some(record) => {
                        meanings.push(record);
                        continue;
                    }
                    None => {}
                }

                // if descriptor.is_some() {
                //     let descriptor = descriptor.unwrap();
                //     if descriptor.meanings.contains(&meaning) {
                //         meaning_record.unwrap().lexical_item =
                //             Some(descriptor.lexical_item.to_owned());
                //     }
                // }
            }

            detail.meanings = meanings;

            new_record.details.push(detail);
        }

        grouped_records.push(new_record);
    }

    Ok(grouped_records)
}
