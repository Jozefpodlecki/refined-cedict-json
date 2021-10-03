use crate::models::CERecord;
use lazy_static::lazy_static;
use regex::Regex;

pub fn parse_ce_record(line: &str, line_number: u32) -> CERecord {
    let mut line = line.to_string();

    if line.contains("lu:4") {
        line = line.replace("lu:4", "lv4");
    }

    if line.contains("lu:3") {
        line = line.replace("lu:3", "lv3");
    }

    if line.contains("lu:2") {
        line = line.replace("lu:2", "lv2");
    }

    if line.contains("lu:e4") {
        line = line.replace("lu:e4", "lve4");
    }

    if line.contains("nu:e4") {
        line = line.replace("nu:e4", "nve4");
    }

    lazy_static! {
        static ref REGEX: Regex = Regex::new(
            r"(?P<traditional>.*?)\s(?P<simplified>.*?)\s\[(?P<pinyin>.*?)\]\s/(?P<meanings>.*)/",
        )
        .unwrap();
    }

    let captures = REGEX.captures(&line).unwrap();
    let traditional = captures.name("traditional").unwrap().as_str().to_string();
    let simplified = captures.name("simplified").unwrap().as_str().to_string();
    let mut wade_giles_pinyin = captures.name("pinyin").unwrap().as_str().to_lowercase();
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_lowercase_pinyin() {
        let line = "万俟 万俟 [Mo4 qi2] /polysyllabic surname Moqi/";
        let result = parse_ce_record(&line, 1);
        assert_eq!(result.wade_giles_pinyin, "mo4 qi2");
    }

    #[test]
    fn should_return_struct() {
        let line = "如泣如訴 如泣如诉 [ru2 qi4 ru2 su4] /lit. as if weeping and complaining (idiom)/fig. mournful (music or singing)/";
        let result = parse_ce_record(&line, 1);
        assert_eq!(result.simplified, "如泣如诉");
        assert_eq!(result.traditional, "如泣如訴");
        assert_eq!(result.wade_giles_pinyin, "ru2 qi4 ru2 su4");
        assert_eq!(
            result.meanings[0],
            "lit. as if weeping and complaining (idiom)"
        );
        assert_eq!(result.meanings[1], "fig. mournful (music or singing)");
        assert_eq!(result.line_number, 1);
    }
}
