use crate::anki::create_deck_from_csv;
use std::env;
use std::error::Error;

mod anki;
mod or;
mod utils;

fn help() {
    println!("Usage: oraki [search_query] [...context phrase]");
    println!("-------------------------------------------------------------------");
    println!("[search_query] can be both english or russian;");
    println!("All the subsequent arguments will be the [...context phrase];");
    println!("To compile the anki deck, run `oraki -c` or `oraki --compile`.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut context_phrase: Option<String> = None;
    match args.len() {
        1 => {
            help();
            return Ok(());
        }
        2 => {
            if ["-c", "--compile"].contains(&args[1].as_str()) {
                create_deck_from_csv()?;
                return Ok(());
            }
        }
        _ => context_phrase = Some(args[2..].join(" ")),
    }
    let search_term = args[1].as_str();
    let result_word_info = or::get_translation_info(search_term, context_phrase).await?;
    or::append_word_info(&result_word_info)?;
    println!("{result_word_info}");
    Ok(())
}
