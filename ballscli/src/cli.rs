use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version = "1.0", about = "This DFS has some big balls")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Store a file on the DFS
    Store {
        /// path to file
        path: String,
    },
    /// Retrieve a file from the DFS
    Retrieve {
        /// filename to retrieve
        filename: String,
    }
}