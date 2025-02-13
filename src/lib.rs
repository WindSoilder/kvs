//! Main doc string.
//! When setting a key to a value, kvs writes the set command to disk in a sequential log, then stores the log pointer (file offset)
//! of that command in the in-memory index from key to pointer. When removing a key, similarly, kvs writes the rm command in the log,
//! then removes the key from the in-memory index. When retrieving a value for a key with the get command, it searches the index, and
//! if found then loads from the log the command at the corresponding log pointer, evaluates the command and returns the result.
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

pub mod command;
mod engine;
mod error;
mod network;
pub mod thread_pool;

pub use engine::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Repr, Result};
pub use network::client::Client;
pub use network::server::Server;
pub use network::Response;
