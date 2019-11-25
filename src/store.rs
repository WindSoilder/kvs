use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::ops::Drop;
use std::path::{Path, PathBuf};

use crate::command::Instruction;
use crate::error::{KvsError, Result};
use crate::KvsEngine;

pub struct KvStore {
    db_file: File,
    folder_path: PathBuf,
    index: HashMap<String, u64>,
}

impl KvStore {
    pub fn do_compaction(self: &mut KvStore) -> Result<()> {
        // for each index, construct relative `set` command.
        // ??? maybe we should lock the file or index while doing compaction.
        let mut insts_str: String = String::new();
        {
            let file_work: File = self.db_file.try_clone()?;
            let mut buffer: BufReader<File> = BufReader::new(file_work);

            for (_, offset) in self.index.iter() {
                // read the relative command.
                buffer.seek(SeekFrom::Start(*offset))?;
                buffer.read_line(&mut insts_str)?;
            }
        }

        // write the tmp file as backup first
        let tmp_name: &str = "tmp.db";
        let tmp_path: PathBuf = self.folder_path.join(tmp_name);
        let file_name: &str = "kvs.db";
        let file_path: PathBuf = self.folder_path.join(file_name);

        let tmp: File = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(tmp_path)?;
        let mut write_buffer: BufWriter<File> = BufWriter::new(tmp);
        write_buffer.write_all(insts_str.as_bytes())?;
        let tmp_path: PathBuf = self.folder_path.join(tmp_name);
        fs::rename(tmp_path, file_path)?;
        Ok(())
    }

    fn build_indx(self: &mut KvStore) -> Result<()> {
        let mut buffer: BufReader<File> = BufReader::new(self.db_file.try_clone()?);
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

        let mut store: KvStore = KvStore {
            db_file,
            folder_path,
            index: HashMap::new(),
        };
        store.build_indx()?;
        Ok(store)
    }
}

impl KvsEngine for KvStore {
    /// Set the value of a string key to a string. Return an error if the value is not written successfully.
    fn set(self: &mut KvStore, key: String, val: String) -> Result<()> {
        // create a relative fiinstruction object.
        let instruction: Instruction = Instruction::Set {
            key: key.clone(),
            value: val,
        };
        let inst_str: String = serde_json::ser::to_string(&instruction)?;
        // just write serialized data into file
        let offset: u64 = self.db_file.seek(SeekFrom::End(0))?;
        // write the current offset to inner index.
        self.index.insert(key, offset);
        self.db_file
            .write_all(format!("{}\n", inst_str).as_bytes())?;
        self.do_compaction()?;
        Ok(())
    }

    /// Get the string value of a string key. If the key does not exist, return None. Return an error if the value is not read successfully.
    fn get(self: &KvStore, key: String) -> Result<Option<String>> {
        if !self.index.contains_key(&key) {
            return Ok(None);
        }
        // access the key to get relative file pointer index.
        let file: File = self.db_file.try_clone()?;
        let mut reader: BufReader<File> = BufReader::new(file);

        // load command from file and run it.
        let pointer = self.index.get(&key).unwrap();
        reader.seek(SeekFrom::Start(*pointer))?;
        let mut buf: String = String::new();
        reader.read_line(&mut buf)?;
        let instruction: Instruction = serde_json::from_str(&buf)?;
        match instruction {
            Instruction::Set { key: _key, value } => Ok(Some(value.clone())),
            _ => Ok(None),
        }
    }

    /// Remove a given key. Return an error if the key does not exist or is not removed successfully.
    fn remove(self: &mut KvStore, key: String) -> Result<()> {
        // check key exists.
        if !self.index.contains_key(&key) {
            return Err(KvsError::from_string("Key not found"));
        }
        // Remember to remove key from inner index.
        self.index.remove(&key);
        // The key exists, so it's ok to append a remove command to end of log file.
        let mut file_work: File = self.db_file.try_clone()?;
        file_work.seek(SeekFrom::End(0))?;
        let instruction: Instruction = Instruction::Rm { key };
        let inst_str = serde_json::to_string(&instruction)?;
        self.db_file
            .write_all(format!("{}\n", inst_str).as_bytes())?;
        Ok(())
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        self.do_compaction().expect("Do compaction failed.");
    }
}
