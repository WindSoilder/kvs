///! Command definition for kvs.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Get { key: String },
    Set { key: String, value: String },
    Rm { key: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Instruction {
    command: Command,
}

impl Instruction {
    pub fn new(command: Command) -> Instruction {
        Instruction { command }
    }

    /// play my instruction under the given store.
    ///
    /// which may affect the store.
    pub fn play(&self, store: &mut HashMap<String, String>) {
        match &self.command {
            Command::Set { key, value } => {
                store.insert(key.clone(), value.clone());
            }
            Command::Rm { key } => {
                store.remove(key);
            }
            _ => {} // for get, do nothing
        };
    }
}
