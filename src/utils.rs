use crate::api::get_info_from_writtenchinese;
use crate::api::get_stroke_count_from_wiktionary;
use crate::customReader::customReader::BufReader;
use crate::models::Descriptor;
use crate::CERecord;
use crate::PinyinMap;
use lazy_static::lazy_static;
use regex::Regex;
use select::predicate::{Attr, Class, Name};
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::LineWriter;
use std::path::Path;
use std::path::PathBuf;

#[cfg(test)]
#[path = "./test.rs"]
mod test;

pub fn is_cjk(data: &char) -> bool {
    match data {
        '\u{4E00}'..='\u{9FFF}' => true,
        '\u{3400}'..='\u{4DBF}' => true,
        '\u{20000}'..='\u{2A6DF}' => true,
        '\u{2A700}'..='\u{2B73F}' => true,
        '\u{2B740}'..='\u{2B81F}' => true,
        '\u{2B820}'..='\u{2CEAF}' => true,
        '\u{F900}'..='\u{FAFF}' => true,
        '\u{2F800}'..='\u{2FA1F}' => true,
        _ => false,
    }
}

pub fn get_stroke_order_map(file_path: &Path) -> Result<HashMap<String, u8>, Box<dyn Error>> {
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

pub fn get_descriptors_from_file(
    file_path: &Path,
) -> Result<HashMap<String, Descriptor>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashMap<String, Descriptor> = HashMap::new();

    for line in reader {
        let line = line?;

        let parts: Vec<&str> = line.split(",").collect();
        let meanings: Vec<String> = parts[2].split("/").map(|pr| pr.to_owned()).collect();
        let tags: Vec<String> = parts[2].split("/").map(|pr| pr.to_owned()).collect();

        let record = Descriptor {
            simplified: parts[0].to_owned(),
            pinyin: parts[1].to_owned(),
            meanings: meanings,
            lexical_item: parts[3].to_owned(),
            tags: tags,
        };

        list.insert(parts[0].to_string(), record);
    }

    Ok(list)
}

pub fn get1_lines_from_file(file_path: &Path) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut dict: HashMap<String, String> = HashMap::new();

    for line in reader {
        let line = line?;
        let mut parts = line.split(",").map(|pr| pr.trim().to_string());

        dict.insert(parts.next().unwrap(), parts.next().unwrap());
    }

    Ok(dict)
}

pub fn get_row_from_file(
    file_path: &Path,
    index: usize,
    separator: &str,
) -> Result<HashSet<String>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashSet<String> = HashSet::new();

    for line in reader {
        let line = line?;
        let rows: Vec<&str> = line.split(separator).collect();
        let row = rows.get(index);

        if row.is_none() {
            continue;
        }

        list.insert(row.unwrap().trim().to_owned());
    }

    Ok(list)
}

pub fn get_lines_from_file(file_path: &Path) -> Result<HashSet<String>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashSet<String> = HashSet::new();

    for line in reader {
        let line = line?;
        list.insert(line.trim().to_string());
    }

    Ok(list)
}

pub fn update_pinyins(pinyin_path: &str, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let pinyins = get_pinyins(pinyin_path)?;
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

pub fn write_lines_to_file(file_path: PathBuf, items: HashSet<char>) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut line_writer = LineWriter::new(file);

    for character in items {
        let line = format!("{}\n", character);
        line_writer.write_all(line.as_bytes())?;
    }

    Ok(())
}

pub fn import_stroke_order(file_path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let reader = BufReader::open(file_path.clone())?;
    let mut list = Vec::with_capacity(10000);

    for line in reader {
        let line = line?;

        let mut parts = line.split(" ").map(|s| s.trim());
        let key = parts.next().ok_or("")?;

        let stroke_count = get_stroke_count_from_wiktionary(key)?;

        if stroke_count.is_none() {
            println!("Could not process character: {}", line);
            continue;
        }

        let line = format!("{} {}\n", key, stroke_count.unwrap());
        print!("Processing {}", line);
        list.push(line);
    }

    Ok(list)
}

pub fn get_single_characters(records: &[CERecord]) -> HashSet<char> {
    let mut single_characters: HashSet<char> = HashSet::with_capacity(records.len() * 2);

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

        if chars.len() != 1 {
            continue;
        }

        let character = chars[0];

        if is_cjk(&character) {
            single_characters.insert(character);
        }
    }

    single_characters
}

pub fn read_file_bytes(file_path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let file_size = fs::metadata(&file_path)?.len();
    let mut bytes = Vec::with_capacity(usize::try_from(file_size)?);
    File::open(file_path)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn try_get_ce_dict_records(
    file_path: &Path,
    cache_path: &Path,
) -> Result<Vec<CERecord>, Box<dyn Error>> {
    let lines = BufReader::open(file_path)?;
    let mut list: Vec<CERecord> = Vec::new();

    if cache_path.exists() {
        let bytes = &read_file_bytes(cache_path)?;
        list = serde_json::from_slice(bytes)?;
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

pub fn get_pinyins_map(file_path: &str) -> Result<HashMap<String, PinyinMap>, Box<dyn Error>> {
    //let pinyins = get_pinyins(pinyin_path);
    let reader = BufReader::open(file_path)?;
    let mut pinyins_map: HashMap<String, PinyinMap> = HashMap::new();

    for line in reader {
        let line = line?;

        let parts: Vec<&str> = line.split(" ").collect();

        let pinyin = PinyinMap {
            pinyin: parts[0].to_string(),
            wade_giles: parts[1].to_string(),
        };
        pinyins_map.insert(parts[1].to_owned(), pinyin);
    }

    // for pinyin in pinyins {
    //     let wade_giles = pinyin.wade_giles.clone();
    //     pinyins_map.insert(wade_giles.clone(), pinyin);
    // }
    Ok(pinyins_map)
}

pub fn get_pinyins(file_path: &str) -> Result<BTreeSet<PinyinMap>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;

    let mut pinyins: BTreeSet<PinyinMap> = BTreeSet::new();

    for line in reader {
        let line = line?;

        let parts: Vec<&str> = line.split(" ").collect();

        let pinyin = PinyinMap {
            pinyin: parts[0].to_string(),
            wade_giles: parts[1].to_string(),
        };
        pinyins.insert(pinyin);
    }

    Ok(pinyins)
}

pub fn remove_duplicates(file_path: &str) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;

    //let mut pinyins: HashSet<PinyinMap> = HashSet::new();
    let mut pinyins: BTreeSet<PinyinMap> = BTreeSet::new();

    for line in reader {
        let line = line?;

        let parts: Vec<&str> = line.split(" ").collect();

        let pinyin = PinyinMap {
            pinyin: parts[0].to_string(),
            wade_giles: parts[1].to_string(),
        };
        pinyins.insert(pinyin);
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .unwrap();
    let mut file = LineWriter::new(file);

    for pinyin in pinyins {
        let line = format!("{} {}\n", pinyin.pinyin, pinyin.wade_giles);
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

pub fn parse_ce_record(line: &str, line_number: u32) -> CERecord {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(
            r"(?P<traditional>.*?)\s(?P<simplified>.*?)\s\[(?P<pinyin>.*?)\]\s/(?P<meanings>.*)/",
        )
        .unwrap();
    }

    let captures = REGEX.captures(&line).unwrap();
    let traditional = captures.name("traditional").unwrap().as_str().to_string();
    let simplified = captures.name("simplified").unwrap().as_str().to_string();
    let wade_giles_pinyin = captures.name("pinyin").unwrap().as_str().to_string();
    let meanings = captures
        .name("meanings")
        .unwrap()
        .as_str()
        .split("/")
        .map(|s| s.to_string())
        .collect();

    let mut normalized_line = line.to_string();
    normalized_line.pop();
    normalized_line.pop();

    CERecord {
        line_number: line_number,
        line: normalized_line,
        simplified: simplified,
        traditional: traditional,
        wade_giles_pinyin: wade_giles_pinyin,
        meanings: meanings,
    }
}
