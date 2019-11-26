//! Sled kvs engine.
use crate::engine::KvsEngine;
use crate::{KvsError, Result};
use sled::Db;
use sled::IVec;
use std::path::Path;
use std::str;

pub struct SledKvsEngine {
    inner: Db,
}

impl SledKvsEngine {
    fn open(path: &Path) -> Result<SledKvsEngine> {
        Ok(SledKvsEngine {
            inner: Db::open(path)?,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, val: String) -> Result<()> {
        self.inner.insert(key.as_bytes(), val.as_bytes())?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let result = self.inner.get(key.as_bytes())?;
        if let Some(value) = result {
            // ??? 1. why this can work.
            // ??? 2. how IVec implement to_vec method.
            return Ok(Some(String::from_utf8(value.to_vec())?));
        } else {
            Err(KvsError::from_string("Key not found"))
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let result = self.inner.remove(key.as_bytes())?;
        if let Some(value) = result {
            Ok(())
        } else {
            Err(KvsError::from_string("Key not found"))
        }
    }
}
