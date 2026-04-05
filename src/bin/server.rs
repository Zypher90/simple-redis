use std::error::Error;
use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, sync::broadcast
};

#[derive(Debug, Clone)]
enum Command {
    Set(String, String),
    Get(String),
    Delete(String),
    Unknown
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server_addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(server_addr).await?;
    println!("Server running on: {}", server_addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("[{}] New connection from: {}", Local::now().format("%H:%M:%S"), addr);

        tokio::spawn(async move {
            handler(stream).await;
        });
    }

    Ok(())
}

async fn handler(mut stream: TcpStream) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop{
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break; //Client disconnected
                }

                let input = line.trim();
                println!("Input: {}", input);
                let command = command_parser(&input);
                let response = match command {
                    Command::Set(_, _) => "OK\n",
                    Command::Get(_) => "GOT\n",
                    Command::Delete(_) => "DELETED\n",
                    Command::Unknown => "ERR\n"
                };
                println!("Response: {}", response);

                writer.write_all(response.as_bytes()).await.unwrap();
            }
        }
    }
}

fn command_parser(command: &str) -> Command{
    let parts: Vec<&str> = command.trim().split_whitespace().collect();

    if parts.len() == 0{
        return Command::Unknown;
    }

    match parts[0].to_ascii_uppercase().as_str() {
        "GET" if parts.len() == 2 => {
            Command::Get(parts[1].to_lowercase().to_string())
        },
        "SET" if parts.len() == 3 => {
            Command::Set(parts[1].to_lowercase().to_string(), parts[2].to_lowercase().to_string())
        },
        "DELETE" if parts.len() == 2  => {
            Command::Delete(parts[1].to_lowercase().to_string())
        },
        _ => {
            Command::Unknown
        }
    }
}