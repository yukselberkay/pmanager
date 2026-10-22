/**
 * db.rs
 * Encrypted database file format.
 *
 * The decrypted database only ever lives in memory. On disk the file looks
 * like this (integers are little endian):
 *
 *   magic       8 bytes   "PMANAGER"
 *   version     u16       FORMAT_VERSION
 *   mem_cost    u32       argon2id memory cost in KiB
 *   time_cost   u32       argon2id iterations
 *   lanes       u32       argon2id parallelism
 *   salt        16 bytes  random, generated once per database
 *   nonce       12 bytes  random, generated on every save
 *   ciphertext  aes-256-gcm of the libkvdb log, the header above is
 *               authenticated as associated data.
 *
 * Every value in the log is a json encoded `Entry`.
*/
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{Cursor, ErrorKind};
use std::path::{Path, PathBuf};

use libkvdb::KeyValueDB;
use rand::rngs::OsRng;
use rand::RngCore;
use serde_derive::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::aes_gcm::{AesGcm256, KEY_LEN, NONCE_LEN, TAG_LEN};
use crate::kdf::{Argon2, KdfParams};
use crate::Result;

pub const MAGIC: &[u8; 8] = b"PMANAGER";
pub const FORMAT_VERSION: u16 = 2;
const SALT_LEN: usize = 16;
const HEADER_LEN: usize = MAGIC.len() + 2 + 3 * 4 + SALT_LEN + NONCE_LEN;

// pmanager <= 0.9.7 encrypted the whole file with this constant nonce.
const LEGACY_NONCE: &[u8; NONCE_LEN] = b"unique nonce";
// pmanager <= 0.9.7 inserted this placeholder record when initializing a database.
const LEGACY_PLACEHOLDER_KEY: &[u8] = b" ";
const LEGACY_SEPARATOR: &str = " -> ";

pub const LEGACY_FORMAT_MESSAGE: &str = "This database was created by an older version of pmanager and uses an insecure format. Run 'pmanager migrate' to upgrade it.";
const DECRYPT_ERROR_MESSAGE: &str =
    "Cannot decrypt the database: wrong master password or the file is corrupted.";

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct Entry {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Format {
    Current,
    Legacy,
}

pub fn detect_format(bytes: &[u8]) -> Result<Format> {
    if bytes.len() < MAGIC.len() + 2 || !bytes.starts_with(MAGIC) {
        return Ok(Format::Legacy);
    }

    let version = u16::from_le_bytes([bytes[MAGIC.len()], bytes[MAGIC.len() + 1]]);
    match version {
        FORMAT_VERSION => Ok(Format::Current),
        v if v > FORMAT_VERSION => Err(format!(
            "This database uses format version {}, which requires a newer version of pmanager.",
            v
        )
        .into()),
        v => Err(format!("Unknown database format version {}.", v).into()),
    }
}

fn not_found(path: &Path) -> Box<dyn std::error::Error> {
    format!(
        "Database not found at {}. Run 'pmanager init --db-path ~' to create one.",
        path.display()
    )
    .into()
}

pub fn read_db_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).map_err(|why| {
        if why.kind() == ErrorKind::NotFound {
            not_found(path)
        } else {
            format!("Cannot read {}: {}", path.display(), why).into()
        }
    })
}

struct Header {
    params: KdfParams,
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
}

impl Header {
    fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER_LEN);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&self.params.mem_cost.to_le_bytes());
        out.extend_from_slice(&self.params.time_cost.to_le_bytes());
        out.extend_from_slice(&self.params.lanes.to_le_bytes());
        out.extend_from_slice(&self.salt);
        out.extend_from_slice(&self.nonce);
        out
    }

    fn parse(bytes: &[u8]) -> Result<Header> {
        if bytes.len() < HEADER_LEN + TAG_LEN {
            return Err("The database file is truncated.".into());
        }

        let u32_at = |offset: usize| {
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
        };
        let params_offset = MAGIC.len() + 2;
        let salt_offset = params_offset + 3 * 4;
        let nonce_offset = salt_offset + SALT_LEN;

        Ok(Header {
            params: KdfParams {
                mem_cost: u32_at(params_offset),
                time_cost: u32_at(params_offset + 4),
                lanes: u32_at(params_offset + 8),
            },
            salt: bytes[salt_offset..nonce_offset].try_into().unwrap(),
            nonce: bytes[nonce_offset..HEADER_LEN].try_into().unwrap(),
        })
    }
}

fn lock_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".lock");
    path.with_file_name(name)
}

/// Takes an exclusive lock so that two processes can not modify the database
/// at the same time and silently lose each other's changes. The lock is
/// released when the returned file is dropped.
pub fn lock(path: &Path) -> Result<File> {
    if !path.exists() {
        return Err(not_found(path));
    }
    let lock_path = lock_path(path);
    let mut options = OpenOptions::new();
    options.create(true).truncate(false).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options
        .open(&lock_path)
        .map_err(|why| format!("Cannot open lock file {}: {}", lock_path.display(), why))?;

    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(
            "The database is being modified by another pmanager process. Try again once it has finished."
                .into(),
        ),
        Err(TryLockError::Error(why)) => {
            Err(format!("Cannot lock {}: {}", lock_path.display(), why).into())
        }
    }
}

/// A decrypted database held in memory.
pub struct Vault {
    path: PathBuf,
    key: Zeroizing<[u8; KEY_LEN]>,
    salt: [u8; SALT_LEN],
    params: KdfParams,
    store: KeyValueDB<Cursor<Vec<u8>>>,
    _lock: Option<File>,
}

impl Vault {
    /// Creates a new, empty database in memory. Nothing is written until `save`.
    pub fn new(path: &Path, password: &str) -> Result<Vault> {
        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        let params = KdfParams::DEFAULT;

        Ok(Vault {
            path: path.to_path_buf(),
            key: Argon2::derive_key(password, &salt, &params)?,
            salt,
            params,
            store: KeyValueDB::in_memory(),
            _lock: None,
        })
    }

    /// Opens and decrypts a database. Pass the file returned by `lock` when the
    /// database is going to be modified, it is held until the vault is dropped.
    pub fn open(path: &Path, password: &str, lock: Option<File>) -> Result<Vault> {
        let bytes = read_db_file(path)?;
        if detect_format(&bytes)? == Format::Legacy {
            return Err(LEGACY_FORMAT_MESSAGE.into());
        }

        let header = Header::parse(&bytes)?;
        let key = Argon2::derive_key(password, &header.salt, &header.params)?;
        let mut plaintext = AesGcm256::decrypt_bytes(
            &key,
            &header.nonce,
            &bytes[..HEADER_LEN],
            &bytes[HEADER_LEN..],
        )
        .map_err(|_| DECRYPT_ERROR_MESSAGE)?;

        let store = KeyValueDB::from_bytes(std::mem::take(&mut *plaintext))
            .map_err(|why| format!("The database is corrupted: {}", why))?;

        Ok(Vault {
            path: path.to_path_buf(),
            key,
            salt: header.salt,
            params: header.params,
            store,
            _lock: lock,
        })
    }

    pub fn contains(&self, domain: &str) -> bool {
        self.store.index.contains_key(domain.as_bytes())
    }

    pub fn get(&mut self, domain: &str) -> Result<Option<Entry>> {
        let value = match self.store.get(domain.as_bytes())? {
            None => return Ok(None),
            Some(value) => Zeroizing::new(value),
        };

        let entry: Entry = serde_json::from_slice(&value)
            .map_err(|why| format!("The entry for {} is corrupted: {}", domain, why))?;
        Ok(Some(entry))
    }

    pub fn insert(&mut self, domain: &str, entry: &Entry) -> Result<()> {
        if domain.is_empty() {
            return Err("Domain cannot be empty.".into());
        }
        let value = Zeroizing::new(serde_json::to_vec(entry)?);
        self.store.insert(domain.as_bytes(), &value)?;
        Ok(())
    }

    pub fn remove(&mut self, domain: &str) -> Result<()> {
        self.store.delete(domain.as_bytes())?;
        Ok(())
    }

    /// Every domain in the database, sorted.
    pub fn domains(&self) -> Vec<String> {
        let mut domains: Vec<String> = self
            .store
            .keys()
            .map(|key| String::from_utf8_lossy(key).into_owned())
            .collect();
        domains.sort();
        domains
    }

    /// Encrypts the database with a fresh nonce and atomically replaces the file on disk.
    pub fn save(&mut self) -> Result<()> {
        // compact the log so that deleted and overwritten passwords are not kept around.
        let mut compacted = KeyValueDB::in_memory();
        let keys: Vec<Vec<u8>> = self.store.keys().cloned().collect();
        for key in keys {
            if let Some(value) = self.store.get(&key)? {
                let value = Zeroizing::new(value);
                compacted.insert(&key, &value)?;
            }
        }
        let mut old = std::mem::replace(&mut self.store, compacted);
        old.buffer_mut().zeroize();

        let mut nonce = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);
        let header = Header {
            params: self.params,
            salt: self.salt,
            nonce,
        }
        .to_bytes();

        let ciphertext =
            AesGcm256::encrypt_bytes(&self.key, &nonce, &header, self.store.as_bytes())
                .map_err(|_| "Cannot encrypt the database.")?;

        let mut out = header;
        out.extend_from_slice(&ciphertext);
        crate::util::write_file_atomic(&self.path, &out)
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        self.store.buffer_mut().zeroize();
    }
}

/// Decrypts a database written by pmanager <= 0.9.7 and returns its entries,
/// sorted by domain. Warnings about entries that needed fixing up are
/// appended to `warnings`.
pub fn read_legacy(
    bytes: &[u8],
    password: &str,
    warnings: &mut Vec<String>,
) -> Result<Vec<(String, Entry)>> {
    let key = Argon2::derive_legacy_key(password)?;
    let mut plaintext = AesGcm256::decrypt_bytes(&key, LEGACY_NONCE, &[], bytes)
        .map_err(|_| DECRYPT_ERROR_MESSAGE)?;

    let mut store = KeyValueDB::from_bytes(std::mem::take(&mut *plaintext))
        .map_err(|why| format!("The legacy database is corrupted: {}", why))?;

    let keys: Vec<Vec<u8>> = store.keys().cloned().collect();
    let mut entries = Vec::with_capacity(keys.len());
    for key in keys {
        if key == LEGACY_PLACEHOLDER_KEY {
            continue;
        }
        let value = match store.get(&key)? {
            Some(value) => Zeroizing::new(value),
            None => continue,
        };

        let domain = String::from_utf8_lossy(&key).into_owned();
        let value = Zeroizing::new(String::from_utf8_lossy(&value).into_owned());

        // legacy values are "username -> password". Usernames were rarely
        // entered with " -> " in them, so the first separator is the split
        // point. This also recovers passwords containing " -> ", which older
        // versions truncated when copying them.
        let entry = match value.split_once(LEGACY_SEPARATOR) {
            Some((username, password)) => Entry {
                username: username.to_string(),
                password: password.to_string(),
            },
            None => {
                warnings.push(format!(
                    "Entry {:?} had no username/password separator, it was imported as a username with an empty password.",
                    domain
                ));
                Entry {
                    username: value.to_string(),
                    password: String::new(),
                }
            }
        };
        entries.push((domain, entry));
    }

    store.buffer_mut().zeroize();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pmanager-test-{:016x}", OsRng.next_u64()));
        crate::util::create_private_dir(&dir).unwrap();
        dir.join(name)
    }

    fn entry(username: &str, password: &str) -> Entry {
        Entry {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    #[test]
    fn save_and_reopen() {
        let path = test_path("db.pmanager");
        let mut vault = Vault::new(&path, "master").unwrap();
        vault.insert("github.com", &entry("alice", "pa -> ss 🔑")).unwrap();
        vault.insert("gone.com", &entry("bob", "secret")).unwrap();
        vault.remove("gone.com").unwrap();
        vault.save().unwrap();
        drop(vault);

        let bytes = fs::read(&path).unwrap();
        assert_eq!(detect_format(&bytes).unwrap(), Format::Current);

        let mut vault = Vault::open(&path, "master", Some(lock(&path).unwrap())).unwrap();
        assert_eq!(vault.domains(), vec!["github.com".to_string()]);
        let e = vault.get("github.com").unwrap().unwrap();
        assert_eq!(e.username, "alice");
        assert_eq!(e.password, "pa -> ss 🔑");
        assert!(vault.get("gone.com").unwrap().is_none());

        // a second writer is refused while the lock is held.
        assert!(lock(&path).is_err());

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn deleted_data_is_compacted_away() {
        let path = test_path("db.pmanager");
        let mut vault = Vault::new(&path, "master").unwrap();
        vault.insert("a.com", &entry("u", "first-password")).unwrap();
        vault.insert("a.com", &entry("u", "second-password")).unwrap();
        vault.save().unwrap();

        let log = String::from_utf8_lossy(vault.store.as_bytes()).into_owned();
        assert!(!log.contains("first-password"));
        assert!(log.contains("second-password"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn every_save_uses_a_new_nonce() {
        let path = test_path("db.pmanager");
        let mut vault = Vault::new(&path, "master").unwrap();
        vault.save().unwrap();
        let first = fs::read(&path).unwrap();
        vault.save().unwrap();
        let second = fs::read(&path).unwrap();

        assert_eq!(first[..HEADER_LEN - NONCE_LEN], second[..HEADER_LEN - NONCE_LEN]);
        assert_ne!(first[HEADER_LEN - NONCE_LEN..HEADER_LEN], second[HEADER_LEN - NONCE_LEN..HEADER_LEN]);

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn wrong_password_and_tampering_are_rejected() {
        let path = test_path("db.pmanager");
        let mut vault = Vault::new(&path, "master").unwrap();
        vault.insert("a.com", &entry("u", "p")).unwrap();
        vault.save().unwrap();
        drop(vault);

        assert!(Vault::open(&path, "wrong", None).is_err());

        // flipping a bit in the authenticated header must be detected.
        let mut bytes = fs::read(&path).unwrap();
        bytes[HEADER_LEN - 1] ^= 1;
        fs::write(&path, &bytes).unwrap();
        assert!(Vault::open(&path, "master", None).is_err());

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn legacy_database_is_detected_and_read() {
        // build a database exactly like pmanager <= 0.9.7 did.
        let mut store = KeyValueDB::in_memory();
        store.insert(b" ", b" ").unwrap();
        store.insert(b"github.com", b"alice -> old").unwrap();
        store.insert(b"github.com", b"alice -> pa -> ss").unwrap();
        store.insert(b"gone.com", b"bob -> secret").unwrap();
        store.delete(b"gone.com").unwrap();
        let key = Argon2::derive_legacy_key("master").unwrap();
        let bytes = AesGcm256::encrypt_bytes(&key, LEGACY_NONCE, &[], store.as_bytes()).unwrap();

        assert_eq!(detect_format(&bytes).unwrap(), Format::Legacy);

        let mut warnings = Vec::new();
        assert!(read_legacy(&bytes, "wrong", &mut warnings).is_err());
        let entries = read_legacy(&bytes, "master", &mut warnings).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "github.com");
        assert_eq!(entries[0].1.username, "alice");
        assert_eq!(entries[0].1.password, "pa -> ss");
    }

    #[test]
    fn tampered_kdf_parameters_are_rejected() {
        let path = test_path("db.pmanager");
        let mut vault = Vault::new(&path, "master").unwrap();
        vault.save().unwrap();
        drop(vault);

        let mut bytes = fs::read(&path).unwrap();
        bytes[MAGIC.len() + 2..MAGIC.len() + 6].copy_from_slice(&u32::MAX.to_le_bytes());
        fs::write(&path, &bytes).unwrap();
        let err = Vault::open(&path, "master", None).err().unwrap();
        assert!(err.to_string().contains("Invalid key derivation parameters"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
