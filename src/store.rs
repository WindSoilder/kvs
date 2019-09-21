use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::command::{Command, Instruction};
use crate::error::{KvsError, Result};

pub struct KvStore {
    db_file: File,
}

impl KvStore {
    /// Set the value of a string key to a string. Return an error if the value is not written successfully.
    ///
    /// # Examples
    ///
    /// ```
    /// use kvs::KvStore;
    ///
    /// let mut store: KvStore = KvStore::new();
    /// store.set("name".to_owned(), "zero".to_owned());
    /// assert_eq!(store.get("name".to_owned()), Some("zero".to_owned()));
    /// ```
    ///
    pub fn set(self: &mut KvStore, key: String, val: String) -> Result<()> {
        // create a relative instruction object.
        let command: Command = Command::Set { key, value: val };
        let instruction: Instruction = Instruction::new(command);
        let inst_str: String = serde_json::ser::to_string(&instruction)?;
        // just write serialized data into file
        self.db_file.seek(SeekFrom::End(0))?;
        self.db_file
            .write_all(format!("{}\n", inst_str).as_bytes())?;
        Ok(())
    }

    /// Get the string value of a string key. If the key does not exist, return None. Return an error if the value is not read successfully.
    ///
    /// # Examples
    ///
    /// ```
    /// use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("name".to_owned(), "zero".to_owned());
    /// assert_eq!(store.get("name".to_owned()), Some("zero".to_owned()));
    /// ```
    ///
    /// Access an un-existed key should return None.
    /// ```
    /// use kvs::KvStore;
    /// let store = KvStore::new();
    /// assert_eq!(store.get("name".to_owned()).is_none(), true);
    /// ```
    pub fn get(self: &KvStore, key: String) -> Result<Option<String>> {
        let mut file_work: File = self.db_file.try_clone()?;
        // re-seek file and build from store.
        file_work.seek(SeekFrom::Start(0))?;
        let mut buffer: BufReader<File> = BufReader::new(file_work);
        let mut map: HashMap<String, u64> = HashMap::new();

        self.build_indx(&mut buffer, &mut map)?;

        // check if key exists in inner map and if the relative command is set.
        if let Some(position) = map.get(&key) {
            buffer.seek(SeekFrom::Start(*position))?;
            let mut line_content: String = String::new();
            buffer.read_line(&mut line_content)?;
            let instruction: Instruction = serde_json::from_str(&line_content)?;
            match instruction.get_command() {
                Command::Rm { key: _key } => {
                    return Ok(None);
                }
                Command::Set { key: _key, value } => return Ok(Some(value.clone())),
                _ => {}
            }
        }
        Ok(None)
    }

    fn build_indx<T>(
        self: &KvStore,
        buffer: &mut BufReader<T>,
        map: &mut HashMap<String, u64>,
    ) -> Result<()>
    where
        T: Read + Seek,
    {
        loop {
            let position_before: u64 = buffer.seek(SeekFrom::Current(0))?;
            let mut line_content: String = String::new();
            buffer.read_line(&mut line_content)?;
            // Command is end.
            if line_content.is_empty() {
                break;
            }
            let instruction: Instruction = serde_json::from_str(&line_content)?;
            instruction.play(map, position_before)
        }
        Ok(())
    }

    /// Remove a given key. Return an error if the key does not exist or is not removed successfully.
    ///
    /// # Examples
    ///
    /// ```
    /// use kvs::KvStore;
    ///
    /// let mut store = KvStore::new();
    /// store.set("name".to_owned(), "zero".to_owned());
    /// store.remove("name".to_owned());
    /// assert_eq!(store.get("name".to_owned()).is_none(), true);
    /// ```
    pub fn remove(self: &mut KvStore, key: String) -> Result<()> {
        // load commands and play with it to build internal store.
        let mut file_work: File = self.db_file.try_clone()?;
        file_work.seek(SeekFrom::Start(0))?;
        let mut buffer: BufReader<File> = BufReader::new(file_work);
        let mut map: HashMap<String, u64> = HashMap::new();

        self.build_indx(&mut buffer, &mut map)?;

        if let Some(position) = map.get(&key) {
            buffer.seek(SeekFrom::Start(*position))?;
            let mut line_content: String = String::new();
            buffer.read_line(&mut line_content)?;
            let instruction: Instruction = serde_json::from_str(&line_content)?;
            match instruction.get_command() {
                Command::Rm { key: _key } => {
                    return Err(KvsError::from_string("Key not found"));
                }
                Command::Set {
                    key: _key,
                    value: _value,
                } => {
                    // the key exists, so it's ok to append a remove command to end of log file.
                    self.db_file.seek(SeekFrom::End(0))?;
                    let instruction: Instruction = Instruction::new(Command::Rm { key });
                    let inst_str: String = serde_json::to_string(&instruction)?;
                    self.db_file
                        .write_all(format!("{}\n", inst_str).as_bytes())?;
                    return Ok(())
                }
                _ => {}
            }
        }
        Err(KvsError::from_string("Key not found"))
    }

    /// Open the local kvs store from given file.
    pub fn open(path: &Path) -> Result<KvStore> {
        let file_name = "kvs.db";

        let full_path = path.join(file_name);
        let db_file: File = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(full_path)?;
        Ok(KvStore { db_file })
    }
}
