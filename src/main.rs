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
    println!();
    println!("Alternatively, run `oraki -f path/to/file` do the searches by file.");
    println!("Check the documentation for more details on that.");
}

async fn run(search_query: &str, full_info: bool) -> Result<(), Box<dyn Error>> {
    let result_translation_info = or::get_translation_info(search_query).await?;
    or::append_translation_info(&result_translation_info)?;
    if full_info {
        println!("{result_translation_info}")
    } else {
        println!("Getting info for {search_query}...")
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
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
            if ["-f", "--file"].contains(&args[1].as_str()) {
                panic!("Missing file argument.");
            }
        }
        _ => {
            if ["-f", "--file"].contains(&args[1].as_str()) {
                or::append_translation_infos_from_file_name(&args[2]).await?;
                return Ok(());
            }
        }
    };
    run(args[1].as_str(), true).await
}
