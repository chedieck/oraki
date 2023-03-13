use reqwest::header::USER_AGENT;
use serde_json::Value;
use std::error::Error;
use std::io::prelude::*;
use std::env;
use std::path;
use std::fs::OpenOptions;
use scraper::Html;
use regex::Regex;
use std::fmt;
use genanki_rs::{Field, Model, Note, Template, Error as AnkiError, Deck};
use csv::{ReaderBuilder, WriterBuilder};


const MODEL_ID: i64 = 4173289758;
const DECK_ID: i64 = 8129381912;
const DECK_NAME: &str = "Searched russian words";
const DECK_DESCRIPTION: &str = "Words searched using oraki";
const DEFAULT_EMPTY_VALUE: &str = "-";

fn get_or_crate_data_dir() -> Result<path::PathBuf, Box<dyn Error>>{
        let dir_path = dirs::data_dir().unwrap().join("oraki/");
        if !dir_path.is_dir() {
            std::fs::create_dir(&dir_path)?;
        }
        Ok(dir_path)
}

fn get_main_output_anki_path() -> Result<path::PathBuf, Box<dyn Error>>{
    let dir_path = get_or_crate_data_dir()?;
    let file_path = dir_path.join("output.apkg");
    Ok(file_path)
}

fn get_main_css_path() -> Result<Option<path::PathBuf>, Box<dyn Error>>{
        let dir_path = get_or_crate_data_dir()?;
        let file_path = dir_path.join("main.css");
        if !file_path.is_file() {
            return Ok(None)
        }
        Ok(Some(file_path))
}

fn get_main_csv_path() -> Result<path::PathBuf, Box<dyn Error>>{
        let dir_path = get_or_crate_data_dir()?;
        let file_path = dir_path.join("main.csv");
        if !file_path.is_file() {
            let header_string = "search_term|search_result|title|main_translation|other_translations|overview|context_phrase\n";
            let mut file = std::fs::File::create(&file_path)?;
            file.write_all(header_string.as_bytes())?;
        }
        Ok(file_path)
}

fn make_anki_model() -> Result<Model, Box<AnkiError>> {
    let model = Model::new(
        MODEL_ID,
        "Searched russian word model",
        vec![
        Field::new("search_term"),
        Field::new("search_result"),
        Field::new("title"),
        Field::new("main_translation"),
        Field::new("other_translations"),
        Field::new("overview"),
        Field::new("context_phrase")
        ],
        vec![Template::new("Card 1")
        .qfmt("<span class=\"search_term\">{{search_result}}</span> ({{search_term}})<hr>{{context_phrase}}")
        .afmt(r#"{{FrontSide}}<p>{{title}}</p><hr id="answer"><span class=\"main_translation\">{{main_translation}}</span><br>{{other_translations}}<br><div class=\"overview\">{{overview}}</div>"#)],
    );
    let custom_css_path = get_main_css_path().unwrap();
    match custom_css_path {
        Some(p) => Ok(model.css(p.display())),
        None => Ok(model),
    }
}

const HEADER_USER_AGENT : &str= "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.75 Safari/537.36";

#[derive(Debug, serde::Deserialize)]
struct WordInfo {
    search_term:  String,
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
            self.overview.split('\n').map(|x| x.len()).max().unwrap_or(0),
        ];
        match field_array.iter().max() {
            Some(m) => *m,
            None => 1
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
        format!(
            "({})",
            self.other_translations_joined()
        )
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
            write!(
                f,
                "\nContext: {}",
                c,
            )?;
        }
        Ok(())
    }
}

// first request, get some word to match search term
async fn get_search_term_response_json(input_term: &str)-> Result<Value, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client.get(
        format!(
            "https://api.openrussian.org/suggestions?q={}&dummy=1654996242200&lang=en",
            input_term
        )
    )
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
        return Some(
            String::from(first_term)
            )
    }
    None
}

fn get_response_json_first_form_of(response_json: &Value) -> Option<String> {
    if let Some(first_term) = response_json["result"]["formOf"][0]["source"]["ru"].as_str() {
        return Some(
            String::from(first_term)
            )
    }
    None
}

async fn get_search_result(search_term: &str) -> Result<Option<String>, Box<dyn Error>> {
    let full_res_json = match get_search_term_response_json(search_term)
        .await {
            Ok(res) => res,
            Err(error) => return Err(error)
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
    client.get(
        format!(
            "https://en.openrussian.org/ru/{}",
            search_result
        )
    )
        .header(USER_AGENT, HEADER_USER_AGENT)
        .send()
        .await?
        .text()
        .await
}

fn _get_class_content_from_html(html: Html, class_selector: &str) -> Result<String, Box<dyn Error>> {
    let selector = scraper::Selector::parse(class_selector).unwrap();
    let first_text =  html.select(&selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    match first_text {
        Some(text) => Ok(text),
        None => {
            Err("No element with class=\"{class_selector}\" found.".into())
        }
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


    let text =  overview_html.select(&p_selector)
        .flat_map(|x|
        x.text()) // here it maybe text
        .collect::<Vec<&str>>()
        .join("\n")
        .replace("\n \n", " ");
    Ok(
        text
    )
}

fn get_other_translations_from_translations_text(basics_text: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    let other_translations_text = _get_class_content_from_html(document, ".tl-also").unwrap_or(String::from(DEFAULT_EMPTY_VALUE));

    let re = Regex::new("Also<.*>").unwrap();
    Ok(
        re.replace(
            other_translations_text.as_str(),
            ""
        )
        .split(", ")
        .map(String::from)
        .collect::<Vec<String>>()
    )
}

fn get_main_translation_from_translations_text(basics_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    _get_class_content_from_html(document, ".tl")
}

async fn get_translation_info(search_term: &str, context_phrase: Option<String>) -> Result<WordInfo, Box<dyn Error>> {
    let search_result = match get_search_result(search_term).await {
        Ok(result) =>
            match result {
                Some(result) => result.replace('\'', ""),
                None => return Err(format!("No results found for {search_term}.").into())
            },
        Err(error) => return Err(format!("Couldn't find search result for term `{search_term}` with error:\n{error}").into())
    };
    let response_text = get_search_result_response_text(&search_result).await?;

    let basics_text = get_basics_text_from_response_text(response_text.as_str())?;
    let title = get_title_from_basics_text(basics_text.as_str())?;
    let overview = get_overview_from_basics_text(basics_text.as_str())?;

    let translations_text = get_translations_text_from_response_text(response_text.as_str())?;
    let main_translation = get_main_translation_from_translations_text(translations_text.as_str())?;
    let other_translations = get_other_translations_from_translations_text(translations_text.as_str())?;
    Ok(WordInfo {
        search_term: String::from(search_term),
        search_result,
        title,
        main_translation,
        other_translations,
        overview,
        context_phrase
    })
}

fn create_note_from_result(model: Model, result: csv::StringRecord) -> Result<Note, Box<AnkiError>> {
    let context_phrase = result.get(6).unwrap();
    let search_term = result.get(0).unwrap();
    let context_phrase = context_phrase.replace(search_term, format!("<span class=\"search_term\">{search_term}</span>").as_str());
    Ok(
        Note::new(
            model,
            vec![
            search_term,
            result.get(1).unwrap(),
            result.get(2).unwrap(),
            result.get(3).unwrap(),
            result.get(4).unwrap(),
            result.get(5).unwrap(),
            context_phrase.as_str()
            ]
        ).expect(format!("Could not create note from {}", result.as_slice()).as_str())
    )
}

fn create_deck_from_csv()-> Result<(), Box <dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path(get_main_csv_path()?)?;
    let mut my_deck = Deck::new(
        DECK_ID,
        DECK_NAME,
        DECK_DESCRIPTION,
    );
    for record in reader.records() {
        let result = record?;
        println!("Creating note for {}...", &result[2]);
        let note = create_note_from_result(make_anki_model()?, result)?;
        my_deck.add_note(note);
    }
    my_deck.write_to_file(get_main_output_anki_path().unwrap().to_str().unwrap())?;
    Ok(())
}

fn append_word_info(word_info: &WordInfo) -> Result<(), Box<dyn Error>> {
    let write_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(get_main_csv_path()?)
        .unwrap();
    let mut writer = WriterBuilder::new()
        .delimiter(b'|')
        .from_writer(write_file);
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path(get_main_csv_path()?)?;
    let word_info_string = word_info.to_csv_string_record_slice();
    for record in reader.records() {
        let result = record?;
        if result.as_slice() == word_info_string {
            return Ok(())
        }
    }
    writer.write_record([
        word_info.search_term.as_str(),
        word_info.search_result.as_str(),
        word_info.title.as_str(),
        word_info.main_translation.as_str(),
        word_info.other_translations_joined().as_str(),
        word_info.overview_in_one_line().as_str(),
        word_info.context_phrase.as_ref().unwrap_or(&String::from("")).as_str()
        ])?;
    Ok(())
}

fn help() {
    println!("Usage: oraki [search_query] [...context phrase]");
    println!("-------------------------------------------------------------------");
    println!("[search_query] can be both english or russian;");
    println!("All the subsequent arguments will be the [...context phrase];");
    println!("To compile the anki deck, run `oraki -c` or `oraki --compile`.");
}

#[tokio::main]
async fn main() -> Result<(), Box <dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut context_phrase:  Option<String> = None;
    match args.len() {
        1 => {
            help();
            return Ok(())
        },
        2 => {
            if ["-c", "--compile"].contains(&args[1].as_str()) {
                create_deck_from_csv()?;
                return Ok(())
            }
        },
        _ => context_phrase = Some(args[2..].join(" ")),

    }
    let search_term = args[1].as_str();
    let result_word_info = get_translation_info(search_term, context_phrase).await?;
    append_word_info(&result_word_info)?;
    println!("{result_word_info}");
    Ok(())
}
