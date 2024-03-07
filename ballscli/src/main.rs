use serde_json::json;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::env;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect(); 
    let command = &args[1];
    let filename = &args[2];

    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    match command.as_str() {
        "store" => {
            let content = std::fs::read(filename)?;
            let cmd = json!({
                "filename": filename,
                "content": content,
            });
            stream.write_all(serde_json::to_string(&cmd)?.as_bytes())?;
        },
        "retrieve" => {
            let cmd = json!({
                "filename": filename,
            });
            stream.write_all(serde_json::to_string(&cmd)?.as_bytes())?;
            let mut content = Vec::new();
            stream.read_to_end(&mut content)?;
            println!("Retrieved content: {:?}", content);
        },
        _ => println!("Unknown command"),
    }

    Ok(())
}
