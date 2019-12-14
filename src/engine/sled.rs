//! Sled kvs engine.
use super::KvsEngine;
use crate::{KvsError, Result};
use sled::Db;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, Mutex, MutexGuard};

struct InnerSledEngine {
    inner: Db,
}

impl InnerSledEngine {
    pub fn insert(&mut self, key: String, val: String) -> Result<()> {
        self.inner.insert(key.as_bytes(), val.as_bytes())?;
        self.inner.flush()?;
        Ok(())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        let result = self.inner.remove(key.as_bytes())?;
        // This maybe not efficient.
        self.inner.flush()?;
        if let Some(_) = result {
            Ok(())
        } else {
            Err(KvsError::from_string("Key not found"))
        }
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        let result = self.inner.get(key.as_bytes())?;
        if let Some(value) = result {
            // NOTE: sled::IVec implement Deref<target=[u8]>, so sled::IVec can invoke to_vec method.
            return Ok(Some(String::from_utf8(value.to_vec())?));
        } else {
            Err(KvsError::from_string("Key not found"))
        }
    }

    pub fn new(path: &Path) -> Result<InnerSledEngine> {
        Ok(InnerSledEngine {
            inner: Db::open(path)?,
        })
    }
}

pub struct SledKvsEngine {
    inner: Arc<Mutex<InnerSledEngine>>,
}

impl SledKvsEngine {
    pub fn open(path: &Path) -> Result<SledKvsEngine> {
        Ok(SledKvsEngine {
            inner: Arc::new(Mutex::new(InnerSledEngine::new(path)?)),
        })
    }

    pub fn db_exists(path: &Path) -> bool {
        let file_name: &str = "db";
        let full_path: PathBuf = path.join(file_name);
        full_path.exists()
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&self, key: String, val: String) -> Result<()> {
        let mut inner: MutexGuard<InnerSledEngine> = self.inner.lock().expect("Can't get lock");
        inner.insert(key, val)
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        let mut inner: MutexGuard<InnerSledEngine> = self.inner.lock().expect("Can't get lock");
        inner.get(key)
    }

    fn remove(&self, key: String) -> Result<()> {
        let mut inner: MutexGuard<InnerSledEngine> = self.inner.lock().expect("Can't get lock");
        inner.remove(key)
    }
}

impl Clone for SledKvsEngine {
    fn clone(&self) -> Self {
        SledKvsEngine {
            inner: self.inner.clone(),
        }
    }
}
