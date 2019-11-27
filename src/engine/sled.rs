//! Sled kvs engine.
use super::KvsEngine;
use crate::{KvsError, Result};
use sled::Db;
use std::path::{Path, PathBuf};
use std::str;

pub struct SledKvsEngine {
    inner: Db,
}

impl SledKvsEngine {
    pub fn open(path: &Path) -> Result<SledKvsEngine> {
        Ok(SledKvsEngine {
            inner: Db::open(path)?,
        })
    }

    pub fn db_exists(path: &Path) -> bool {
        let file_name: &str = "db";
        let full_path: PathBuf = path.join(file_name);
        full_path.exists()
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, val: String) -> Result<()> {
        self.inner.insert(key.as_bytes(), val.as_bytes())?;
        // This maybe not efficient.
        self.inner.flush()?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let result = self.inner.get(key.as_bytes())?;
        if let Some(value) = result {
            // NOTE: sled::IVec implement Deref<target=[u8]>, so sled::IVec can invoke to_vec method.
            return Ok(Some(String::from_utf8(value.to_vec())?));
        } else {
            Err(KvsError::from_string("Key not found"))
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let result = self.inner.remove(key.as_bytes())?;
        // This maybe not efficient.
        self.inner.flush()?;
        if let Some(_) = result {
            Ok(())
        } else {
            Err(KvsError::from_string("Key not found"))
        }
    }
}
