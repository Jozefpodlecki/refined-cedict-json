use crate::PinyinMap;
use std::collections::HashMap;

pub fn to_pinyin(wade_giles_pinyin: &str, pinyin_map: &HashMap<String, PinyinMap>) -> String {
    let breakdown = wade_giles_pinyin.split(" ").map(|s| s.trim());
    let mut result = "".to_string();

    for item in breakdown {
        let mut normalized = item.to_string().to_lowercase();
        let chars: Vec<char> = normalized.chars().collect();
        let first_char = chars.first().unwrap();

        if chars.len() == 1 && first_char.is_alphabetic() {
            normalized = first_char.to_string();

            if result.is_empty() {
                result = result + &normalized;
            } else {
                result = result + " " + &normalized;
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

        let pinyin = pinyin_map
            .get(&normalized)
            .map(|pr| pr.pinyin.to_string())
            .unwrap_or(normalized);

        normalized = pinyin.to_owned();

        if result.is_empty() {
            result = result + &normalized;
        } else {
            result = result + " " + &normalized;
        }
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_convert() {
        let wade_giles_pinyin = "ren2 gong1";
        let mut pinyin_map: HashMap<String, PinyinMap> = HashMap::new();
        pinyin_map.insert(
            "ren2".to_string(),
            PinyinMap {
                pinyin: "rén".to_string(),
                wade_giles: "ren2".to_string(),
            },
        );
        pinyin_map.insert(
            "gong1".to_string(),
            PinyinMap {
                pinyin: "gōng".to_string(),
                wade_giles: "gong1".to_string(),
            },
        );

        let result = to_pinyin(wade_giles_pinyin, &pinyin_map);
        assert_eq!(result, "rén gōng");
    }
}
