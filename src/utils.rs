use std::error::Error;
use std::io::prelude::*;
use std::path;

pub fn get_or_crate_data_dir() -> Result<path::PathBuf, Box<dyn Error>> {
    let dir_path = dirs::data_dir().unwrap().join("oraki/");
    if !dir_path.is_dir() {
        std::fs::create_dir(&dir_path)?;
    }
    Ok(dir_path)
}

pub fn get_main_output_anki_path() -> Result<path::PathBuf, Box<dyn Error>> {
    let dir_path = get_or_crate_data_dir()?;
    let file_path = dir_path.join("output.apkg");
    Ok(file_path)
}

pub fn get_style_css_path() -> Result<Option<path::PathBuf>, Box<dyn Error>> {
    let dir_path = get_or_crate_data_dir()?;
    let file_path = dir_path.join("style.css");
    if !file_path.is_file() {
        return Ok(None);
    }
    Ok(Some(file_path))
}

pub fn get_main_csv_path() -> Result<path::PathBuf, Box<dyn Error>> {
    let dir_path = get_or_crate_data_dir()?;
    let file_path = dir_path.join("main.csv");
    if !file_path.is_file() {
        let header_string = "search_query|search_result|title|main_translation|other_translations|overview|context_phrase\n";
        let mut file = std::fs::File::create(&file_path)?;
        file.write_all(header_string.as_bytes())?;
    }
    Ok(file_path)
}
