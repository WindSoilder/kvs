use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use super::KvsEngine;
use crate::command::Instruction;
use crate::error::{KvsError, Result};

struct InnerStore {
    reader: BufReaderSeekable<File>,
    writer: BufWriterSeekable<File>,
    folder_path: PathBuf,
    index: HashMap<String, u64>,
}

pub struct KvStore {
    inner: Arc<Mutex<InnerStore>>,
}

impl InnerStore {
    pub fn do_compaction(self: &mut InnerStore) -> Result<()> {
        // for each index, construct relative `set` command.
        // ??? maybe we should lock the file or index while doing compaction.
        let mut insts_str: String = String::new();
        {
            for (_, offset) in self.index.iter() {
                // read the relative command.
                self.reader.seek(SeekFrom::Start(*offset))?;
                self.reader.read_line(&mut insts_str)?;
            }
        }

        // directly write instructions into `kvs.db`.
        let truncated_file: File = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(self.folder_path.join("kvs.db"))?;
        self.writer = BufWriterSeekable {
            inner: BufWriter::new(truncated_file),
            pos: 0,
        };
        self.writer.write_all(insts_str.as_bytes())?;
        self.writer.flush()?;

        let new_file: File = File::open(self.folder_path.join("kvs.db"))?;
        self.reader = BufReaderSeekable {
            inner: BufReader::new(new_file),
            pos: 0,
        };

        // don't forget to re-build index.
        self.index = build_indx(&mut self.reader)?;
        Ok(())
    }
}

fn build_indx(reader: &mut BufReaderSeekable<File>) -> Result<HashMap<String, u64>> {
    let mut map: HashMap<String, u64> = HashMap::new();
    loop {
        let position_before: u64 = reader.seek(SeekFrom::Current(0))?;
        let mut line_content: String = String::new();
        reader.read_line(&mut line_content)?;
        // Instruction is end.
        if line_content.is_empty() {
            return Ok(map);
        }
        let instruction: Instruction = serde_json::from_str(&line_content)?;
        instruction.play(&mut map, position_before);
    }
}

impl KvStore {
    /// Open the local kvs store from given file.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into();
        // the inner file name is kvs.db
        fs::create_dir_all(&path)?;

        // locate inner kvs.db file
        let f_path: PathBuf = path.join("kvs.db");

        let db_writer: BufWriterSeekable<File> =
            BufWriterSeekable::new(OpenOptions::new().write(true).create(true).open(&f_path)?);
        let mut db_reader: BufReaderSeekable<File> = BufReaderSeekable::new(File::open(&f_path)?);
        // Build memory-index.
        let indx: HashMap<String, u64> = build_indx(&mut db_reader)?;
        Ok(KvStore {
            inner: Arc::new(Mutex::new(InnerStore {
                reader: db_reader,
                writer: db_writer,
                folder_path: path,
                index: indx,
            })),
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
        let mut inner: MutexGuard<InnerStore> = self.inner.lock().expect("Lock KvsEngine failed.");

        // create a relative fiinstruction object.
        let instruction: Instruction = Instruction::Set {
            key: key.clone(),
            value: val,
        };
        let inst_str: String = serde_json::ser::to_string(&instruction)?;
        // just write serialized data into file
        let offset: u64 = inner.writer.seek(SeekFrom::End(0))?;
        // write the current offset to inner index.
        inner.index.insert(key, offset);
        inner
            .writer
            .write_all(format!("{}\n", inst_str).as_bytes())?;
        inner.writer.flush()?;
        // NOTE: do_compaction here is not efficient.
        inner.do_compaction()?;
        Ok(())
    }

    fn get(self: &KvStore, key: String) -> Result<Option<String>> {
        let mut inner: MutexGuard<InnerStore> = self.inner.lock().expect("Lock KvsEngine failed.");
        if !inner.index.contains_key(&key) {
            return Ok(None);
        }

        // load command from file and run it.
        let pointer = inner.index.get(&key).unwrap().to_owned();
        inner.reader.seek(SeekFrom::Start(pointer))?;
        let mut buf: String = String::new();
        inner.reader.read_line(&mut buf)?;
        let instruction: Instruction = serde_json::from_str(&buf)?;
        match instruction {
            Instruction::Set { key: _key, value } => Ok(Some(value.clone())),
            _ => Ok(None),
        }
    }

    fn remove(self: &KvStore, key: String) -> Result<()> {
        let mut inner: MutexGuard<InnerStore> = self.inner.lock().expect("Lock KvsEngine failed.");

        // check key exists.
        if !inner.index.contains_key(&key) {
            return Err(KvsError::from_string("Key not found"));
        }
        // Remember to remove key from inner index.
        inner.index.remove(&key);
        inner.writer.seek(SeekFrom::End(0))?;
        let instruction: Instruction = Instruction::Rm { key };
        let inst_str = serde_json::to_string(&instruction)?;
        inner
            .writer
            .write_all(format!("{}\n", inst_str).as_bytes())?;
        inner.writer.flush()?;
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

struct BufReaderSeekable<T: Seek + Read> {
    inner: BufReader<T>,
    pos: u64,
}

impl<T: Seek + Read> BufReaderSeekable<T> {
    fn new(reader: T) -> BufReaderSeekable<T> {
        BufReaderSeekable {
            inner: BufReader::new(reader),
            pos: 0,
        }
    }
}

impl<T: Seek + Read> BufRead for BufReaderSeekable<T> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}

impl<T: Seek + Read> Read for BufReaderSeekable<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let readed: usize = self.inner.read(buf)?;
        self.pos += readed as u64;
        Ok(readed)
    }
}

impl<T: Seek + Read> Seek for BufReaderSeekable<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos: u64 = self.inner.seek(pos)?;
        self.pos = new_pos;
        Ok(new_pos)
    }
}

struct BufWriterSeekable<T: Seek + Write> {
    inner: BufWriter<T>,
    pos: u64,
}

impl<T: Seek + Write> BufWriterSeekable<T> {
    fn new(writer: T) -> BufWriterSeekable<T> {
        BufWriterSeekable {
            inner: BufWriter::new(writer),
            pos: 0,
        }
    }
}

impl<T: Seek + Write> Write for BufWriterSeekable<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size: usize = self.inner.write(buf)?;
        self.pos += size as u64;
        Ok(size)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek + Write> Seek for BufWriterSeekable<T> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos: u64 = self.inner.seek(pos)?;
        self.pos = new_pos;
        Ok(new_pos)
    }
}
