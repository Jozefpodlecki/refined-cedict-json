use crate::CERecord;
use crate::PinyinMap;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::LineWriter;
use std::io::{prelude::*, BufReader};
use scraper::Html;
use scraper::Selector;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use std::error::Error;
use urlencoding::encode;

pub fn get_stroke_count(character: &str) -> Result<u8, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client = reqwest::blocking::Client::builder().build().unwrap();
    }

    let encoded = encode(character);
    let base_url = "http://www.strokeorder.info/mandarin.php?q=";
    let url = format!("{} {}", base_url, encoded);

    let response: reqwest::blocking::Response = CLIENT
        .get(url)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36")
        .send()
        .unwrap();

    let body_response = response.text()?;
    let parsed_html = Html::parse_document(&body_response);

    let selector = &Selector::parse(".container b").unwrap();
    let mut stroke_count = 0;
    for element in parsed_html.select(&selector) {
        let text: String = element.text().collect();
        if text.contains("Strokes：") {
            let mut stroke_count_str = element.next_sibling().unwrap().value().as_text().unwrap().to_string();
            stroke_count_str = stroke_count_str.trim().to_string();
          
            stroke_count = stroke_count_str.parse::<u8>()?;
            println!(" {}", stroke_count);
        }
    }

    Ok(stroke_count)
}