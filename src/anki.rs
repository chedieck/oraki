use crate::utils::{get_main_csv_path, get_main_output_anki_path, get_style_css_path};
use csv::ReaderBuilder;
use genanki_rs::{Deck, Error as AnkiError, Field, Model, Note, Template};
use std::error::Error;

const MODEL_ID: i64 = 4173289758;
const DECK_ID: i64 = 8129381912;
const DECK_NAME: &str = "Oraki searched words";
const DECK_DESCRIPTION: &str = "Words searched using oraki";
const Q_FORMAT: &str =
    r#"<span class="search_result">{{search_result}}</span> <hr>{{context_phrase}}"#;
const A_FORMAT: &str = r#"{{FrontSide}}<p class="title">{{title}}</p> <p>({{search_term}})</p><hr id="answer"><span class="main_translation">{{main_translation}}</span><br>{{other_translations}}<br><div class="overview">{{overview}}</div>"#;

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
            Field::new("context_phrase"),
        ],
        vec![Template::new("Card 1").qfmt(Q_FORMAT).afmt(A_FORMAT)],
    );
    let custom_css_path = get_style_css_path().unwrap();
    match custom_css_path {
        Some(p) => Ok(model.css(std::fs::read_to_string(p).unwrap())),
        None => {
            println!("No css found.");
            Ok(model)
        }
    }
}

fn create_note_from_result(
    model: Model,
    result: csv::StringRecord,
) -> Result<Note, Box<AnkiError>> {
    let context_phrase = result.get(6).unwrap();
    let search_term = result.get(0).unwrap();
    let context_phrase = context_phrase.replace(
        search_term,
        format!("<span class=\"search_term\">{search_term}</span>").as_str(),
    );
    Ok(Note::new(
        model,
        vec![
            search_term,
            result.get(1).unwrap(),
            result.get(2).unwrap(),
            result.get(3).unwrap(),
            result.get(4).unwrap(),
            result.get(5).unwrap(),
            context_phrase.as_str(),
        ],
    )
    .expect(format!("Could not create note from {}", result.as_slice()).as_str()))
}

pub fn create_deck_from_csv() -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(b'|')
        .from_path(get_main_csv_path()?)?;
    let mut my_deck = Deck::new(DECK_ID, DECK_NAME, DECK_DESCRIPTION);
    let mut seen_search_results: Vec<String> = vec![];
    for record in reader.records() {
        let result = record?;
        let result_search_result = result[2].to_string();
        if seen_search_results.contains(&result_search_result) {
            println!("Skipping note for {} (already exists)...", &result[2]);
            continue
        }
        println!("Creating note for {}...", &result[2]);
        let note = create_note_from_result(make_anki_model()?, result)?;
        seen_search_results.push(result_search_result);
        my_deck.add_note(note);
    }
    my_deck.write_to_file(get_main_output_anki_path().unwrap().to_str().unwrap())?;
    Ok(())
}
