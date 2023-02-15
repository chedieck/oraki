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
    title: String,
    main_translation: String, // WIP
    other_translations: Vec<String>, // WIP
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

async fn get_basics_text_from_response_text(response_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_document(response_text);
    let basics_selector = scraper::Selector::parse(".basics").unwrap();
    let first_basics_text =  document.select(&basics_selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    match first_basics_text {
        Some(text) => Ok(text),
        None => panic!("No element with class=\"basics\" found.")
    }
}

async fn get_translations_text_from_response_text(response_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_document(response_text);
    let translations_selector = scraper::Selector::parse(".translations").unwrap();
    let first_basics_text =  document.select(&translations_selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    match first_basics_text {
        Some(text) => Ok(text),
        None => panic!("No element with class=\"translations\" found.")
    }
}

async fn get_title_from_basics_text(basics_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    let bare_selector = scraper::Selector::parse(".bare").unwrap();
    let first_bare_text = document.select(&bare_selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    match first_bare_text {
        Some(text) => Ok(text),
        None => panic!("No element with class=\"bare\" found.")
    }
}

async fn get_overview_from_basics_text(basics_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    let overview_selector = scraper::Selector::parse(".overview").unwrap();
    let overview_html = document
        .select(&overview_selector)
        .map(|x| Html::parse_fragment(x.inner_html().as_str())) // here it maybe text
        .next();
    let overview_html = match overview_html {
        Some(html) => html,
        None => panic!("No element with class=\"overview\" found.")
    };
    let p_selector = scraper::Selector::parse("p").unwrap();
    let p_vec = overview_html.select(&p_selector)
        .map(|x| x.inner_html()) // here it maybe text
        .collect::<Vec<String>>();
    Ok(p_vec.join("\n"))
}

async fn get_other_translations_from_translations_text(basics_text: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    let other_translations_selector = scraper::Selector::parse(".tl-also").unwrap();
    let other_translations_text = document
        .select(&other_translations_selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    let other_translations_text = match other_translations_text {
        Some(html) => html,
        None => panic!("No element with class=\"overview\" found.")
    };
    Ok(
        other_translations_text
        .split(", ")
        .map(String::from)
        .collect::<Vec<String>>()
    )
}

async fn get_main_translation_from_translations_text(basics_text: &str) -> Result<String, Box<dyn Error>> {
    let document = Html::parse_fragment(basics_text);
    let tl_selector = scraper::Selector::parse(".tl").unwrap();
    let first_tl_text = document.select(&tl_selector)
        .map(|x| x.inner_html()) // here it maybe text
        .next();
    match first_tl_text {
        Some(text) => Ok(text),
        None => panic!("No element with class=\"bare\" found.")
    }
}

async fn get_translation_info(search_term: &str) -> Result<WordInfo, Box<dyn Error>> {
    let search_result = match get_search_result(search_term).await {
        Ok(result) =>
            match result {
                Some(result) => result,
                None => panic!("No results found for {search_term}.")
            },
        Err(error) => panic!("Couldn't find search result for term `{search_term}` with error:\n{error}")
    };
    let response_text = get_search_result_response_text(&search_result).await?;

    let basics_text = get_basics_text_from_response_text(response_text.as_str()).await?;
    let title = get_title_from_basics_text(basics_text.as_str()).await?;
    let overview = get_overview_from_basics_text(basics_text.as_str()).await?;

    let translations_text = get_translations_text_from_response_text(response_text.as_str()).await?;
    let main_translation = get_main_translation_from_translations_text(translations_text.as_str()).await?;
    let other_translations = get_other_translations_from_translations_text(translations_text.as_str()).await?;
    Ok(WordInfo {
        search_term: String::from(search_term),
        search_result,
        title,
        main_translation,
        other_translations,
        overview,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box <dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.is_empty() { 
        println!("No search string provided.");
        return Ok(())
    };
    let search_term = args[1].as_str();
    let search_result = get_translation_info(search_term).await?;
    println!("Search result: {search_result:#?}");
    Ok(())
}

