use crate::Result;

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

mod kvs;
mod sled;

pub use self::kvs::KvStore;
pub use self::sled::SledKvsEngine;
