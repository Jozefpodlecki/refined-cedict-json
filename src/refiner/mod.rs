pub mod refine_meaning_record;
pub mod refine_records;
pub mod to_pinyin;
use crate::models::*;
use crate::utils::*;
use log::{debug, info};
use std::error::Error;
use std::path::Path;

#[cfg(test)]
#[path = "./enhancer_test.rs"]
mod test;

pub fn get_cached_refined_records(cache_dict_path: &Path) -> Result<Vec<Group>, Box<dyn Error>> {
    let bytes = &read_file_bytes(cache_dict_path)?;
    let dict = serde_json::from_slice(bytes)?;
    Ok(dict)
}
