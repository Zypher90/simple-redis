use std::{
    error::Error, io, sync::Arc
};
use chrono::Local;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream}, 
    sync::{Mutex, broadcast},
    runtime::Handle
};

mod decl;
use decl::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let metrics = Handle::current().metrics();

    let server_addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(server_addr).await?;
    println!("Server running on: {}", server_addr);

    let store = Arc::new(Mutex::new(Store::new()));
    let store_s = Arc::clone(&store);

    let (tx, rx) = broadcast::channel::<Arc<Mutex<Store>>>(100);
    let rx = Arc::new(Mutex::new(rx));
    
    let (tx_sv, mut rx_sv) = broadcast::channel::<ServerResponse>(100);

    let (tx_r, rx_r) = broadcast::channel::<&str>(10);
    let rx_r = Arc::new(Mutex::new(rx_r));

    let mut buf = String::new();
    
    match tx.send(Arc::clone(&store)) {
        Ok(_) => println!("Reference of store has been broadcasted to all client threads"),
        Err(e) => println!("Error occured while broadcasting store: {}", e)
    }

    tokio::spawn(async move{
        loop {
            io::stdin().read_line(&mut buf).expect("Command expected");
            let server_response = match buf.clone().trim() {
                "stop" => ServerResponse::ServerClose,
                "list" => ServerResponse::ListClients,
                "show" => ServerResponse::DisplayDB,
                _ => ServerResponse::InvalidCommand
            };
            buf.clear();
            let _ = tx_sv.send(server_response.clone());
            match server_response {
                ServerResponse::ServerClose => break,
                _ => {}
            }
        }
    });

    loop {
        tokio::select! {
            res = listener.accept() => {
                let (stream, addr) = res.unwrap();
                println!("[{}] New connection from: {}", Local::now().format("%H:%M:%S"), addr);
                let receiver = Arc::clone(&rx);
                let rx_response = Arc::clone(&rx_r);

                tokio::spawn(async move {
                    let mut r = receiver.lock().await;
                    let store = r.recv().await.unwrap();
                    handler(stream, &store, rx_response).await;
                });
            }

            res = rx_sv.recv() => {
                let response = res.unwrap();
                match response {
                    ServerResponse::ServerClose => {
                        _ = tx_r.send("close");
                        println!("Server closed");
                        return Ok(());
                    },
                    ServerResponse::DisplayDB => {
                        let store_s = store_s.lock().await;
                        println!("------------------------STORE--------------------------");
                        store_s.display();
                        println!("=======================================================");
                    },
                    ServerResponse::ListClients => {
                        println!("Active clients: {}", metrics.num_alive_tasks()-1);
                    },
                    ServerResponse::InvalidCommand => {
                        println!("Invalid Command");
                    }
                }
            }
        }
    }

}

async fn handler(mut stream: TcpStream, store: &Arc<Mutex<Store>>, rx_response: Arc<Mutex<broadcast::Receiver<&str>>>) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let mut rx_response = rx_response.lock().await;

    loop{
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break; //Client disconnected
                }

                let request = line.trim();
                println!("Incoming request: {}", request);
                let command = command_parser(request);
                let response = execute(command, store);
                let output = format_response(response.await);
                println!("Response: {}", output);

                writer.write_all(output.as_bytes()).await.unwrap();
                line.clear();
            }

            res = rx_response.recv() => {
                match res.unwrap() {
                    "close" => {    
                        let close_msg = "server closed";
                        writer.write_all(close_msg.as_bytes()).await.unwrap();
                    },
                    _ => {}
                }
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
        "UPDATE" if parts.len() == 3 => {
            Command::Update(parts[1].to_lowercase().to_string(), parts[2].to_lowercase().to_string())
        },
        _ => {
            Command::Unknown
        }
    }
}