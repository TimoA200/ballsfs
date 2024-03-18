use super::cli::Commands;
use std::{fs, result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use serde_json::json;
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    daemons: Vec<String>,
}

// use Box<..> as the error type to allow for different kinds of errors
type Result<T> = result::Result<T, Box<dyn std::error::Error>>;

async fn read_config() -> Result<Config> {
    let config_data = tokio::fs::read_to_string("balls-config.json").await?;
    let config: Config = serde_json::from_str(&config_data)?;

    Ok(config)
}

async fn select_random_daemon() -> Result<String> {
    let daemons = read_config().await?.daemons;
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..daemons.len());
    Ok(daemons[index].clone())
}

pub async fn handle_command(command: Commands) -> Result<()> {
    let daemon = select_random_daemon().await?;
    println!("Selected daemon: {}", daemon);

    let mut stream = TcpStream::connect(daemon).await
        .map_err(|e| {
            println!("Failed to connect to deamon 127.0.0.1:8001: {}", e);  
            e
        })?;

    match command {
        Commands::Store { path } => store_file(&path, &mut stream).await,
        Commands::Retrieve { filename } => retrieve_file(&filename, &mut stream).await,
        Commands::List => list_files(&mut stream).await,
        Commands::Delete { filename } => delete_file(&filename, &mut stream).await,
    }
}

async fn store_file(path: &str, stream: &mut TcpStream) -> Result<()>{
    println!("Storing file: {}", path);
    let content = fs::read_to_string(path)?;
    let message = json!({
        "action": "Store",
        "filename": path,
        "content": content
    });

    stream.write_all(&serde_json::to_vec(&message)?).await?;
    stream.write_all(b"\n").await?; // a newline needs to be send, to signal the end of a message
    Ok(())
}

async fn retrieve_file(filename: &str, stream: &mut TcpStream) -> Result<()> {
    println!("Retrieving file: {}", filename);
    let message = json!({
        "action": "Retrieve",
        "filename": filename
    });

    stream.write_all(&serde_json::to_vec(&message)?).await?;
    stream.write_all(b"\n").await?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    fs::write(filename, &buffer)?;
    println!("File content: {}", String::from_utf8(buffer)?);

    Ok(())
}

async fn list_files(stream: &mut TcpStream) -> Result<()> {
    println!("list all files");
    let message = json!({
        "action": "List"
    });

    stream.write_all(&serde_json::to_vec(&message)?).await?;
    stream.write_all(b"\n").await?;

    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    println!("Files: {}", String::from_utf8(buffer)?);

    Ok(())
}

async fn delete_file(filename: &str, stream: &mut TcpStream) -> Result<()> {
    print!("Delete file: {}", filename);
    let message = json!({
        "action": "Delete",
        "filename": filename
    });

    stream.write_all(&serde_json::to_vec(&message)?).await?;
    stream.write_all(b"\n").await?;
    Ok(())
}