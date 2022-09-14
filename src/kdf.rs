/**
 * kdf.rs
 * Functions responsible for deriving a key.
 */

use argon2::{self, Config, ThreadMode, Variant, Version};
use crate::password::Password;

use rand::{Rng, SeedableRng};
use rand_hc::Hc128Rng;

pub struct Argon2 {
    variant: Variant,
    version: Version,
    mem_cost: u32,
    time_cost: u32,
    thread_mode: ThreadMode,
    hash_length: u32,
}

impl Argon2 {

    fn generate_salt() -> String {
        let salt_len = 16;

        let ascii_chars: &str = "!#$%&()*+,-./0123456789:;<=>?@ABCDEFGHIJKL1MNOPQRSTUVWXYZ[]^_abcdefghijklmnopqrstuvwxyz}{|~";
        let mut rng = Hc128Rng::from_entropy();

        
        let mut salt: String = String::new();
        for _ in 0..salt_len {
            let rnd_chr = rng.gen_range(0..ascii_chars.len());
            let ascii_char_byte: u8 = ascii_chars.as_bytes()[rnd_chr as usize];
            let character: char = ascii_char_byte as char;
            salt.push(character);
        }
        String::from("1111111111111111")
    }
 
    pub fn derive_key(pass: Password) -> String {        
        dbg!(&pass.pass, &pass.len);

        let argon2 = Argon2 {
            variant: Variant::Argon2id,
            version: Version::Version13,
            mem_cost: 65536, //bytes
            time_cost: 1,
            thread_mode: ThreadMode::Parallel,
            hash_length: 256,
        };
        
        let password = pass.pass.as_bytes();
        let salt = Argon2::generate_salt();
        
        let config = Config {
            variant: argon2.variant,
            version: argon2.version,
            mem_cost: argon2.mem_cost,
            time_cost: argon2.time_cost,
            lanes: 4,
            thread_mode: argon2.thread_mode,
            secret: &[],
            ad: &[],
            hash_length: argon2.hash_length
        };

        let hash = argon2::hash_encoded(password, salt.as_bytes(), &config)
            .expect("Cannot create a hash with given parameters.");
        
        // verify if the created hash is valid
        argon2::verify_encoded(&hash, password)
            .expect("Final hash is not valid argon2.");
        
        hash
    }
}
