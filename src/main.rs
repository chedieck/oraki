use reqwest::header::USER_AGENT;
use serde_json::Value;
use std::error::Error;
use std::env;
use scraper::Html;

const HEADER_USER_AGENT : &str= "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.75 Safari/537.36";

#[derive(Debug)]
struct WordInfo {
    search_term:  String,
    search_result: String,
    main_translation: String,
    other_translations: Vec<String>,
    overview: String,
}

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

async fn get_search_result_response_text(search_result: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client.get(
        format!(
            "https://en.openrussian.org/ru/{}",
            search_result
        )
    )
        .header(USER_AGENT, HEADER_USER_AGENT)
        .send()
        .await?
        .text()
        .await?;
    //let ret = Html::parse_document(response.as_str());
    //Ok(ret)
}
/*
async fn get_translation_info(search_term: &str) -> Result<WordInfo, Box<dyn Error>> {
    let search_result = match get_search_result(search_term).await {
        Ok(result) =>
            match result {
                Some(result) => result,
                None => panic!("No results found for {search_term}.")
            },
        Err(error) => panic!("Couldn't find search result for term `{search_term}` with error:\n{error}")
    };
    let response_text = get_search_result_response_text(&search_result);
    let main_translation = get_main_translation_from_response_text(&response_text);
    let other_translations = get_other_translations_from_response_text(&response_text);
    let overview = get_overview_from_response_text(&response_text);
}
*/

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.is_empty() { return };
    let search_term = args[1].as_str();
    search_result = get_translation_info(search_term).unwrap();
    println!("Search result: {search_result:#?}")
}

