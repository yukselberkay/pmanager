/**
 * kdf.rs
 * Functions responsible for deriving a key.
 */
use argon2::{self, Config, ThreadMode, Variant, Version};
use zeroize::Zeroizing;

use crate::aes_gcm::KEY_LEN;
use crate::Result;

/// Argon2id cost parameters. They are stored in the database header so they
/// can be raised in the future without breaking existing databases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KdfParams {
    pub mem_cost: u32, // KiB
    pub time_cost: u32,
    pub lanes: u32,
}

impl KdfParams {
    pub const DEFAULT: KdfParams = KdfParams {
        mem_cost: 64 * 1024, // 64 MiB
        time_cost: 3,
        lanes: 4,
    };

    // Bounds for parameters read from a database header. A tampered header
    // must not be able to make us allocate unbounded memory or spin forever.
    const MAX_MEM_COST: u32 = 4 * 1024 * 1024; // 4 GiB
    const MAX_TIME_COST: u32 = 100;
    const MAX_LANES: u32 = 64;

    pub fn validate(&self) -> Result<()> {
        let valid = (1..=Self::MAX_LANES).contains(&self.lanes)
            && (1..=Self::MAX_TIME_COST).contains(&self.time_cost)
            && self.mem_cost >= 8 * self.lanes
            && self.mem_cost <= Self::MAX_MEM_COST;
        if !valid {
            return Err(format!("Invalid key derivation parameters in database header: {:?}", self).into());
        }
        Ok(())
    }
}

pub struct Argon2;

impl Argon2 {
    /// Derives a 256 bit key from the master password with Argon2id.
    pub fn derive_key(
        password: &str,
        salt: &[u8],
        params: &KdfParams,
    ) -> Result<Zeroizing<[u8; KEY_LEN]>> {
        params.validate()?;

        let config = Config {
            variant: Variant::Argon2id,
            version: Version::Version13,
            mem_cost: params.mem_cost,
            time_cost: params.time_cost,
            lanes: params.lanes,
            thread_mode: ThreadMode::Parallel,
            secret: &[],
            ad: &[],
            hash_length: KEY_LEN as u32,
        };

        let hash = Zeroizing::new(argon2::hash_raw(password.as_bytes(), salt, &config)?);

        let mut key = Zeroizing::new([0u8; KEY_LEN]);
        key.copy_from_slice(&hash);
        Ok(key)
    }

    /// Key derivation used by pmanager <= 0.9.7. Only used to read legacy
    /// databases during migration. It uses a fixed salt and squeezes the
    /// argon2 output through md5, never use it to encrypt anything.
    pub fn derive_legacy_key(password: &str) -> Result<Zeroizing<[u8; KEY_LEN]>> {
        let config = Config {
            variant: Variant::Argon2id,
            version: Version::Version13,
            mem_cost: 16000,
            time_cost: 20,
            lanes: 4,
            thread_mode: ThreadMode::Parallel,
            secret: &[],
            ad: &[],
            hash_length: 256,
        };

        let hash = Zeroizing::new(argon2::hash_encoded(
            password.as_bytes(),
            b"1111111111111111",
            &config,
        )?);

        // the legacy key is the md5 digest in lowercase hex, used as raw bytes.
        let digest = Zeroizing::new(format!("{:x}", md5::compute(hash.as_bytes())));

        let mut key = Zeroizing::new([0u8; KEY_LEN]);
        key.copy_from_slice(digest.as_bytes());
        Ok(key)
    }
}
