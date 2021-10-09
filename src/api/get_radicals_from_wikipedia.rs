use lazy_static::lazy_static;
use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;
use soup::prelude::*;
use std::error::Error;

use crate::api::USER_AGENT;
use crate::models::Radical;

pub fn get_radicals_from_wikipedia() -> Result<Vec<Radical>, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
    }
    let mut list: Vec<Radical> = Vec::new();

    const BASE_URL: &str = "https://en.wikipedia.org/wiki/Kangxi_radical";
    let response = CLIENT
        .get(BASE_URL)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?;

    let body_response = response.text()?;
    let parsed_html = Html::parse_document(&body_response);

    let selector = &Selector::parse("tbody > tr").unwrap();

    for element in parsed_html.select(&selector) {
        let items = element
            .children()
            .filter_map(|child| ElementRef::wrap(child))
            .flat_map(|el| el.text())
            .collect::<Vec<_>>();

        if items.len() != 16 {
            continue;
        }

        let test = element.inner_html();
        let test_items = element.children().collect::<Vec<_>>();

        let value = items[2].to_owned();
        let stroke_count = items[4].trim().parse::<u8>().unwrap().to_owned();
        let meaning = items[5].to_owned();
        let pinyin = items[5].to_owned();

        let record = Radical {
            meaning,
            stroke_count,
            value,
            pinyin,
        };

        list.push(record);
    }

    Ok(list)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_get_radicals_from_wikipedia() {
        let line = get_radicals_from_wikipedia().unwrap();
        assert_eq!(line.len(), 24);
    }
}
