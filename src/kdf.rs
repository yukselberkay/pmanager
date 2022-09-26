use crate::password::Password;
/**
 * kdf.rs
 * Functions responsible for deriving a key.
 */
use argon2::{self, Config, ThreadMode, Variant, Version};

pub struct Argon2 {
    variant: Variant,
    version: Version,
    mem_cost: u32,
    time_cost: u32,
    thread_mode: ThreadMode,
    hash_length: u32,
}

impl Argon2 {
    pub fn derive_key(pass: Password) -> String {
        let argon2 = Argon2 {
            variant: Variant::Argon2id,
            version: Version::Version13,
            mem_cost: 16000, // 16kb
            time_cost: 20,   // 20 iterations
            thread_mode: ThreadMode::Parallel,
            hash_length: 256,
        };

        let password = pass.pass.as_bytes();
        let salt = String::from("1111111111111111");

        let config = Config {
            variant: argon2.variant,
            version: argon2.version,
            mem_cost: argon2.mem_cost,
            time_cost: argon2.time_cost,
            lanes: 4,
            thread_mode: argon2.thread_mode,
            secret: &[],
            ad: &[],
            hash_length: argon2.hash_length,
        };

        let hash = argon2::hash_encoded(password, salt.as_bytes(), &config)
            .expect("Cannot create a hash with given parameters.");

        // verify if the created hash is valid
        argon2::verify_encoded(&hash, password).expect("Final hash is not valid argon2.");

        hash
    }
}
