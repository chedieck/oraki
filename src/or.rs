use crate::utils::get_main_csv_path;
use csv::{ReaderBuilder, WriterBuilder};
use regex::Regex;
use reqwest::header::USER_AGENT;
use scraper::Html;
use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::fs::OpenOptions;

const HEADER_USER_AGENT : &str= "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.75 Safari/537.36";
const DEFAULT_EMPTY_VALUE: &str = "-";

#[derive(Debug, serde::Deserialize)]
pub struct WordInfo {
    search_term: String,
    search_result: String,
    context_phrase: Option<String>,
    title: String,
    main_translation: String,
    other_translations: Vec<String>,
    overview: String,
}

impl WordInfo {
    fn max_field_len(&self) -> usize {
        let field_array: [usize; 4] = [
            self.title.len(),
            self.main_translation.len(),
            self.other_translations_concatenated().len(),
            self.overview
                .split('\n')
                .map(|x| x.len())
                .max()
                .unwrap_or(0),
        ];
        match field_array.iter().max() {
            Some(m) => *m,
            None => 1,
        }
    }

    fn other_translations_joined(&self) -> String {
        self.other_translations.join(", ")
    }

    fn overview_in_one_line(&self) -> String {
        self.overview.replace('\n', "; ")
    }

    fn to_csv_string_record_slice(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            self.search_term,
            self.search_result,
            self.title,
            self.main_translation,
            self.other_translations_joined(),
            self.overview_in_one_line(),
            self.context_phrase.as_ref().unwrap_or(&String::from(""))
        )
    }

    fn other_translations_concatenated(&self) -> String {
        format!("({})", self.other_translations_joined())
    }
}

impl fmt::Display for WordInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}",
            self.title.as_str(),
            self.main_translation.as_str(),
            self.other_translations_concatenated(),
            "-".repeat(self.max_field_len()),
            self.overview,
        )?;
        if let Some(c) = &self.context_phrase {
            write!(f, "\nContext: {}", c,)?;
        }
        Ok(())
    }
}

// first request, get some word to match search term
async fn get_search_term_response_json(input_term: &str) -> Result<Value, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://api.openrussian.org/suggestions?q={}&dummy=1654996242200&lang=en",
            input_term
        ))
        .header(USER_AGENT, HEADER_USER_AGENT)
        .send()
        .await?
        .text()
        .await?;
    let ret: Value = serde_json::from_str(response.as_str())?;
    Ok(ret)
}

fn get_response_json_first_term(response_json: &Value) -> Option<String> {
    if let Some(first_term) = response_json["result"]["words"][0]["ru"].as_str() {
        return Some(String::from(first_term));
    }
    None
}

fn get_response_json_first_form_of(response_json: &Value) -> Option<String> {
    if let Some(first_term) = response_json["result"]["formOf"][0]["source"]["ru"].as_str() {
        return Some(String::from(first_term));
    }
    None
}

async fn get_search_result(search_term: &str) -> Result<Option<String>, Box<dyn Error>> {
    let full_res_json = match get_search_term_response_json(search_term).await {
        Ok(res) => res,
        Err(error) => return Err(error),
    };

    if let Some(first_term) = get_response_json_first_term(&full_res_json) {
        Ok(Some(first_term))
    } else if let Some(first_form_of) = get_response_json_first_form_of(&full_res_json) {
        Ok(Some(first_form_of))
    } else {
        Ok(None)
    }
}

// second request, get detailes of matched word
async fn get_search_result_response_text(search_result: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .get(format!("https://en.openrussian.org/ru/{}", search_result))
        .header(USER_AGENT, HEADER_USER_AGENT)
        .send()
        .await?
        .text()
        .await
}

fn _get_class_content_from_html(
    html: Html,
    class_selector: &str,
) -> Result<String, Box<dyn Error>> {
    let selector = scraper::Selector::parse(class_selector).unwrap();
    let first_text = html
        .select(&selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    match first_text {
        Some(text) => Ok(text),
        None => Err("No element with class=\"{class_selector}\" found.".into()),
    }
}

fn get_basics_text_from_response_text(response_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_document(response_text);
    _get_class_content_from_html(document, ".basics")
}

fn get_translations_text_from_response_text(response_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_document(response_text);
    _get_class_content_from_html(document, ".translations")
}

fn get_title_from_basics_text(basics_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    _get_class_content_from_html(document, ".bare span")
}

fn get_overview_from_basics_text(basics_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    let overview_html_text = _get_class_content_from_html(document, ".overview")?;

    let overview_html = Html::parse_fragment(overview_html_text.as_str());
    let p_selector = scraper::Selector::parse("p").unwrap();

    let text = overview_html
        .select(&p_selector)
        .flat_map(|x| x.text()) // here it maybe text
        .collect::<Vec<&str>>()
        .join("\n")
        .replace("\n \n", " ");
    Ok(text)
}

fn get_other_translations_from_translations_text(
    basics_text: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    let other_translations_text = _get_class_content_from_html(document, ".tl-also")
        .unwrap_or(String::from(DEFAULT_EMPTY_VALUE));

    let re = Regex::new("Also<.*>").unwrap();
    Ok(re
        .replace(other_translations_text.as_str(), "")
        .split(", ")
        .map(String::from)
        .collect::<Vec<String>>())
}

fn get_main_translation_from_translations_text(
    basics_text: &str,
) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    _get_class_content_from_html(document, ".tl")
}

pub async fn get_translation_info(
    search_term: &str,
    context_phrase: Option<String>,
) -> Result<WordInfo, Box<dyn Error>> {
    let search_result = match get_search_result(search_term).await {
        Ok(result) => match result {
            Some(result) => result.replace('\'', ""),
            None => return Err(format!("No results found for {search_term}.").into()),
        },
        Err(error) => {
            return Err(format!(
                "Couldn't find search result for term `{search_term}` with error:\n{error}"
            )
            .into())
        }
    };
    let response_text = get_search_result_response_text(&search_result).await?;

    let basics_text = get_basics_text_from_response_text(response_text.as_str())?;
    let title = get_title_from_basics_text(basics_text.as_str())?;
    let overview = get_overview_from_basics_text(basics_text.as_str())?;

    let translations_text = get_translations_text_from_response_text(response_text.as_str())?;
    let main_translation = get_main_translation_from_translations_text(translations_text.as_str())?;
    let other_translations =
        get_other_translations_from_translations_text(translations_text.as_str())?;
    Ok(WordInfo {
        search_term: String::from(search_term),
        search_result,
        title,
        main_translation,
        other_translations,
        overview,
        context_phrase,
    })
}

pub fn append_word_info(word_info: &WordInfo) -> Result<(), Box<dyn Error>> {
    let write_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(get_main_csv_path()?)
        .unwrap();
    let mut writer = WriterBuilder::new().delimiter(b'|').from_writer(write_file);
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path(get_main_csv_path()?)?;
    let word_info_string = word_info.to_csv_string_record_slice();
    for record in reader.records() {
        let result = record?;
        if result.as_slice() == word_info_string {
            return Ok(());
        }
    }
    writer.write_record([
        word_info.search_term.as_str(),
        word_info.search_result.as_str(),
        word_info.title.as_str(),
        word_info.main_translation.as_str(),
        word_info.other_translations_joined().as_str(),
        word_info.overview_in_one_line().as_str(),
        word_info
            .context_phrase
            .as_ref()
            .unwrap_or(&String::from(""))
            .as_str(),
    ])?;
    Ok(())
}