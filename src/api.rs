use crate::CERecord;
use crate::PinyinMap;
use bytes::Bytes;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
use scraper::Html;
use scraper::Selector;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use serde_json::Value;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::LineWriter;
use std::io::{prelude::*, BufReader};
use urlencoding::encode;

static user_agent: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.4577.82 Safari/537.36";

pub fn download_cedict() -> Result<Bytes, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
    }

    const URL: &str = "https://www.mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.zip";
    let response = CLIENT
        .get(URL)
        .header(reqwest::header::USER_AGENT, user_agent)
        .send()?;

    let bytes = response.bytes()?;

    Ok(bytes)
}

pub fn get_info_from_writtenchinese(pinyin: &str) -> Result<Option<String>, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
    }

    let mut form_data = HashMap::new();
    form_data.insert("searchKey", pinyin);

    const url: &str = "https://dictionary.writtenchinese.com/ajaxsearch/simsearch.action";
    let response = CLIENT
        .post(url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .form(&form_data)
        .send()?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let data = &response.bytes()?;

    let value: Value = serde_json::from_slice(data)?;
    let mut result: Option<String> = None;

    match value.get("signPinyinShort") {
        Some(records) => {
            if !records.is_null() {
                let record = records.as_array().unwrap().first().unwrap();
                result = record["tone"].as_str().map(|pr| pr.to_owned());
            }
        }
        None => {}
    }

    if result.is_none() {
        match value.get("mdbgPinyin") {
            Some(records) => {
                if !records.is_null() {
                    let record = records.as_array().unwrap().first().unwrap();
                    result = record["tone"].as_str().map(|pr| pr.to_owned());
                }
            }
            None => {}
        }
    }

    Ok(result)
}

pub fn get_stroke_count_from_wiktionary(character: &str) -> Result<Option<u8>, Box<dyn Error>> {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(r"(\d+)\sstrokes?",).unwrap();
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
    }

    let encoded = encode(character);
    const BASE_URL: &str = "https://en.wiktionary.org/wiki/";
    let url = format!("{}{}", BASE_URL, encoded);

    let response: reqwest::blocking::Response = CLIENT
        .get(url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .send()?;

    let body_response = response.text()?;

    let captures = REGEX.captures(&body_response);

    if captures.is_none() {
        return Ok(None);
    }

    let temp = captures.unwrap();
    let mut iter = temp.iter();
    iter.next();
    let stroke_count_str = iter.next().unwrap().unwrap().as_str();
    let stroke_count = stroke_count_str.parse::<u8>()?;

    Ok(Some(stroke_count))
}

pub fn get_stroke_count_from_nihongo(character: &str) -> Result<u8, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
    }

    let encoded = encode(character);
    let base_url = "https://www.nihongo-pro.com/kanji-pal/kanji/";
    let url = format!("{}{}", base_url, encoded);

    let response: reqwest::blocking::Response = CLIENT
        .get(url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .send()
        .unwrap();

    let body_response = response.text()?;
    let parsed_html = Html::parse_document(&body_response);

    let selector = &Selector::parse(".container b").unwrap();
    let mut stroke_count = 0;

    for element in parsed_html.select(&selector) {
        let text: String = element.text().collect();
        if text.contains("Strokes：") {
            let mut next_sibling = element.next_sibling().unwrap();
            let mut stroke_count_str = next_sibling.value().as_text().unwrap().to_string();

            let is_numeric = stroke_count_str.chars().all(char::is_numeric);

            if !is_numeric {
                next_sibling = next_sibling.next_sibling().unwrap();
                stroke_count_str = next_sibling.value().as_text().unwrap().to_string();
            }

            stroke_count_str = stroke_count_str.trim().to_string();
            stroke_count = stroke_count_str.parse::<u8>()?;
            break;
        }
    }

    Ok(stroke_count)
}

pub fn get_stroke_count_from_strokeorder(character: &str) -> Result<u8, Box<dyn Error>> {
    lazy_static! {
        static ref CLIENT: reqwest::blocking::Client =
            reqwest::blocking::Client::builder().build().unwrap();
    }

    let encoded = encode(character);
    let base_url = "http://www.strokeorder.info/mandarin.php?q=";
    let url = format!("{}{}", base_url, encoded);

    let response: reqwest::blocking::Response = CLIENT
        .get(url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .send()
        .unwrap();

    let body_response = response.text()?;
    let parsed_html = Html::parse_document(&body_response);

    let selector = &Selector::parse(".container b").unwrap();
    let mut stroke_count = 0;

    for element in parsed_html.select(&selector) {
        let text: String = element.text().collect();
        if text.contains("Strokes：") {
            let mut next_sibling = element.next_sibling().unwrap();
            let mut stroke_count_str = next_sibling.value().as_text().unwrap().to_string();

            let is_numeric = stroke_count_str.chars().all(char::is_numeric);

            if !is_numeric {
                next_sibling = next_sibling.next_sibling().unwrap();
                stroke_count_str = next_sibling.value().as_text().unwrap().to_string();
            }

            stroke_count_str = stroke_count_str.trim().to_string();
            stroke_count = stroke_count_str.parse::<u8>()?;
            break;
        }
    }

    Ok(stroke_count)
}
