extern crate csv;
use crate::prelude::*;
use csv::{Error, StringRecord};

pub fn get_string_records(path: &Path) -> Result<Vec<StringRecord>> {
    let text = get_text(path);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut res: Vec<StringRecord> = Vec::new();
    for record in reader.records() {
        let record = record?;
        res.push(record);
    }
    Ok(res)
}
