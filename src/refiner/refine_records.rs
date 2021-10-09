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

pub fn parse_decomposition(line: &str, lookup: &HashMap<String, Radical>) -> Option<Decomposition> {
    lazy_static! {
        static ref EXTRACT_REGEX: Regex = Regex::new(r"(\(.*?\))|\s").unwrap();
    }
    let linee = EXTRACT_REGEX.replace_all(line, "");
    let parts: Vec<&str> = linee.split(";").collect();

    if parts.len() == 1 {
        return None;
    }

    let radical: Vec<Radical> = parts[2]
        .split(",")
        .map(|pr| lookup.get(pr).unwrap().to_owned())
        .collect();

    let graphical: Vec<String> = parts[3].split(",").map(|pr| pr.to_string()).collect();

    Some(Decomposition { radical, graphical })
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_handle_empty() {
        let lookup: HashMap<String, Radical> = HashMap::new();

        let line = "𬬸";
        let result = parse_decomposition(&line, &lookup);
        assert!(result.is_none());
    }

    #[test]
    fn should_create_decomposition() {
        let mut lookup: HashMap<String, Radical> = HashMap::new();
        lookup.insert(
            "女".to_string(),
            Radical {
                stroke_count: 1,
                meaning: "woman".to_string(),
                value: "女".to_string(),
                pinyin: "".to_string(),
            },
        );
        lookup.insert(
            "耳".to_string(),
            Radical {
                stroke_count: 1,
                meaning: "ear".to_string(),
                value: "耳".to_string(),
                pinyin: "".to_string(),
            },
        );
        lookup.insert(
            "又".to_string(),
            Radical {
                stroke_count: 1,
                meaning: "right hand".to_string(),
                value: "又".to_string(),
                pinyin: "".to_string(),
            },
        );

        let line = "娵;女, 取;女 (woman), 耳 (ear), 又 (right hand);㇛, 一, 丿, 二, 丨, 二, ㇇, ㇏";
        let result = parse_decomposition(&line, &lookup).unwrap();
        let expected_radicals = vec![
            lookup.get("女").unwrap().to_owned(),
            lookup.get("耳").unwrap().to_owned(),
            lookup.get("又").unwrap().to_owned(),
        ];

        assert_eq!(result.radical, expected_radicals);
        assert_eq!(
            result.graphical,
            vec!["㇛", "一", "丿", "二", "丨", "二", "㇇", "㇏"]
        );
    }
}
