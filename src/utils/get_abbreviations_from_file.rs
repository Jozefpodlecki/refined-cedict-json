use crate::customReader::custom_reader::BufReader;
use crate::models::Abbreviation;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

pub fn get_abbreviations_from_file(
    file_path: &Path,
) -> Result<HashMap<String, Vec<Abbreviation>>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut dict: HashMap<String, Vec<Abbreviation>> = HashMap::with_capacity(1200);

    for line in reader {
        let line = line?;
        let parts: Vec<String> = line.split(";").map(|pr| pr.trim().to_owned()).collect();
        let key = Rc::new(parts[0].to_owned());
        let value = parts
            .get(1)
            .filter(|pr| !pr.is_empty())
            .map(|pr| pr.to_string());
        let traditional = parts
            .get(2)
            .filter(|pr| !pr.is_empty())
            .map(|pr| pr.to_string());
        let simplified = parts
            .get(3)
            .filter(|pr| !pr.is_empty())
            .map(|pr| pr.to_string());
        let wade_giles_pinyin = parts
            .get(4)
            .filter(|pr| !pr.is_empty())
            .map(|pr| pr.to_string());

        let record = Abbreviation {
            key: key.to_string(),
            value,
            wade_giles_pinyin,
            simplified,
            traditional,
        };

        match dict.get_mut(&key.to_string()) {
            Some(list) => {
                list.push(record);
            }
            None => {
                dict.insert(key.to_string(), vec![record]);
            }
        }
    }

    Ok(dict)
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[test]
    fn should_get_abbreviations() {
        let current_directory = env::current_dir().unwrap();
        let assets_directory = current_directory.join("assets");
        let file_path = &assets_directory.join("abbreviations.txt");
        let result = get_abbreviations_from_file(&file_path).unwrap();
        let abbreviations = &result["交通大学"];
        assert_eq!(
            abbreviations[0].value.as_ref().unwrap(),
            "Shanghai Jiao Tong University"
        );
        assert_eq!(
            abbreviations[0].simplified.as_ref().unwrap(),
            "上海交通大学"
        );
        assert_eq!(
            abbreviations[0].traditional.as_ref().unwrap(),
            "上海交通大學"
        );
    }
}
