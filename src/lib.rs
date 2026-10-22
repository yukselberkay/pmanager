/**
 * lib.rs
 * Log structured append only database backend.
 *
 * The store works on any `Read + Write + Seek` backend: a file on disk, or an
 * in-memory buffer (`Cursor<Vec<u8>>`) so that decrypted data never has to be
 * written to disk. A record with an empty value is a tombstone (deletion).
*/
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Cursor, SeekFrom};
use std::path::{Path, PathBuf};

use crc::crc32;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

type ByteString = Vec<u8>;
type ByteStr = [u8];

// Upper bound for the up-front allocation of a record, the rest grows on demand.
// Prevents a corrupted length field from triggering a huge allocation.
const MAX_PREALLOC: u64 = 1024 * 1024;

pub struct KeyValueDB<S = File> {
    f: S,
    pub index: HashMap<ByteString, u64>,
}

pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

impl KeyValueDB<File> {
    pub fn open_and_load(f: &PathBuf) -> KeyValueDB {
        let mut store = KeyValueDB::open(f).expect("Unable to open database file");
        store.load().expect("Unable to load data from database");

        store
    }

    pub fn open(path: &Path) -> io::Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;

        let index: HashMap<ByteString, u64> = HashMap::new();
        Ok(KeyValueDB { f, index })
    }
}

impl KeyValueDB<Cursor<Vec<u8>>> {
    /// Creates an empty store that lives entirely in memory.
    pub fn in_memory() -> Self {
        KeyValueDB {
            f: Cursor::new(Vec::new()),
            index: HashMap::new(),
        }
    }

    /// Loads a store from its serialized log.
    pub fn from_bytes(bytes: Vec<u8>) -> io::Result<Self> {
        let mut store = KeyValueDB {
            f: Cursor::new(bytes),
            index: HashMap::new(),
        };
        store.load()?;

        Ok(store)
    }

    /// The serialized log.
    pub fn as_bytes(&self) -> &[u8] {
        self.f.get_ref()
    }

    /// Mutable access to the underlying buffer, e.g. to wipe it.
    pub fn buffer_mut(&mut self) -> &mut Vec<u8> {
        self.f.get_mut()
    }
}

impl<S: Read + Write + Seek> KeyValueDB<S> {
    fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
        let saved_checksum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let val_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len as u64 + val_len as u64;

        let mut data = ByteString::with_capacity(data_len.min(MAX_PREALLOC) as usize);

        {
            f.by_ref().take(data_len).read_to_end(&mut data)?;
        }

        if data.len() as u64 != data_len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated record",
            ));
        }

        let checksum = crc32::checksum_ieee(&data);
        if checksum != saved_checksum {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "data corruption"));
        }
        let value = data.split_off(key_len as usize);
        let key = data;
        Ok(KeyValuePair { key, value })
    }

    pub fn load(&mut self) -> io::Result<()> {
        let mut f = BufReader::new(&mut self.f);

        loop {
            // returns the number of the bytes from the start of the file.
            // this becomes the value of the index.
            let position = f.seek(SeekFrom::Current(0))?;

            // read a record in the file at its current position
            let maybe_kv = Self::process_record(&mut f);

            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(err) => match err.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        break;
                    }
                    _ => return Err(err),
                },
            };

            // the latest record wins, a tombstone removes the key.
            if kv.value.is_empty() {
                self.index.remove(&kv.key);
            } else {
                self.index.insert(kv.key, position);
            }
        }

        Ok(())
    }

    pub fn list(&mut self) {
        for (key, val) in &self.index {
            let mut f = BufReader::new(&mut self.f);
            f.seek(SeekFrom::Start(*val)).unwrap();
            let kv: KeyValuePair = Self::process_record(&mut f).unwrap();

            let s_key = String::from_utf8_lossy(key);
            if s_key == " " {
                continue;
            }
            let user_pass_pair = String::from_utf8_lossy(&kv.value);
            if user_pass_pair == "" {
                continue;
            }

            let res = user_pass_pair.split(" -> ");
            let pair: Vec<&str> = res.collect();
            let username = pair[0];

            println!("{:?} -> {:?}", s_key, username);
        }
    }

    /// Every live key in the store, in no particular order.
    pub fn keys(&self) -> impl Iterator<Item = &ByteString> {
        self.index.keys()
    }

    pub fn get(&mut self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        let position = match self.index.get(key) {
            None => return Ok(None),
            Some(position) => *position,
        };

        let kv = self.get_at(position)?;

        Ok(Some(kv.value))
    }

    pub fn get_at(&mut self, position: u64) -> io::Result<KeyValuePair> {
        let mut f = BufReader::new(&mut self.f);
        f.seek(SeekFrom::Start(position))?;
        let kv = Self::process_record(&mut f)?;

        Ok(kv)
    }

    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        let position = self.insert_but_ignore_index(key, value)?;

        self.index.insert(key.to_vec(), position);
        Ok(())
    }

    pub fn insert_but_ignore_index(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<u64> {
        let mut f = BufWriter::new(&mut self.f);

        let key_len = key.len();
        let val_len = value.len();
        let mut tmp = ByteString::with_capacity(key_len + val_len);

        for byte in key {
            tmp.push(*byte);
        }
        for byte in value {
            tmp.push(*byte);
        }

        let checksum = crc32::checksum_ieee(&tmp);

        // records are always appended, so the new record starts at the end.
        let current_position = f.seek(SeekFrom::End(0))?;
        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key_len as u32)?;
        f.write_u32::<LittleEndian>(val_len as u32)?;
        f.write_all(&mut tmp)?;
        f.flush()?;

        Ok(current_position)
    }

    pub fn update(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        self.insert(key, value)
    }

    pub fn delete(&mut self, key: &ByteStr) -> io::Result<()> {
        self.insert_but_ignore_index(key, b"")?;
        self.index.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_get_delete_roundtrip() {
        let mut store = KeyValueDB::in_memory();
        store.insert(b"a", b"1").unwrap();
        store.insert(b"b", b"2").unwrap();
        // reading moves the cursor, the next insert must still be indexed correctly.
        assert_eq!(store.get(b"a").unwrap(), Some(b"1".to_vec()));
        store.update(b"a", b"3").unwrap();
        store.delete(b"b").unwrap();

        assert_eq!(store.get(b"a").unwrap(), Some(b"3".to_vec()));
        assert_eq!(store.get(b"b").unwrap(), None);

        let mut reloaded = KeyValueDB::from_bytes(store.as_bytes().to_vec()).unwrap();
        assert_eq!(reloaded.get(b"a").unwrap(), Some(b"3".to_vec()));
        assert_eq!(reloaded.get(b"b").unwrap(), None);
        assert_eq!(reloaded.keys().count(), 1);
    }

    #[test]
    fn corrupted_record_is_an_error_not_a_panic() {
        let mut store = KeyValueDB::in_memory();
        store.insert(b"key", b"value").unwrap();
        let mut bytes = store.as_bytes().to_vec();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;

        let err = KeyValueDB::from_bytes(bytes).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn huge_length_fields_do_not_panic() {
        let mut bytes = vec![0u8; 4];
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        let store = KeyValueDB::from_bytes(bytes).unwrap();
        assert_eq!(store.keys().count(), 0);
    }
}
