mod db;
mod password;
mod kdf;
mod aes_gcm;
mod args;
mod init;
mod util;
mod test;

use std::str::from_utf8;

use md5;

use args::Subcommands;
use libkvdb::KeyValueDB;
use std::path::Path;


//use dialoguer::{Input, Password};

//use crate::db::db_test;


fn main() {

    //test::test();

    // TODO this will be supplied by user if not supplied
    // default parameters will be used.
    let db_location = String::from(".");
    init::init(db_location);
    //db::configuration();

    let args = args::arg_parser();

    let fname = args.path;
    let path = Path::new(&fname);

    let mut store = KeyValueDB::open(path).expect("unable to open file");
    store.load().expect("unable to load data");
    
    match &args.command {
        Some(Subcommands::Get { key }) => {
            println!("Get {}", key);
            let result = store.get(key.as_bytes()).unwrap().unwrap();
            println!("{:?}", result);
        },
        Some(Subcommands::Insert { key, value }) => {
            println!("Insert {} -> {}", key, value);
            store.insert(key.as_bytes(), value.as_bytes()).unwrap();
        },
        Some(Subcommands::Delete { key }) => {
            println!("Delete -> {}", key);
        },
        Some(Subcommands::Update { key, value }) => {
            println!("update -> {}, {}", key, value);
        },
        Some(Subcommands::Find { target }) => {
            let res = store.find(target.as_bytes()).unwrap();
            println!("{:?}", res);
        }
        // prints out generated help message automatically
        None => {}        
    }
    
}

fn debug() {
    dbg!("debug subcommand supplied.");
    //let rand_pass = password::Password::genpass(32);
    let pass = String::from("secret_pass");
    let rand_pass = password::Password { pass: (pass), len: (11) };
    let derived_key: String = kdf::Argon2::derive_key(rand_pass);

    // key must be 32 bytes
    // should we use md5 here ??
    let digest = md5::compute(derived_key.as_bytes());
    let key_value = format!("{:x}", digest);
    dbg!(&key_value);

    let ciphertext = aes_gcm::AesGcm256::encrypt(&key_value, String::from("unique nonce"), String::from("facebook:  12314322342321"));
    // util::create_file(&String::from("db.pmanager"), ciphertext);
    dbg!(&ciphertext);

    let deciphered_values = aes_gcm::AesGcm256::decrypt(&key_value, String::from("unique nonce"), ciphertext);

    let plain_text = from_utf8(&deciphered_values).unwrap();
    dbg!(&plain_text);

}