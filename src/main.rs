use crate::anki::create_deck_from_csv;
use std::env;
use std::error::Error;

mod anki;
mod or;
mod utils;

fn help() {
    println!("Usage: oraki [option] [search_query]");
    println!("-------------------------------------------------------------------");
    println!("Options:");
    println!("-c, --compile: Compile searched queries into $HOME/.local/share/oraki/output.apkg.");
    println!("               Further arguments will be ignored.");
    println!("-f, --file:    Do multiple searchs, one for each line of the file.");
    println!();
    println!("[search_query] can be both english or russian.");
}

async fn run(search_query: &str, print_full_info: bool) -> Result<(), Box<dyn Error>> {
    let result_translation_info = or::get_translation_info(search_query).await?;
    or::append_translation_info(&result_translation_info)?;
    if print_full_info {
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
