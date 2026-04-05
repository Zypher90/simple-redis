use std::{env, io};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, sync::broadcast
};

#[tokio::main]
async fn main() {
    let mut command = String::new();
    println!("Enter command: \n");
    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read input");

    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();
    let (reader, mut writer) = stream.into_split();
    writer.write_all(command.as_bytes()).await.unwrap();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    println!("{}", line);

    writer.shutdown().await.unwrap();
}