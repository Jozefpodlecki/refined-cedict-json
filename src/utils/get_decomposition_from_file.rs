use crate::customReader::custom_reader::BufReader;
use crate::models::Decomposition;
use crate::models::Radical;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::io::prelude::*;
use std::path::Path;

pub fn get_decomposition_from_file(
    file_path: &Path,
    radical_lookup: &HashMap<String, Radical>,
) -> Result<HashMap<String, Decomposition>, Box<dyn Error>> {
    lazy_static! {
        static ref EXTRACT_REGEX: Regex = Regex::new(r"(\(.*?\))|\s").unwrap();
    }

    let lines = BufReader::open(file_path)?;
    let mut dict: HashMap<String, Decomposition> = HashMap::new();

    for line in lines {
        let line = line?;

        let linee = EXTRACT_REGEX.replace_all(&line, "");
        let parts: Vec<&str> = linee.split(";").collect();

        if parts.len() == 1 {
            return Err("invalid file format".into());
        }

        let radical: Vec<Radical> = parts[2]
            .split(",")
            .map(|pr| radical_lookup.get(pr).unwrap().to_owned())
            .collect();

        let graphical: Vec<String> = parts[3].split(",").map(|pr| pr.to_string()).collect();

        let record = Decomposition { radical, graphical };

        dict.insert(parts[0].to_owned(), record);
    }

    Ok(dict)
}
