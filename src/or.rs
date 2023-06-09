use crate::utils::get_main_csv_path;
use indicatif::{ProgressBar, ProgressStyle};
use csv::{ReaderBuilder, WriterBuilder};
use regex::Regex;
use reqwest::header::USER_AGENT;
use scraper::Html;
use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};

const HEADER_USER_AGENT : &str= "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.75 Safari/537.36";
const DEFAULT_EMPTY_VALUE: &str = "-";

#[derive(Debug, serde::Deserialize)]
pub struct TranslationInfo {
    search_query: String,
    search_result: String,
    context_phrase: Option<String>,
    context_phrase_translation: Option<String>,
    title: String,
    main_translation: String,
    other_translations: Vec<String>,
    overview: String,
}

impl TranslationInfo {
    fn get_n_of_diacritics(&self, field_str: &str) -> usize {
        let mut i = 0;
        for char in field_str.chars() {
            if char == '\u{301}' {
                i += 1
            }
        }
        i
    }
    fn max_field_len(&self) -> usize {
        let field_array: [usize; 4] = [
            self.title.chars().count(),
            self.main_translation.chars().count(),
            self.other_translations_concatenated().chars().count(),
            self.overview
                .split('\n')
                .map(|x| x.chars().count())
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

    fn overview_centered_with_walls(&self) -> String {
        let width = self.max_field_len() + 2;
        return self
            .overview
            .split('\n')
            .map(|x| {
                format!(
                    "\u{2502}{:^width$}\u{2502}",
                    x,
                    width = width + self.get_n_of_diacritics(x)
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
    }

    fn from_csv_string_record_slice(record: csv::StringRecord) -> Result<Self, Box<dyn Error>> {
        let other_translations = record
            .get(4)
            .unwrap()
            .to_string()
            .split(", ")
            .map(|s| s.to_string())
            .collect();
        let overview = record.get(5).unwrap().replace("; ", "\n");
        let context_phrase = match record.get(6).unwrap() {
            "" => None,
            s => Some(s.to_string()),
        };
        let context_phrase_translation = match record.get(7).unwrap() {
            "" => None,
            s => Some(s.to_string()),
        };

        Ok(Self {
            search_query: record.get(0).unwrap().to_string(),
            search_result: record.get(1).unwrap().to_string(),
            title: record.get(2).unwrap().to_string(),
            main_translation: record.get(3).unwrap().to_string(),
            other_translations,
            overview,
            context_phrase,
            context_phrase_translation,
        })
    }

    fn to_csv_string_record_slice(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}",
            self.search_query,
            self.search_result,
            self.title,
            self.main_translation,
            self.other_translations_joined(),
            self.overview_in_one_line(),
            self.context_phrase.as_ref().unwrap_or(&String::from("")),
            self.context_phrase_translation
                .as_ref()
                .unwrap_or(&String::from(""))
        )
    }

    fn other_translations_concatenated(&self) -> String {
        format!("({})", self.other_translations_joined())
    }
}

impl fmt::Display for TranslationInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = self.max_field_len() + 2;
        let separator = "\u{2500}".repeat(width);
        let title_width = width + self.get_n_of_diacritics(&self.title);

        write!(
            f,
            "\u{0250c}{}\u{2510}\n\u{2502}{:^title_width$}\u{2502}\n\u{2502}{:^width$}\u{2502}\n\u{2502}{:^width$}\u{2502}\n\u{2502}{}\u{2502}\n{}\n\u{2514}{}\u{2518}",
            separator,
            self.title.as_str(),
            self.main_translation.as_str(),
            self.other_translations_concatenated(),
            separator,
            self.overview_centered_with_walls(),
            separator,
            width=width,
            title_width=title_width,
        )?;
        if let Some(c) = &self.context_phrase {
            writeln!(f)?;
            write!(f, "\n{}", c)?;
        }
        if let Some(ct) = &self.context_phrase_translation {
            write!(f, "\n{}", ct,)?;
        }
        Ok(())
    }
}

// first request, get some word to match search term
async fn get_search_query_response_json(input_term: &str) -> Result<Value, Box<dyn Error>> {
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
    let result_words = response_json["result"]["words"].as_array().unwrap();
    for word in result_words {
        let tls = word["word"]["tls"].as_array().unwrap();
        if tls.is_empty() {
            continue
        }
        if let Some(result) = word["word"]["ru"].as_str() {
            return Some(String::from(result));
        }
    }
    None
}

async fn get_search_result(search_query: &str) -> Result<Option<String>, Box<dyn Error>> {
    let full_res_json = match get_search_query_response_json(search_query).await {
        Ok(res) => res,
        Err(error) => return Err(error),
    };

    if let Some(first_term) = get_response_json_first_term(&full_res_json) {
        Ok(Some(first_term.replace('\'', "")))
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

fn _get_class_content_from_html(html: Html, selector_str: &str) -> Result<String, Box<dyn Error>> {
    let selector = scraper::Selector::parse(selector_str).unwrap();
    let first_text = html
        .select(&selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    match first_text {
        Some(text) => Ok(text),
        None => Err(format!("No element for \"{selector_str}\" found.").into()),
    }
}

fn get_selector_text_from_bigger_text(
    selector_str: &str,
    bigger_text: &str,
) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_document(bigger_text);
    _get_class_content_from_html(document, selector_str)
}
fn get_first_sentence_and_translation_from_response_text(
    response_text: &str,
) -> Result<(String, String), Box<dyn Error>> {
    let sentences_text = get_selector_text_from_bigger_text("ul.sentences > li", response_text)?;
    let ru_html_text = get_selector_text_from_bigger_text(".ru", &sentences_text)?;
    let ru_html = Html::parse_fragment(ru_html_text.as_str());
    let span_a_selector = scraper::Selector::parse("a,span").unwrap();
    let ru_sentence = ru_html
        .select(&span_a_selector)
        .flat_map(|x| x.text()) // here it maybe text
        .collect::<Vec<&str>>()
        .join("");

    let en_sentence = get_selector_text_from_bigger_text(".tl span", &sentences_text)?;

    Ok((ru_sentence, en_sentence))
}

fn get_overview_from_basics_text(basics_text: &str) -> Result<String, Box<dyn Error>> {
    let overview_html_text = get_selector_text_from_bigger_text(".overview", basics_text)?;

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

// second argument on return refers if result already existed
pub async fn get_translation_info(search_query: &str) -> Result<(TranslationInfo, bool), Box<dyn Error>> {
    if let Ok(Some(translation_info)) = get_cached_translation_info_for_query(search_query) {
        return Ok((translation_info, true));
    }
    let search_result = match get_search_result(search_query).await {
        Ok(result) => match result {
            Some(result) => result,
            None => return Err(format!("No results found for {search_query}.").into()),
        },
        Err(error) => {
            return Err(format!(
                "Couldn't find search result for term `{search_query}` with error:\n{error}"
            )
            .into())
        }
    };
    let response_text = get_search_result_response_text(&search_result).await?;
    let basics_text = get_selector_text_from_bigger_text(".basics", response_text.as_str())?;

    // get context phrase
    let mut context_phrase_translation = None;
    let mut context_phrase = None;
    let first_sentence_result =
        get_first_sentence_and_translation_from_response_text(response_text.as_str());
    if let Ok(first_sentence) = first_sentence_result {
        context_phrase = Some(first_sentence.0);
        context_phrase_translation = Some(first_sentence.1);
    }

    let title = get_selector_text_from_bigger_text(".bare span", basics_text.as_str())?;
    let overview = get_overview_from_basics_text(basics_text.as_str())?;

    let translations_text =
        get_selector_text_from_bigger_text(".translations", response_text.as_str())?;
    let main_translation = get_selector_text_from_bigger_text(".tl", translations_text.as_str())?;
    let other_translations =
        get_other_translations_from_translations_text(translations_text.as_str())?;

    Ok((TranslationInfo {
        search_query: String::from(search_query),
        search_result,
        title,
        main_translation,
        other_translations,
        overview,
        context_phrase,
        context_phrase_translation,
    }, false))
}

#[derive(Debug)]
pub struct FileResult {
    pub failed_results: Vec<String>,
    pub fetched_results: Vec<String>,
    pub existent_results: Vec<String>,
    pub n_total: u64,
}

pub async fn append_translation_infos_from_file_name(
    file_name: &str,
) -> Result<FileResult, Box<dyn Error>> {
    let file = std::fs::File::open(file_name)?;
    let file = BufReader::new(file);
    let file_lines = file.lines().collect::<Vec<Result<String, std::io::Error>>>();

    let n_lines = file_lines.len() as u64;
    // progress bar
    let pb = ProgressBar::new(n_lines);
    pb.set_style(
        ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")?
        .progress_chars("#>-"),
    );

    let mut failed_results: Vec<String> = vec![];
    let mut existent_results: Vec<String> = vec![];
    let mut fetched_results: Vec<String> = vec![];
    for result in file_lines.into_iter() {
        pb.inc(1);
        let line = result?;
        let mut line_words = line.split_whitespace();
        let Some(search_query) = line_words.next() else {
            continue
        };
        if search_query.starts_with('#') {
            continue;
        }

        if let Ok(already_existed) = super::run(search_query, false).await {
            if already_existed {
                pb.println(format!("Got existent info for {search_query}..."));
                existent_results.push(search_query.to_string());
            } else {
                pb.println(format!("Got new info for {search_query}..."));
                fetched_results.push(search_query.to_string());
            }
        }
        else {
            pb.println(format!("Failed getting info for {search_query}."));
            failed_results.push(search_query.to_string());
            continue
        };
    }
    Ok(FileResult {
        failed_results,
        fetched_results,
        existent_results,
        n_total: n_lines 
    })
}

pub fn get_cached_translation_info_for_query(
    search_query: &str,
) -> Result<Option<TranslationInfo>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path(get_main_csv_path()?)?;
    for record in reader.records() {
        let result = record?;
        if result.get(0) == Some(search_query) {
            return Ok(Some(TranslationInfo::from_csv_string_record_slice(result)?));
        }
    }
    Ok(None)
}

pub fn append_translation_info(translation_info: &TranslationInfo) -> Result<(), Box<dyn Error>> {
    let write_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(get_main_csv_path()?)
        .unwrap();
    let mut writer = WriterBuilder::new().delimiter(b'|').from_writer(write_file);
    if get_cached_translation_info_for_query(&translation_info.search_query)?.is_some() {
        return Ok(());
    }
    writer.write_record([
        translation_info.search_query.as_str(),
        translation_info.search_result.as_str(),
        translation_info.title.as_str(),
        translation_info.main_translation.as_str(),
        translation_info.other_translations_joined().as_str(),
        translation_info.overview_in_one_line().as_str(),
        translation_info
            .context_phrase
            .as_ref()
            .unwrap_or(&String::from(""))
            .as_str(),
        translation_info
            .context_phrase_translation
            .as_ref()
            .unwrap_or(&String::from(""))
            .as_str(),
    ])?;
    Ok(())
}
