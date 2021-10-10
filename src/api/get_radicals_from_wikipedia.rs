use lazy_static::lazy_static;
use regex::Regex;
use soup::prelude::*;
use std::error::Error;

use crate::api::USER_AGENT;
use crate::models::Radical;

pub fn get_radicals_from_wikipedia() -> Result<Vec<Radical>, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
        static ref COMMA_CURLY_BRACES: Regex = Regex::new(r"、|\(|\)").unwrap();
    }
    let mut list: Vec<Radical> = Vec::new();

    const BASE_URL: &str = "https://en.wikipedia.org/wiki/Kangxi_radical";
    let response = CLIENT
        .get(BASE_URL)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    let body_response = response.text()?;
    let soup = Soup::new(&body_response);

    let table_elements = soup.tag("table").find_all().collect::<Vec<_>>();

    for element in table_elements[2].tag("tr").find_all().skip(1) {
        let text = element.text();
        let items = text
            .trim()
            .split("\n")
            .filter(|pr| !pr.is_empty())
            .collect::<Vec<_>>();

        let mut values = items[1].to_owned();

        values = COMMA_CURLY_BRACES.replace_all(&values, " ").to_string();
        let values = values
            .split(" ")
            .filter(|pr| !pr.is_empty())
            .map(|pr| pr.to_string());

        let stroke_count = items[2].trim().parse::<u8>().unwrap().to_owned();
        let meaning = items[3].to_owned();
        let pinyin = items[4].to_owned();

        for value in values {
            let record = Radical {
                meaning: meaning.clone(),
                stroke_count,
                value,
                pinyin: pinyin.clone(),
            };
            list.push(record);
        }
    }

    Ok(list)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_get_radicals_from_wikipedia() {
        let radicals = get_radicals_from_wikipedia().unwrap();
        assert_eq!(radicals.len(), 282);
    }
}
