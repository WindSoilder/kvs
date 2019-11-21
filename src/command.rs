///! Command definition for kvs.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum Instruction {
    Get { key: String },
    Set { key: String, value: String },
    Rm { key: String },
}

impl Instruction {
    pub fn play(&self, store: &mut HashMap<String, u64>, position: u64) {
        match self {
            Instruction::Set { key, value: _value } => {
                store.insert(key.clone(), position);
            }
            Instruction::Rm { key } => {
                store.remove(key);
            }
            _ => {} // for get, do nothing.
        }
    }
}
