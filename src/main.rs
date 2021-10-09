mod api;
mod customReader;
mod models;
mod refiner;
use crate::api::download_cedict;
use crate::api::get_character_decomposition_from_hanzicraft::get_character_decomposition_from_hanzicraft;
use crate::models::*;
mod utils;
use crate::customReader::custom_reader::BufReader;
use log::{debug, info};
use refiner::refine_records::refine_records;
use refiner::*;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::LineWriter;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;
use utils::*;

#[macro_use]
extern crate log;

pub fn update_descriptor(assets_directory: &Path) -> Result<(), Box<dyn Error>> {
    let mut descriptors = get_descriptors_from_file(&assets_directory.join("descriptor.txt"))?;

    let hsk21 = get_lines_from_file(&assets_directory.join("hsk-version-2-1.txt"))?;
    let hsk22 = get_lines_from_file(&assets_directory.join("hsk-version-2-2.txt"))?;
    let hsk23 = get_lines_from_file(&assets_directory.join("hsk-version-2-3.txt"))?;
    let hsk24 = get_lines_from_file(&assets_directory.join("hsk-version-2-4.txt"))?;
    let hsk25 = get_lines_from_file(&assets_directory.join("hsk-version-2-5.txt"))?;
    let hsk26 = get_lines_from_file(&assets_directory.join("hsk-version-2-6.txt"))?;
    let hsk31 = get_row_from_file(&assets_directory.join("hsk-version-3-1.txt"), 1, "\t")?;
    let hsk32 = get_row_from_file(&assets_directory.join("hsk-version-3-2.txt"), 1, "\t")?;
    let hsk33 = get_row_from_file(&assets_directory.join("hsk-version-3-3.txt"), 1, "\t")?;
    let hsk34 = get_row_from_file(&assets_directory.join("hsk-version-3-4.txt"), 1, "\t")?;
    let hsk35 = get_row_from_file(&assets_directory.join("hsk-version-3-5.txt"), 1, "\t")?;
    let hsk36 = get_row_from_file(&assets_directory.join("hsk-version-3-6.txt"), 1, "\t")?;
    let hsk37 = get_row_from_file(&assets_directory.join("hsk-version-3-7.txt"), 1, "\t")?;

    const attributes: &'static [&'static str] = &[
        "hsk-2-1", "hsk-2-2", "hsk-2-3", "hsk-2-4", "hsk-2-5", "hsk-2-6", "hsk-3-1", "hsk-3-2",
        "hsk-3-3", "hsk-3-4", "hsk-3-5", "hsk-3-6", "hsk-3-7",
    ];

    for (key, descriptor) in descriptors.iter_mut() {
        let mut temp = descriptor.tags.clone().unwrap_or_default();
        let key = &descriptor.simplified;

        let results = vec![
            hsk21.get(key),
            hsk22.get(key),
            hsk23.get(key),
            hsk24.get(key),
            hsk25.get(key),
            hsk26.get(key),
            hsk31.get(key),
            hsk32.get(key),
            hsk33.get(key),
            hsk34.get(key),
            hsk35.get(key),
            hsk36.get(key),
            hsk37.get(key),
        ];

        for (result, attribute) in results.iter().zip(attributes) {
            if let Some(_) = result {
                temp.push(attribute.to_string());
            }
        }

        if !temp.is_empty() {
            descriptor.tags = Some(temp);
        }
    }

    save_descriptors_to_file(descriptors, &assets_directory.join("descriptor1.txt"))?;
    Ok(())
}

pub fn download_cedict_and_save_to_disk(
    cedict_ts_path: &Path,
    assets_directory: &Path,
) -> Result<(), Box<dyn Error>> {
    let cedict_ts_zip_path = &assets_directory.join("cedict_ts.u8.zip");
    let bytes = download_cedict()?;
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(cedict_ts_zip_path)?;
    file.write_all(&bytes)?;
    file.seek(SeekFrom::Start(0))?;

    let mut archive = zip::ZipArchive::new(&file)?;
    let zip_file = archive.by_index(0)?;

    let mut file = File::create(cedict_ts_path)?;
    let bytes: Result<Vec<_>, _> = zip_file.bytes().collect();
    file.write_all(&bytes?)?;

    debug!("Removing find cedict_ts.u8.zip");
    fs::remove_file(cedict_ts_zip_path)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let current_directory = env::current_dir()?;
    let assets_directory = current_directory.join("assets");
    let public_directory = current_directory.join("public");

    loop {
        let mut command = String::from("");
        println!("Commands");
        println!("1. Export characters");
        println!("2. Import stroke count from wikipedia");
        println!("3. Export basic json version of cedict_ts.u8");
        println!("4. Export refined json cedict_ts.u8");
        println!("5. Export refined phrases to separate json files");
        println!("6. Export pinyins");
        println!("7. Export records from cedict when they match given pattern");
        println!("8. Extract meanings");
        println!("9. Exit");
        io::stdin().read_line(&mut command)?;
        command = command.trim().to_owned();

        let cedict_ts_path = &assets_directory.join("cedict_ts.u8");
        let cache_list_path = &current_directory.join("cache-list.json");
        let cache_dict_path = &current_directory.join("cache-dict.json");
        let cache_refined_path = &current_directory.join("cache-refined.json");

        if !cedict_ts_path.exists() {
            debug!("Could not find cedict_ts.u8");
            download_cedict_and_save_to_disk(cedict_ts_path, &assets_directory)?;
        }

        match command.as_str() {
            "1" => {
                let list = try_get_ce_dict_records(cedict_ts_path, cache_list_path)?;

                let single_characters = get_single_characters(&list);
                let output_path = &current_directory.join("stroke-order.txt");
                let file = File::create(output_path)?;
                let mut line_writer = LineWriter::new(file);

                for character in single_characters {
                    let line = format!("{}\n", character);
                    line_writer.write_all(line.as_bytes())?;
                }
            }
            "2" => {
                let output_path = &current_directory.join("stroke-order.txt");
                let lines = import_stroke_order(output_path)?;

                let file = File::create(output_path)?;
                let mut line_writer = LineWriter::new(file);

                for line in lines {
                    line_writer.write_all(line.as_bytes())?;
                }
            }
            "3" => {
                let list = try_get_ce_dict_records(cedict_ts_path, cache_list_path)?;
                let single_characters = get_single_characters(&list);
                let mut processed: HashSet<String> = HashSet::new();
                let mut lines: Vec<String> = Vec::new();
                let file_path = current_directory.join("character-decomposition.txt");

                if file_path.exists() {
                    let reader = BufReader::open(&file_path)?;

                    for line in reader {
                        let line = line?;
                        let radical = line.split(";").next().unwrap().to_string();
                        lines.push(line.to_string());
                        processed.insert(radical);
                    }
                }

                let file = File::create(file_path)?;
                let mut line_writer = LineWriter::new(file);

                for line in lines {
                    line_writer.write_all(line.as_bytes())?;
                }

                for character in single_characters {
                    if processed.contains(&character.to_string()) {
                        continue;
                    }

                    info!("Processing: {}", character);
                    let decomposition =
                        get_character_decomposition_from_hanzicraft(&character.to_string())?;

                    let line = format!("{}\n", decomposition);
                    line_writer.write_all(line.as_bytes())?;
                }
            }
            "4" => {
                let mut refined_records: Vec<Group> = Vec::new();

                if cache_dict_path.exists() {
                    let grouped_records =
                        get_group_ce_records_by_simplified(&Vec::new(), cache_dict_path)?;
                    refined_records = refine_records(
                        grouped_records,
                        &current_directory,
                        &public_directory,
                        &assets_directory,
                    )?;
                } else {
                    let list = try_get_ce_dict_records(cedict_ts_path, cache_list_path)?;
                    let grouped_records =
                        get_group_ce_records_by_simplified(&list, cache_dict_path)?;
                    refined_records = refine_records(
                        grouped_records,
                        &current_directory,
                        &public_directory,
                        &assets_directory,
                    )?;
                }

                let file = File::create(cache_refined_path)?;
                let buffer_writer = BufWriter::new(file);
                serde_json::to_writer_pretty(buffer_writer, &refined_records)?;
            }
            "5" => {
                let list = try_get_ce_dict_records(cedict_ts_path, cache_list_path)?;

                let grouped_records = get_group_ce_records_by_simplified(&list, cache_dict_path)?;
                let refined_records = refine_records(
                    grouped_records,
                    &current_directory,
                    &public_directory,
                    &assets_directory,
                )?;

                for refined_record in refined_records {
                    let file = File::create(
                        public_directory.join(format!("{}.json", &refined_record.simplified)),
                    )?;
                    serde_json::to_writer_pretty(file, &refined_record)?;
                }
            }
            "6" => {
                let file_path = current_directory.join("extracted.txt");
                let lines = BufReader::open(file_path)?;

                for line in lines {
                    let line = line?;
                }
            }
            "7" => {
                let list = try_get_ce_dict_records(cedict_ts_path, cache_list_path)?;

                let file = File::create(current_directory.join("extracted.txt"))?;
                let mut line_writer = LineWriter::new(file);

                for record in &list {
                    let line = &record.line;

                    // if !line.contains("city in") || !line.contains("capital of") {
                    //     continue;
                    // }

                    for meaning in &record.meanings {
                        if !meaning.contains("abbr.") {
                            continue;
                        }

                        let line = format!("{} - {}\n", record.simplified, meaning);
                        line_writer.write_all(line.as_bytes())?;
                    }
                }
            }
            "8" => {
                let mut refined_records: Vec<Group> = Vec::new();

                if cache_refined_path.exists() {
                    refined_records = get_cached_refined_records(cache_refined_path)?;
                } else {
                    let list = try_get_ce_dict_records(cedict_ts_path, cache_list_path)?;

                    let grouped_records =
                        get_group_ce_records_by_simplified(&list, cache_dict_path)?;
                    refined_records = refine_records(
                        grouped_records,
                        &current_directory,
                        &public_directory,
                        &assets_directory,
                    )?;
                }

                let file = File::create(current_directory.join("unmapped.txt"))?;
                let mut line_writer = LineWriter::new(file);

                for group in &refined_records {
                    for detail in &group.details {
                        for meaning in &detail.meanings {
                            let pinyin = &detail.pronunciation.first().unwrap().wade_giles_pinyin;
                            let value = &meaning.value;
                            let contains_abbr = meaning
                                .context
                                .clone()
                                .unwrap_or_default()
                                .contains(&"abbreviation".to_string());

                            if contains_abbr {
                                continue;
                            }

                            if meaning.lexical_item.is_some() {
                                continue;
                            }

                            if value.is_none() {
                                continue;
                            }

                            let line = format!(
                                "{}, {}, {}, , \n",
                                group.simplified,
                                pinyin,
                                value.as_ref().unwrap()
                            );
                            line_writer.write_all(line.as_bytes())?;
                        }
                    }
                }
            }
            "9" => {
                debug!("Quit");
                break;
            }
            _ => {
                info!("Could not find command.");
            }
        }
    }

    //
    // println!("The current directory is {}", current_directory.display());

    // let file_path = assets_directory.join(file_name);

    // let grouped_records =
    //     get_group_ce_records_by_simplified(&list, )?;

    // let file = File::create(public_directory.join("result.json"))?;
    // serde_json::to_writer(file, &result)?;

    Ok(())
}
