use lazy_static::lazy_static;
use regex::Regex;
use scraper::Html;
use scraper::Selector;
use std::error::Error;
use urlencoding::encode;

use crate::api::USER_AGENT;

pub fn get_character_decomposition_from_hanzicraft(
    character: &str,
) -> Result<String, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
        static ref ARROW_PATTERN: Regex = Regex::new(r"(?:.*\s=>\s)(.*)").unwrap();
    }

    const BASE_URL: &str = "https://hanzicraft.com/character/";
    let encoded = encode(character);
    let url = format!("{}{}", BASE_URL, encoded);
    let response = CLIENT
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    let body_response = response.text()?;
    let parsed_html = Html::parse_document(&body_response);

    let selector = &Selector::parse(".decompbox").unwrap();
    let mut parts: Vec<String> = Vec::new();

    for element in parsed_html.select(&selector) {
        let text: String = element.text().collect();
        let captures = ARROW_PATTERN.captures(&text).unwrap();
        let part = captures.get(1).unwrap().as_str().trim().to_owned();
        parts.push(part);
    }

    parts.insert(0, character.to_owned());

    let line = parts.join(";");

    Ok(line)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_get_decomposition() {
        let character = "要";
        let line = get_character_decomposition_from_hanzicraft(character).unwrap();
        assert_eq!(line, "要;覀, 女;覀 (west), 女 (woman);覀, ㇛, 一, 丿");
    }
}
