use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize, Deserialize)]
enum Command {
    Store { filename: String, content: Vec<u8>},
    Retrieve { filename: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 1024]; // vector will be initialized with 1024 elements which are all '0'
            while let Ok(n) = socket.read(&mut buf).await { // reads the data into buffer. n = number of bytes
                if n == 0 { // if n == 0 connection is closed
                    // TODO: why is it going into this every time
                    return;
                }

                let cmd: Command = match serde_json::from_slice(&buf[..n]) {
                    Ok (cmd) => cmd,
                    Err(_) => continue,
                };

                println!("test");

                match cmd {
                    Command::Store { filename, content } => {
                        println!("store");
                        let mut file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(filename)
                            .unwrap();
                        file.write_all(&content).unwrap();
                    }
                    Command::Retrieve { filename } => {
                        let mut file = File::open(filename).unwrap();
                        let mut content = Vec::new();
                        file.read_to_end(&mut content).unwrap();
                        socket.write_all(&content).await.unwrap();
                    }
                }
            }
        });
    }
}