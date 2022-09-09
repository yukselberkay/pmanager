

use dirs;
use std::{fs::File, io::Read};
use std::fs;
use std::path::{PathBuf, Path};
use std::process::exit;

use serde_json;

use crate::kdf::Argon2;
use crate::password::Password;
use crate::util;
use crate::aes_gcm::AesGcm256;

pub fn decrypt_db(
    db_location: &PathBuf,
    password: String,
) -> PathBuf {
    let len = password.len();
    let master_password = Password::new(password, len);
    let derived_key: String = Argon2::derive_key(master_password);

    let digest = md5::compute(derived_key.as_bytes());
    let key_value = format!("{:x}", digest);

    let bytes = util::read_as_bytes(&db_location);

    let decrypted_data = match AesGcm256::decrypt(
        &key_value,
        String::from("unique nonce"),
        bytes
    ) {
        Ok(decrypted_data) => decrypted_data,
        Err(why) => {
            eprintln!(
                "Cannot decrypt the db with the given master password -> {}", why
            );
            exit(1);
        }
    };

    // dirty solution, refactor here make filename random.
    let tmp_path = PathBuf::from(String::from("/tmp/.db.dec"));
    let f = util::create_empty_file(&tmp_path);
    util::write_bytes_to_file(f, &decrypted_data);
    
    tmp_path
}
