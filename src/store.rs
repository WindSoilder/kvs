use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::Path;

use crate::command::{Command, Instruction};
use crate::error::{KvsError, Result};

pub struct KvStore {
    db_file: File,
    map: HashMap<String, String>,
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
        let buffer: BufReader<File> = BufReader::new(file_work);
        let mut map: HashMap<String, String> = HashMap::new();

        for line in buffer.lines() {
            if let Ok(line_content) = line {
                let instruction: Instruction = serde_json::from_str(&line_content)?;
                instruction.play(&mut map);
            }
        }
        // check if key exists in inner map.
        Ok(map.get(&key).cloned())
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
        let buffer: BufReader<File> = BufReader::new(file_work);

        for line in buffer.lines() {
            if let Ok(line_content) = line {
                let instruction: Instruction = serde_json::from_str(&line_content)?;
                instruction.play(&mut self.map);
            }
        }

        let remove_result: Option<String> = self.map.remove(&key);
        if remove_result.is_none() {
            return Err(KvsError::from_string("key not found"));
        }
        // construct a remove command and append to log.
        let command: Command = Command::Rm { key };
        let instruction: Instruction = Instruction::new(command);
        let instruction_str: String = serde_json::to_string(&instruction)?;
        self.db_file
            .write_all(format!("{}\n", instruction_str).as_bytes())?;

        // do actual remove action.
        instruction.play(&mut self.map);
        Ok(())
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
        Ok(KvStore {
            db_file,
            map: HashMap::new(),
        })
    }
}
