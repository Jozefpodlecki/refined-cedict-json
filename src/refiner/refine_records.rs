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

    let descriptors = get_descriptors_from_file(&assets_directory.join("descriptor.txt"))?;
    let stroke_order_map = get_stroke_order_map(&assets_directory.join("stroke-order.txt"))?;

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
                tags: None,
                classifiers: None,
                decomposition: None,
                also_written: None,
                traditional: record.traditional,
            };

            detail.pronunciation.push(Pronunciation {
                pinyin: pinyin,
                wade_giles_pinyin: record.wade_giles_pinyin,
                other: None,
            });

            let mut meanings = detail.meanings;

            for meaning in record.meanings {
                let key = record.simplified.clone() + &meaning;

                if meaning.contains("also written") {
                    let mut value = str::replace(&meaning, "also written", "");
                    value = value.trim().to_owned();
                    detail.also_written = Some(value);
                    continue;
                }

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

                        let mut classifiers = detail.classifiers.clone().unwrap_or_default();
                        classifiers.push(classifier);
                        detail.classifiers = Some(classifiers);
                    }
                    continue;
                }

                let meaning_record = refine_meaning_record(&meaning);

                if let Some(mut record) = meaning_record {
                    let descriptor = descriptors.get(&key);

                    if descriptor.is_some() {
                        let descriptor = descriptor.unwrap();
                        let tags = descriptor.tags.clone().unwrap_or_default();
                        record.lexical_item = descriptor.lexical_item.clone();
                        let mut detail_tags = detail.tags.clone().unwrap_or_default();

                        for tag in tags {
                            if detail_tags.contains(&tag.to_string()) {
                                continue;
                            }

                            detail_tags.push(tag.to_string());
                        }

                        detail.tags = Some(detail_tags);
                    }

                    meanings.push(record);
                    continue;
                }
            }

            detail.meanings = meanings;

            new_record.details.push(detail);
        }

        grouped_records.push(new_record);
    }

    Ok(grouped_records)
}
