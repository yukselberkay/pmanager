use std::path::{PathBuf};
use std::fs::read;


use crate::Password;
use crate::kdf::Argon2;
use crate::aes_gcm::AesGcm256;
use crate::util;
use md5;
#[derive(Debug)]
pub struct Xy {
    pub x: Vec<u8>,
    pub y: (Vec<u8>, Vec<u8>)
}


impl Xy {
    pub fn test(x: Vec<u8>, y: (Vec<u8>, Vec<u8>)) -> Xy {
        Xy { x: (x), y: (y) }
    }
}





pub fn test_tmp(db_location: &PathBuf) {
    dbg!("test function is called");

    let wrong_pass = String::from("master_pas");
    let len = wrong_pass.len();
    let master_password = Password::new(wrong_pass, len);
    let wrong_master_key: String = Argon2::derive_key(master_password);
    let digest = md5::compute(wrong_master_key.as_bytes());
    let wrong_key_value = format!("{:x}", digest);

    let password = String::from("master_pass");
    let len = password.len();
    let master_password = Password::new(password, len);
    let derived_master_key: String = Argon2::derive_key(master_password);

    dbg!(&derived_master_key);
    let digest = md5::compute(derived_master_key.as_bytes());
    let key_value = format!("{:x}", digest);
    dbg!(&key_value);

    let bytes = util::read_as_bytes(&db_location);

    let ciphertext = AesGcm256::encrypt_bytes(
        &key_value,
        String::from("unique nonce"),
        bytes
    ).unwrap();

    dbg!(&ciphertext);

    let file = util::create_empty_file(&PathBuf::from("db.encrypted"));

    // write to file
    util::write_bytes_to_file(file, &ciphertext);

    let dec_bytes = util::read_as_bytes(&PathBuf::from("db.encrypted")); 
   

    let decrypted_data = match AesGcm256::decrypt(
        &wrong_key_value,
        String::from("unique nonce"),
        dec_bytes
    ) {
        Ok(decrypted_data) => decrypted_data,
        Err(why) => {
            eprintln!("cannot decrypt with the master password -> {}", why);
            return;
        }
    };
    

    let file = util::create_empty_file(&PathBuf::from("db.decrypted"));

    // write to file
    util::write_bytes_to_file(file, &decrypted_data);
}