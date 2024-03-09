use super::cli::Commands;
use std::result;

// use Box<..> as the error type to allow for different kinds of errors
type Result<T> = result::Result<T, Box<dyn std::error::Error>>;

pub fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Store { path } => store_file(&path),
        Commands::Retrieve { filename } => retrieve_file(&filename)
    }
}

fn store_file(path: &str) -> Result<()>{
    println!("Storing file: {}", path);
    // implement storage logic here
    Ok(())
}

fn retrieve_file(filename: &str) -> Result<()> {
    println!("Retrieving file: {}", filename);
    // implement retrieve logic here
    Ok(())
}