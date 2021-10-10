use crate::models::*;
use crate::refiner::refine_meaning_record::refine_meaning_record;
use crate::refiner::to_pinyin::to_pinyin;
use crate::utils::get_abbreviations_from_file::get_abbreviations_from_file;
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

    let abbreviations = get_abbreviations_from_file(&assets_directory.join("abbreviations.txt"))?;
    let descriptors = get_descriptors_from_file(&assets_directory.join("descriptor.txt"))?;
    let stroke_order_map = get_stroke_order_map(&assets_directory.join("stroke-order.txt"))?;

    let mut index = 1;
    let mut grouped_records: Vec<Group> = Vec::with_capacity(116725);

    lazy_static! {
        static ref EXTRACT_CLASSIFIER_REGEX: Regex =
            Regex::new(r"(.*?[^|])\|?(.*?)\[(.*?)\]").unwrap();
        static ref EXTRACT_PINYIN_REGEX: Regex = Regex::new(r"\[(?P<pinyin>.*?)\]").unwrap();
        static ref EXTRACT_IDIOM_REGEX: Regex = Regex::new(r"\(idiom,?[^\)].*\)").unwrap();
        static ref ALSO_WRITTEN_SIMPL_TRAD_PINYIN_REGEX: Regex =
            Regex::new(r"^also written ([^|\[]+)(?:\|([^\[\]|]+))?(?:\[([^\[\]]+)])?").unwrap();
        static ref COMPLEX_ABBR_REGEX: Regex = Regex::new(r"abbr\.\s(for|of|to)").unwrap();
    }

    for (key, records) in records {
        info!("{}: Processing: {}", index, key);
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
                variant: None,
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

                if COMPLEX_ABBR_REGEX.is_match(&meaning) {
                    if let Some(list) = abbreviations.get(&record.simplified) {
                        for item in list {
                            meanings.push(Meaning {
                                context: Some(vec!["abbreviation".to_string()]),
                                lexical_item: None,
                                simplified: item.simplified.to_owned(),
                                traditional: item.traditional.to_owned(),
                                literal_meaning: None,
                                pinyin: None,
                                value: item.value.to_owned(),
                                wade_giles_pinyin: item.wade_giles_pinyin.to_owned(),
                            });
                        }
                    }
                    continue;
                }

                if meaning.starts_with("also written") {
                    let captures = ALSO_WRITTEN_SIMPL_TRAD_PINYIN_REGEX
                        .captures(&meaning)
                        .unwrap();
                    let simplified = captures.get(1).unwrap().as_str().to_owned();
                    let traditional = captures.get(2).map(|pr| pr.as_str().to_owned());
                    let wade_giles_pinyin = captures.get(3).map(|pr| pr.as_str().to_owned());

                    let variant = Variant {
                        simplified,
                        traditional,
                        wade_giles_pinyin,
                    };

                    detail.variant = Some(variant);
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
    use std::env;

    use super::*;

    #[test]
    fn should_refine_records() {
        let current_directory = env::current_dir().unwrap();
        let assets_directory = current_directory.join("assets");
        let public_directory = current_directory.join("public");
        let mut records: HashMap<String, Vec<CERecord>> = HashMap::new();
        let mut group: Vec<CERecord> = Vec::new();
        let key = "交通大学".to_string();
        let expected = CERecord {
            line: "".to_owned(),
            line_number: 1,
            meanings: vec!["abbr. for 上海交通大學|上海交通大学 Shanghai Jiao Tong University, 西安交通大學|西安交通大学 Xia'an Jiaotong University, 國立交通大學|国立交通大学 National Chiao Tung University (Taiwan) etc".to_string()],
            simplified: key.to_string(),
            traditional: "交通大學".to_string(),
            wade_giles_pinyin: "jiao1 tong1 da4 xue2".to_string(),
        };
        group.push(expected.clone());
        records.insert(key.to_owned(), group);

        let groups = refine_records(
            records,
            &current_directory,
            &public_directory,
            &assets_directory,
        )
        .unwrap();
        let actual = &groups[0];
        let details = &actual.details[0];
        assert_eq!(groups.len(), 1);
        assert_eq!(actual.simplified, expected.simplified);
        assert_eq!(details.traditional, expected.traditional);
        let meanings = &details.meanings[0];
        assert_eq!(
            meanings.context.as_ref().unwrap().first().unwrap(),
            "abbreviation"
        );
        assert_eq!(meanings.simplified.as_ref().unwrap(), "上海交通大学");
        assert_eq!(meanings.traditional.as_ref().unwrap(), "上海交通大學");
    }
}
