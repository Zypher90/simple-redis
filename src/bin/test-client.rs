use std::{io};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, 
    net::{TcpStream}, 
    sync::broadcast
};

#[tokio::main]
async fn main() {
    let mut command = String::new();
    
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let (reader, mut writer) = stream.into_split();
    
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    let (tx, mut rx) = broadcast::channel::<bool>(5);

    tokio::spawn(async move {
        loop {
            print!("Command: ");
            io::stdin()
                .read_line(&mut command)
                .expect("Failed to read command");
            let _ = writer.write_all(command.as_bytes()).await;
            command.clear();
            if rx.recv().await.unwrap(){
                break;
            }
        }
    });
    
    loop {
        if reader.read_line(&mut line).await.unwrap() == 0 {
            break;
        }
        match line.as_str() {
            "server closed" => {
                println!("Server has been closed, connection terminated.");
                // client_handle.abort();
                let _ = tx.send(true);
                return;
            },
            l => {
                println!("{}", l);
            }
        }
        line.clear();
    }
}