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
    fn set(&mut self, key: String, val: String) -> Result<()>;

    fn get(&self, key: String) -> Result<Option<String>>;

    fn remove(&mut self, key: String) -> Result<()>;
}
