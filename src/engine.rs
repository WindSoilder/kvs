use crate::error::{KvsError, Result};
use std::str::FromStr;

#[derive(Debug)]
pub enum Engine {
    /// Our own kvs storage engine.
    Kvs,
    /// Sled storage engine.
    Sled,
}

impl FromStr for Engine {
    type Err = KvsError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "kvs" => Ok(Engine::Kvs),
            "sled" => Ok(Engine::Sled),
            _ => Err(KvsError::from_unsupported_engine(&format!(
                "Unsupported engine {}",
                s
            ))),
        }
    }
}

pub trait KvsEngine {
    /// Set the value of a string key to a string.
    ///
    /// # Errors
    /// This method should return an error if the value is not written successfully.
    fn set(&mut self, key: String, val: String) -> Result<()>;

    /// Get the string value of a string key.
    ///
    /// # Errors
    /// This method should return an error if the value is not read successfully.
    fn get(&self, key: String) -> Result<Option<String>>;

    /// Remove a given key.
    ///
    /// # Errors
    /// An error should occured when the key does not exist or it's not remove successfully.
    fn remove(&mut self, key: String) -> Result<()>;
}
