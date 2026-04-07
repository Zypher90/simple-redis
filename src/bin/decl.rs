use std::{collections::HashMap};
use std::{
    error::Error, fs::File, io::{BufRead, BufReader, Write}, sync::Arc
};
use tokio::sync::Mutex;

#[derive(PartialEq, Clone)]
pub enum Command {
    Set(String, String),
    Get(String),
    Update(String, String),
    Delete(String),
    Unknown
}

#[derive(PartialEq)]
pub enum StoreError{
    NotFoundError,
    TypeMismatchError,
    CommandMismatchError,
    ValueStoreError,
    InvalidCommand
}

#[derive(PartialEq)]
pub enum Response{
    Ok,
    Value(String),
    Nil,
    Err(StoreError)
}

#[derive(Clone)]
pub enum ServerResponse{
    ServerClose,
    DisplayDB,
    ListClients,
    InvalidCommand
}

#[derive(Debug)]
pub struct Store{
    store: HashMap<String, String>
}

impl Store {
    pub fn new() -> Self{
        Self {
            store: HashMap::new(),
        }
    }

    pub fn set(self: &mut Self, command: Command) -> Response{
        let Command::Set(key, value) = command else {
            return Response::Err(StoreError::CommandMismatchError);
        };

        match self.store.get(&key) {
            Some(_) => Response::Err(StoreError::ValueStoreError),
            _ => {
                self.store.insert(key, value);
                Response::Ok
            }
        }
    }

    pub fn get(self: &Self, command: Command) -> Response {
        let Command::Get(key) = command else{
            return Response::Err(StoreError::CommandMismatchError);
        };
        match self.store.get(&key) {
            Some(value) => Response::Value(value.clone()),
            _ => Response::Err(StoreError::NotFoundError)
        }
    }

    pub fn delete(self: &mut Self, command: Command) -> Response{
        let Command::Delete(key) = command else {
            return Response::Err(StoreError::CommandMismatchError);
        };
        match self.store.get(&key) {
            Some(_) => {
                self.store.remove(&key);
                Response::Ok
            },
            _ => Response::Err(StoreError::NotFoundError)
        }
    }

    pub fn update(self: &mut Self, command: Command) -> Response {
        let Command::Update(key, value) = command else {
            return Response::Err(StoreError::CommandMismatchError);
        };
        match self.store.get(&key) {
            Some(_) => {
                self.store.insert(key, value);
                Response::Ok
            },
            _ => Response::Err(StoreError::NotFoundError)
        }
    }

    pub fn display(self: &Self){
        for (k, v) in &self.store {
            println!("{} -> {}", k, v);
        }
    }

}

pub async fn execute(command: Command, store: &Arc<Mutex<Store>>) -> Response{
    let mut store = store.lock().await;
    match command {
        Command::Set(_, _) => store.set(command),
        Command::Get(_) => store.get(command),
        Command::Delete(_) => store.delete(command),
        Command::Update(_, _) => store.update(command),
        Command::Unknown => Response::Err(StoreError::InvalidCommand)
    }
}

pub fn format_response(response: Response) -> String {
    match response {
        Response::Ok => String::from("Command executed\n"),
        Response::Value(val) => String::from(format!("Fetched value: {}\n", val)),
        Response::Nil => String::new(),
        Response::Err(err) => match err {
            StoreError::CommandMismatchError => String::from("Mismatched Command\n"),
            StoreError::InvalidCommand => String::from("Invalid Command\n"),
            StoreError::NotFoundError => String::from("Requested value was not found\n"),
            StoreError::TypeMismatchError => String::from("Mismatched types\n"),
            StoreError::ValueStoreError => String::from("The value to be set was already found in store\n")
        }
    }
}

pub fn command_parser(command: &str) -> Command{
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

pub async fn init_store(file: &File, store: &Arc<Mutex<Store>>) -> Result<(), Box<dyn Error>>{
    let reader = BufReader::new(file);
    // println!("Read log: {}", reader.lines());
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let command =  command_parser(&line.as_str());
                if command == Command::Unknown {
                    return Err(format!("Faced an error while parsing unsupported command: {}", line).into());
                }
                // println!("Formatted command: {}", format_command(&command));
                match execute(command, store).await {
                    Response::Err(_) => return Err(format!("Error faced while executing command log").into()),
                    _ => {}
                }
            },
            Err(E) => {
                return Err(format!("Error while reading from command log: {}", E).into());
            }
        }
    }
    Ok(())
}

pub async fn append_command(file: &mut File, command: String) -> Result<(), Box<dyn Error>>{
    match file.write_all(command.as_bytes()){
        Ok(_) => {},
        Err(E) => return Err(format!("Failed to write command with error: {}", E).into())
    }
    Ok(())
}

fn format_command(command: &Command) -> String {
    match command {
        Command::Get(v) => String::from(format!("GET {}", v)),
        Command::Set(k, v) => String::from(format!("SET {} {}", k, v)),
        Command::Update(k, v) => String::from(format!("UPDATE {} {}", k, v)),
        Command::Delete(k) => String::from(format!("DELETE {}", k)),
        Command::Unknown => String::from("Unknown command")
    }
}
fn main() {

}