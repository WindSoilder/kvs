use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::KvsEngine;
use crate::command::Instruction;
use crate::error::{KvsError, Result};

const THRESHOLD: usize = 4096;

struct InnerStore {
    db_file: File,
    folder_path: PathBuf,
    index: HashMap<String, u64>,
    useless_cmd: usize,
}

pub struct KvStore {
    inner: Arc<RwLock<InnerStore>>,
}

impl InnerStore {
    pub fn do_compaction(self: &mut InnerStore) -> Result<()> {
        if self.useless_cmd < THRESHOLD {
            return Ok(());
        }
        // for each index, construct relative `set` command.
        // ??? maybe we should lock the file or index while doing compaction.
        let mut insts_str: String = String::new();
        {
            let file_work: File = OpenOptions::new()
                .read(true)
                .open(self.folder_path.join("kvs.db"))?;
            let mut buffer: BufReader<File> = BufReader::new(file_work);

            for (_, offset) in self.index.iter() {
                // read the relative command.
                buffer.seek(SeekFrom::Start(*offset))?;
                buffer.read_line(&mut insts_str)?;
            }
        }

        let new_file: File = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.folder_path.join("kvs.db"))?;
        let mut write_buffer: BufWriter<File> = BufWriter::new(new_file);
        write_buffer.write_all(insts_str.as_bytes())?;
        write_buffer.flush()?;
        // remember to rebuild index.
        self.build_indx()?;
        Ok(())
    }

    pub fn build_indx(self: &mut InnerStore) -> Result<()> {
        let mut buffer: BufReader<File> = BufReader::new(
            OpenOptions::new()
                .read(true)
                .open(self.folder_path.join("kvs.db"))?,
        );
        //let mut buffer: BufReader<File> = BufReader::new(self.db_file.try_clone()?);
        loop {
            let position_before: u64 = buffer.seek(SeekFrom::Current(0))?;
            let mut line_content: String = String::new();
            buffer.read_line(&mut line_content)?;
            // Instruction is end.
            if line_content.is_empty() {
                break;
            }
            let instruction: Instruction = serde_json::from_str(&line_content)?;
            instruction.play(&mut self.index, position_before)
        }
        Ok(())
    }
}

impl KvStore {
    /// Open the local kvs store from given file.
    pub fn open(path: &Path) -> Result<KvStore> {
        let folder_path: PathBuf = path.to_owned();
        let file_name: &str = "kvs.db";
        let full_path: PathBuf = path.join(file_name);
        let db_file: File = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(full_path)?;

        let mut inner = InnerStore {
            db_file,
            folder_path,
            index: HashMap::new(),
            useless_cmd: 0,
        };
        inner.build_indx()?;
        Ok(KvStore {
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    /// Check if the db file exists in for the given folder.
    pub fn db_exists(path: &Path) -> bool {
        let file_name: &str = "kvs.db";
        let full_path: PathBuf = path.join(file_name);

        full_path.exists()
    }
}

impl KvsEngine for KvStore {
    fn set(self: &KvStore, key: String, val: String) -> Result<()> {
        let mut inner: RwLockWriteGuard<InnerStore> =
            self.inner.write().expect("Lock KvsEngine failed.");

        // create a relative fiinstruction object.
        let instruction: Instruction = Instruction::Set {
            key: key.clone(),
            value: val,
        };
        let inst_str: String = serde_json::ser::to_string(&instruction)?;
        // just write serialized data into file
        let offset: u64 = inner.db_file.seek(SeekFrom::End(0))?;
        // write the current offset to inner index.
        if let Some(_) = inner.index.insert(key, offset) {
            inner.useless_cmd += 1;
        }
        inner
            .db_file
            .write_all(format!("{}\n", inst_str).as_bytes())?;
        inner.db_file.flush()?;
        // NOTE: do_compaction here is not efficient.
        inner.do_compaction()?;
        Ok(())
    }

    fn get(self: &KvStore, key: String) -> Result<Option<String>> {
        let inner: RwLockReadGuard<InnerStore> = self.inner.read().expect("Lock KvsEngine failed.");
        if !inner.index.contains_key(&key) {
            return Ok(None);
        }
        // access the key to get relative file pointer index.
        // let file: File = inner.db_file.try_clone()?;
        let file: File = OpenOptions::new()
            .read(true)
            .open(inner.folder_path.join("kvs.db"))?;
        let mut reader: BufReader<File> = BufReader::new(file);

        // load command from file and run it.
        let pointer = inner.index.get(&key).unwrap();
        reader.seek(SeekFrom::Start(*pointer))?;
        let mut buf: String = String::new();
        reader.read_line(&mut buf)?;
        let instruction: Instruction = serde_json::from_str(&buf)?;
        match instruction {
            Instruction::Set { key: _key, value } => Ok(Some(value.clone())),
            _ => Ok(None),
        }
    }

    fn remove(self: &KvStore, key: String) -> Result<()> {
        let mut inner: RwLockWriteGuard<InnerStore> =
            self.inner.write().expect("Lock KvsEngine failed.");

        // check key exists.
        if !inner.index.contains_key(&key) {
            return Err(KvsError::from_string("Key not found"));
        }
        // Remember to remove key from inner index.
        if let Some(_) = inner.index.remove(&key) {
            inner.useless_cmd += 1;
        }
        // The key exists, so it's ok to append a remove command to end of log file.
        // let mut file_work: File = inner.db_file.try_clone()?;
        let mut file_work: File = OpenOptions::new()
            .write(true)
            .append(true)
            .open(inner.folder_path.join("kvs.db"))?;

        file_work.seek(SeekFrom::End(0))?;
        let instruction: Instruction = Instruction::Rm { key };
        let inst_str = serde_json::to_string(&instruction)?;
        inner
            .db_file
            .write_all(format!("{}\n", inst_str).as_bytes())?;
        inner.db_file.flush()?;
        // NOTE: do_compaction here is not efficient.
        inner.do_compaction()?;
        Ok(())
    }
}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        KvStore {
            inner: self.inner.clone(),
        }
    }
}
