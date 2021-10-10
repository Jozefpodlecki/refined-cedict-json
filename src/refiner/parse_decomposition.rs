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
