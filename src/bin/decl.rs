use std::{collections::HashMap};
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum Command {
    Set(String, String),
    Get(String),
    Update(String, String),
    Delete(String),
    Unknown
}

pub enum StoreError{
    NotFoundError,
    TypeMismatchError,
    CommandMismatchError,
    ValueStoreError,
    InvalidCommand
}

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

fn main() {

}