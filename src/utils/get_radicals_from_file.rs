use crate::customReader::custom_reader::BufReader;
use crate::models::Radical;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

pub fn get_radicals_from_file(
    file_path: &Path,
) -> Result<HashMap<String, Radical>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut dict: HashMap<String, Radical> = HashMap::with_capacity(300);

    for line in reader {
        let line = line?;
        let parts: Vec<&str> = line.split(";").collect();

        if parts.len() < 4 {
            return Err("invalid file format".into());
        }

        let value = parts[0].to_owned();
        let meaning = parts[1].to_owned();
        let stroke_count = parts[2].parse::<u8>()?;
        let pinyin = parts[3].to_owned();

        let record = Radical {
            value,
            meaning,
            pinyin,
            stroke_count,
        };

        dict.insert(parts[0].to_owned(), record);
    }

    Ok(dict)
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[test]
    fn should_get_radicals_from_wikipedia() {
        let current_directory = env::current_dir().unwrap();
        let assets_directory = current_directory.join("assets");

        let radicals = get_radicals_from_file(&assets_directory.join("radicals.txt")).unwrap();
        assert_eq!(radicals.len(), 282);
    }
}
