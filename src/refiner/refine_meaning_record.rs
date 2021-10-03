use crate::models::Meaning;
use lazy_static::lazy_static;
use regex::Regex;

const PATTERN_CONTEXT_LIST: &[(&str, &str)] = &[
    ("(politics)", "politics"),
    ("(medicine)", "medicine"),
    ("(biochemistry)", "biochemistry"),
    ("(biology)", "biology"),
    ("(math)", "math"),
    ("(physics)", "physics"),
    ("(literary)", "literary"),
    ("(chemistry)", "chemistry"),
    ("(vulgar)", "vulgar"),
    ("(old)", "old"),
    ("(slang)", "slang"),
    ("(Tw)", "taiwan"),
    ("(loanword)", "loanword"),
    ("(geology)", "geology"),
    ("(architecture)", "architecture"),
    ("(electricity)", "electricity"),
    ("(networking)", "networking"),
    ("(computing)", "computing"),
    ("(onom.)", "onomatopoeia"),
    (
        "(Japanese surname and place name)",
        "Japanese surname and place name",
    ),
    ("(honorific)", "honorific"),
    ("(dialect)", "dialect"),
    ("(polite)", "polite"),
    ("(fig.)", "figuratively"),
    ("(coll.)", "colloquial"),
    ("(law.)", "law"),
    ("(lit. and fig.)", "literary and figuratively"),
    ("(Japanese surname)", "Japanese surname"),
    ("(Internet slang)", "Internet slang"),
    ("(bird species of China)", "bird species of China"),
    ("(loanword from Japanese)", "loanword from Japanese"),
];

pub fn refine_meaning_record(meaning: &str) -> Option<Meaning> {
    lazy_static! {
        static ref SIMPL_TRAD_PIN_REGEX: Regex = Regex::new(r"([^|]*)\|?(.*?)\[(.*?)\]").unwrap();
        static ref SIMPL_TRAD_REGEX: Regex = Regex::new(r"([^|]*)\|?([^\[,]*)").unwrap();
        static ref EXTRACT_PINYIN_REGEX: Regex = Regex::new(r"\[(?P<pinyin>.*?)\]").unwrap();
        static ref EXTRACT_IDIOM_REGEX: Regex = Regex::new(r"\(idiom,?[^\)].*\)").unwrap();
    }

    let mut meaning_record = Meaning {
        context: None,
        lexical_item: None,
        simplified: None,
        traditional: None,
        wade_giles_pinyin: None,
        pinyin: None,
        literal_meaning: None,
        value: Some(meaning.to_string()),
    };

    let mut meaning_record_context: Option<Vec<String>> = None;
    let mut value = meaning_record.value.unwrap();

    for (pattern, context) in PATTERN_CONTEXT_LIST {
        if meaning.contains(pattern) {
            if meaning_record_context.is_none() {
                meaning_record_context = Some(vec![context.to_string()]);
            } else {
                let mut temp = meaning_record_context.unwrap();
                temp.push(context.to_string());
                meaning_record_context = Some(temp);
            }

            value = str::replace(&value, pattern, "");
            value = value.trim().to_owned();
        }
    }

    meaning_record.context = meaning_record_context;
    meaning_record.value = Some(value.to_string());

    if meaning_record.context.is_some() {
        return Some(meaning_record);
    }

    if meaning.contains("Japanese") {
        meaning_record.context = Some(vec!["Japanese".to_string()]);
    }

    if EXTRACT_IDIOM_REGEX.is_match(&meaning) {
        let value = EXTRACT_IDIOM_REGEX.replace(&meaning, "");

        if value.contains(";") {
            let parts: Vec<&str> = value.split(";").map(|pr| pr.trim()).collect();
            meaning_record.value = Some(parts[1].to_string());
            meaning_record.literal_meaning = Some(parts[0].to_string());
        } else {
            meaning_record.value = Some(value.to_string());
        }
        return Some(meaning_record);
    }

    if meaning.contains("(idiom)") {
        let mut value = str::replace(&meaning, "(idiom)", "");
        value = value.trim().to_owned();
        meaning_record.lexical_item = Some("idiom".to_owned());

        if meaning.contains("fig.") {
            let pattern = "fig.";
            value = str::replace(&value, pattern, "");
            meaning_record.context = Some(vec!["figuratively".to_owned()]);
        }

        if meaning.contains("lit.") {
            let pattern = "lit.";
            value = str::replace(&value, pattern, "");
            meaning_record.context = Some(vec!["literature".to_owned()]);
        }

        if value.contains(";") {
            let parts: Vec<&str> = value.split(";").map(|pr| pr.trim()).collect();
            meaning_record.value = Some(parts[1].to_string());
            meaning_record.literal_meaning = Some(parts[0].to_string());
        } else {
            meaning_record.value = Some(value);
        }
        return Some(meaning_record);
    }

    if meaning.contains("lit.") {
        let mut pattern = "lit.";
        let mut processed = meaning.to_string();

        processed = str::replace(&processed, pattern, "");
        let result = processed.trim();

        meaning_record.context = Some(vec!["literature".to_string()]);
        meaning_record.value = Some(result.to_string());
        return Some(meaning_record);
    }

    if meaning.starts_with("see ") {
        let pattern = "see";
        let mut processed = str::replace(&meaning, pattern, "");
        processed = processed.trim().to_owned();
        let captures = SIMPL_TRAD_PIN_REGEX.captures(&processed);

        if captures.is_none() {
            debug!("{}", processed.to_string());
            meaning_record.value = Some(processed.to_string());
        } else {
            let captures = captures.unwrap();
            meaning_record.value = None;
            meaning_record.context = Some(vec!["see".to_owned()]);
            meaning_record.simplified = Some(captures.get(1).unwrap().as_str().to_owned());
            meaning_record.traditional = Some(captures.get(2).unwrap().as_str().to_owned());
            meaning_record.wade_giles_pinyin =
                Some(captures.get(3).unwrap().as_str().to_lowercase());
        }

        return Some(meaning_record);
    }

    if meaning.starts_with("see also") {
        let pattern = "see also";
        let mut processed = str::replace(&meaning, pattern, "");
        processed = processed.trim().to_owned();
        let captures = SIMPL_TRAD_PIN_REGEX.captures(&processed);

        if captures.is_none() {
            debug!("{}", processed.to_string());
            meaning_record.value = Some(processed);
        } else {
            let captures = captures.unwrap();
            meaning_record.value = None;
            meaning_record.context = Some(vec!["see also".to_owned()]);
            meaning_record.simplified = Some(captures.get(1).unwrap().as_str().to_owned());
            meaning_record.traditional = Some(captures.get(2).unwrap().as_str().to_owned());
            meaning_record.wade_giles_pinyin =
                Some(captures.get(3).unwrap().as_str().to_lowercase());
        }

        return Some(meaning_record);
    }

    if meaning.contains("variant") {
        let mut pattern = "variant of";
        let mut temp = meaning_record.context.unwrap_or_default();
        let mut processed = meaning.to_string();

        if meaning.contains("Japanese variant of") {
            pattern = "Japanese variant of";
            temp.push("variant".to_owned());
        } else if meaning.contains("old variant") {
            pattern = "old variant of";
            temp.push("old variant".to_owned());
        } else {
            temp.push("variant".to_owned());
        }
        meaning_record.context = Some(temp);

        processed = str::replace(&processed, pattern, "");
        processed = processed.trim().to_owned();

        let captures = SIMPL_TRAD_PIN_REGEX
            .captures(&processed)
            .or(SIMPL_TRAD_REGEX.captures(&processed));

        if captures.is_none() {
            meaning_record.value = Some(processed.to_string());
        } else {
            let captures = captures.unwrap();
            meaning_record.value = None;
            meaning_record.simplified = Some(captures.get(1).unwrap().as_str().to_owned());
            meaning_record.traditional = Some(captures.get(2).unwrap().as_str().to_owned());
            meaning_record.wade_giles_pinyin = captures.get(3).map(|pr| pr.as_str().to_lowercase());
        }

        return Some(meaning_record);
    }

    if meaning.contains("abbr.") {
        let mut pattern = "abbr. for";

        if meaning.contains("(abbr.)") {
            pattern = "(abbr.)";
        }

        let mut processed = str::replace(&meaning, pattern, "");
        processed = processed.trim().to_owned();
        meaning_record.context = Some(vec!["abbreviation".to_owned()]);

        let processed = &meaning_record.value.clone().unwrap();

        let captures = SIMPL_TRAD_PIN_REGEX.captures(processed);

        if captures.is_none() {
        } else {
            let captures = captures.unwrap();
            meaning_record.value = None;
            meaning_record.simplified = Some(captures.get(1).unwrap().as_str().to_owned());
            meaning_record.traditional = Some(captures.get(2).unwrap().as_str().to_owned());
            meaning_record.wade_giles_pinyin =
                Some(captures.get(3).unwrap().as_str().to_lowercase());
        }

        return Some(meaning_record);
    }

    return None;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_handle_text_with_variant() {
        let line = "variant of 邱吉爾|邱吉尔[Qiu1 ji2 er3]";
        let result = refine_meaning_record(&line).unwrap();
        assert_eq!(result.value, None);
        assert_eq!(result.context.unwrap()[0], "variant");
        assert_eq!(result.simplified.unwrap(), "邱吉爾");
        assert_eq!(result.traditional.unwrap(), "邱吉尔");
        assert_eq!(result.wade_giles_pinyin.unwrap(), "qiu1 ji2 er3");
    }

    #[test]
    fn should_handle_text_with_idiom() {
        let line = "lit. family shames must not be spread abroad (idiom); fig. don't wash your dirty linen in public";
        let result = refine_meaning_record(&line).unwrap();
        assert_eq!(result.context.unwrap()[0], "literature");
        assert_eq!(result.lexical_item.unwrap(), "idiom");
        assert_eq!(
            result.value.unwrap(),
            "don't wash your dirty linen in public"
        );
        assert_eq!(
            result.literal_meaning.unwrap(),
            "family shames must not be spread abroad"
        );
    }

    #[test]
    fn should_handle_text_with_variant_and_description() {
        let line = "variant of 開國元勳|开国元勋, founding figure (of country or dynasty)";
        let result = refine_meaning_record(&line).unwrap();
        assert_eq!(result.context.unwrap(), vec!["variant"]);
        assert_eq!(result.value, None);
        assert_eq!(result.simplified.unwrap(), "開國元勳");
        assert_eq!(result.traditional.unwrap(), "开国元勋");
    }

    #[test]
    fn should_handle_text_with_japanese_and_variant() {
        let line = "Japanese variant of 劍|剑";
        let result = refine_meaning_record(&line).unwrap();
        assert_eq!(result.context.unwrap(), vec!["Japanese", "variant"]);
        assert_eq!(result.value, None);
        assert_eq!(result.simplified.unwrap(), "劍");
        assert_eq!(result.traditional.unwrap(), "剑");
    }

    #[test]
    fn should_handle_text_with_two_contexts() {
        let line = "(coll.) (Tw) don't mention it";
        let result = refine_meaning_record(&line).unwrap();
        assert_eq!(result.context.unwrap(), vec!["taiwan", "colloquial"]);
        assert_eq!(result.value.unwrap(), "don't mention it");
    }
}
