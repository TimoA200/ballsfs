mod cli;
mod commands;

use cli::Cli;
use clap::Parser;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // cli ownership is moved here, making it unusable after match
    match cli.command {
        Some(command) => {
            if let Err(e) = commands::handle_command(command).await {
                eprintln!("Error: {}", e);
            }
        },
        None => eprintln!("No command provided"),
    }

    Ok(())
}
