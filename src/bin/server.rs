use std::{
    error::Error, fs::{File, OpenOptions}, io, sync::Arc
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
        Ok(_) => {},
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

    // println!("Debug");
    let file = File::open("store.txt").unwrap();
    match init_store(&file, &store).await {
        Ok(_) => {},
        Err(E) => {
            println!("Error while reconstructing database: {}", E);
            return Err(format!("{}", E).into());
        }
    }

    let (tx_command, mut rx_command) = broadcast::channel::<String>(100);
    let tx_command = Arc::new(Mutex::new(tx_command));

    tokio::spawn(async move {
        let mut file = OpenOptions::new()
            .append(true)
            .open("store.txt")
            .expect("Error opening database record");
        loop{
            let mut cmd = rx_command.recv().await.unwrap_or_else(|_e| "".to_string());
            cmd.push_str("\n");
            match append_command(&mut file, cmd).await {
                Ok(_) => {}
                Err(E) => println!("{}", E)
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
                let tx_command = Arc::clone(&tx_command);

                tokio::spawn(async move {
                    let mut r = receiver.lock().await;
                    let store = r.recv().await.unwrap();
                    handler(stream, &store, rx_response, tx_command).await;
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

async fn handler(
    mut stream: TcpStream, 
    store: &Arc<Mutex<Store>>, 
    rx_response: Arc<Mutex<broadcast::Receiver<&str>>>, 
    tx_command: Arc<Mutex<broadcast::Sender<String>>>
) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let mut rx_response = rx_response.lock().await;

    let tx_c = tx_command.lock().await;

    loop{
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result.unwrap() == 0 {
                    break; //Client disconnected
                }

                let request = line.trim();
                println!("Incoming request: {}", request);
                let command = command_parser(request);
                let c_command = command.clone();
                let response = execute(command, store).await;
                if !(response == Response::Err(StoreError::InvalidCommand) || c_command == Command::Get(request.to_string().clone())){
                    tx_c.send(line.trim().to_string().clone()).unwrap();
                }
                let output = format_response(response);
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