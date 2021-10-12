use crate::customReader::custom_reader::BufReader;
use crate::models::Descriptor;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

pub fn get_descriptors_from_file(
    file_path: &Path,
) -> Result<HashMap<String, Descriptor>, Box<dyn Error>> {
    let reader = BufReader::open(file_path)?;
    let mut list: HashMap<String, Descriptor> = HashMap::with_capacity(200000);

    for line in reader {
        let line = line?;

        let parts: Vec<&str> = line.split(",").map(|pr| pr.trim()).collect();

        let simplified = parts[0].to_owned();
        let pinyin = parts[1].to_owned();
        let meaning = parts[2].to_owned();
        let lexical_item = parts
            .get(3)
            .map(|pr| pr.to_string())
            .filter(|pr| !pr.is_empty());
        let tags_str = parts.get(4);
        let tags = tags_str.map(|pr| {
            pr.split("/")
                .map(|pr| pr.to_owned())
                .filter(|pr| !pr.is_empty())
                .collect()
        });

        let key = simplified.clone() + &meaning;

        let record = Descriptor {
            simplified: simplified,
            pinyin: pinyin,
            meaning: meaning,
            lexical_item: lexical_item,
            tags: tags,
        };

        list.insert(key, record);
    }

    Ok(list)
}
