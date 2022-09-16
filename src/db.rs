use std::env;
/**
 * db.rs
 * Database file/io related operations.
*/
use std::path::PathBuf;
use std::process::exit;

use crate::aes_gcm::AesGcm256;
use crate::kdf::Argon2;
use crate::password::Password;
use crate::{util, TMP_DEC_FILE, TMP_ENC_FILE};

pub fn encrypt_db(db_location: &PathBuf, password: &String) -> PathBuf {
    let len: usize = password.len();
    let x = String::from(password);
    let master_password = Password::new(x, len);
    let derived_key: String = Argon2::derive_key(master_password);

    let digest = md5::compute(derived_key.as_bytes());
    let key_value = format!("{:x}", digest);

    let bytes = util::read_as_bytes(&db_location);

    let encrypted_data =
        match AesGcm256::encrypt_bytes(&key_value, String::from("unique nonce"), bytes) {
            Ok(encrypted_data) => encrypted_data,
            Err(why) => {
                eprintln!(
                    "Cannot encrypt the db with the given master password -> {}",
                    why
                );
                exit(1);
            }
        };

    let mut tmp_path = env::temp_dir();
    tmp_path.push(TMP_ENC_FILE);

    let f = util::create_empty_file(&tmp_path);
    util::write_bytes_to_file(f, &encrypted_data);

    tmp_path
}

pub fn decrypt_db(db_location: &PathBuf, password: &String) -> PathBuf {
    let len = password.len();
    let x = String::from(password);
    let master_password = Password::new(x, len);
    let derived_key: String = Argon2::derive_key(master_password);

    let digest = md5::compute(derived_key.as_bytes());
    let key_value = format!("{:x}", digest);

    let bytes = util::read_as_bytes(&db_location);

    let decrypted_data = match AesGcm256::decrypt_bytes(&key_value, String::from("unique nonce"), bytes) {
        Ok(decrypted_data) => decrypted_data,
        Err(why) => {
            eprintln!(
                "Cannot decrypt the db with the given master password -> {}",
                why
            );
            exit(1);
        }
    };

    let mut tmp_path = env::temp_dir();
    tmp_path.push(TMP_DEC_FILE);

    let f = util::create_empty_file(&tmp_path);
    util::write_bytes_to_file(f, &decrypted_data);

    tmp_path
}
