use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use thiserror::Error;
use serde_json::json;

#[derive(Error, Debug)]
enum ServerError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("unknown command")]
    UnkonwCommand,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action")]
enum Command {
    Store { filename: String, content: String },
    Retrieve { filename: String },
    List,
    Delete { filename: String },
    ReplicateStore { filename: String, content: String},
    ReplicateDelete { filename: String},
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    peers: Vec<String>,
}



#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let listener = TcpListener::bind("0.0.0.0:8001").await?;
    println!("Deamon listening on 0.0.0.0:8001!");

    // create ball if it doesn exist
    fs::create_dir_all("./ball").await?;

    let config = read_config().await?;
    println!("Peers: {:?}", config.peers);

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn read_config() -> Result<Config, ServerError> {
    let config_data = fs::read_to_string("balls-config.json").await?;
    let config: Config = serde_json::from_str(&config_data)?;
    Ok(config)
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), ServerError> {
    let mut buf = Vec::new();
    let mut stream = BufReader::new(&mut socket);
    let bytes_read = stream.read_until(b'\n', &mut buf).await?; // this waits for a newline so that the client can signal the end of a command
    if bytes_read == 0 {
        return Err(ServerError::IoError(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Connection closed by peer")));
    }
    let cmd: Command = serde_json::from_slice(&buf)?;

    match cmd {
        Command::Store { filename, content } => {
            store_file(&filename, content.as_bytes()).await?;
        },
        Command::Retrieve { filename } => {
            retrive_file(&filename, &mut socket).await?;
        },
        Command::List => {
            list_files(&mut socket).await?;
        },
        Command::Delete { filename } => {
            delete_file(&filename).await?;
        },
        Command::ReplicateStore { filename, content } => {
            replicate_store(&filename, content.as_bytes()).await?;
        },
        Command::ReplicateDelete { filename } => {
            replicate_delete(&filename).await?;
        }
    }

    Ok(())
}

async fn store_file(filename: &str, content: &[u8]) -> Result<(), ServerError> {
    let path = format!("./ball/{}", filename);
    fs::write(&path, content).await?;

    println!("File stored locally: {}", filename);

    replicate_to_peers(filename, content).await?;

    Ok(())
}

async fn retrive_file(filename: &str, socket: &mut TcpStream) -> Result<(), ServerError> {
    let path = format!("./ball/{}", filename);
    let content = match fs::read(&path).await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            return Err(ServerError::IoError(e));
        }
    };

    socket.write_all(&content).await.map_err(|e| {
        eprintln!("Failed to send file: {}", e);
        ServerError::IoError(e)
    })?;

    println!("File {} sent successfully", filename);
    Ok(())
}

async fn list_files(socket: &mut TcpStream) -> Result<(), ServerError> {
    println!("test");
    let mut paths = fs::read_dir("./ball").await.map_err(|e| {
        eprintln!("Failed to read directory: {}", e);
        ServerError::IoError(e)
    })?;

    let mut files = Vec::new();
    while let Ok(Some(entry)) = paths.next_entry().await {
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                files.push(file_name.to_string());
            }
        }
    }

    let message = serde_json::to_string(&files)?;
    socket.write_all(message.as_bytes()).await.map_err(|e| {
        eprintln!("Failed to send file list: {}", e);
        ServerError::IoError(e)
    })?;

    println!("File list sent successfully");

    Ok(())
}

async fn delete_file(filename: &str) -> Result<(), ServerError> {
    let path = format!("./ball/{}", filename);

    fs::remove_file(&path).await.map_err(|e| {
        eprintln!("Failed to delete file: {}", e);
        ServerError::IoError(e)
    })?;

    println!("File deleted locally: {}", filename);

    notify_peers_to_delete(filename).await?;
    
    Ok(())
}

async fn replicate_store(filename: &str, content: &[u8]) -> Result<(), ServerError> {
    let path = format!("./ball/{}", filename);
    fs::write(&path, content).await?;

    println!("File store replicated locally: {}", filename);

    Ok(())
}

async fn replicate_delete(filename: &str) -> Result<(), ServerError> {
    let path = format!("./ball/{}", filename);

    fs::remove_file(&path).await.map_err(|e| {
        eprintln!("Failed to delete file: {}", e);
        ServerError::IoError(e)
    })?;

    println!("File delete replicated locally: {}", filename);
 
    Ok(())
}

async fn replicate_to_peers(filename: &str, content: &[u8]) -> Result<(), ServerError> {
    let message = json!({
        "action": "ReplicateStore",
        "filename": filename,
        "content": base64::encode(content),
    });

    let serialized_msg = serde_json::to_vec(&message)?;
    let serialized_msg_with_newline = [serialized_msg.as_slice(), b"\n"].concat();

    let peers = read_config().await?.peers;

    for peer in peers.iter() {
        match TcpStream::connect(peer).await {
            Ok(mut stream) => {
                if stream.write_all(&serialized_msg_with_newline).await.is_err() {
                    eprintln!("Failed to replicate file to peers: {}", peer);
                }
            },
            Err(_) => eprintln!("Could not connect peer: {}", peer),
        }
    }

    println!("Replication attempt to peers completed for file: {}", filename);

    Ok(())
}

async fn notify_peers_to_delete(filename: &str) -> Result<(), ServerError> {

    let message = json!({
        "action": "ReplicateDelete",
        "filename": filename,
    });

    let serialized_msg = serde_json::to_vec(&message)?;
    let serialized_msg_with_newline = [serialized_msg.as_slice(), b"\n"].concat();

    let peers = read_config().await?.peers;

    for peer in peers.iter() {
        match TcpStream::connect(peer).await {
            Ok(mut stream) => {
                if stream.write_all(&serialized_msg_with_newline).await.is_err() {
                    eprintln!("Failed to notify peer to delete file: {}", peer);
                }
            },
            Err(_) => eprintln!("Could not connect to peer: {}", peer),
        }
    }

    println!("Deletion notification sent to peers for file: {}", filename);

    Ok(())
}