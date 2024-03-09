mod cli;
mod commands;

use cli::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    // cli ownership is moved here, making it unusable after match
    match cli.command {
        Some(command) => {
            if let Err(e) = commands::handle_command(command) {
                eprintln!("Error: {}", e);
            }
        },
        None => eprintln!("No command provided"),
    }
}
