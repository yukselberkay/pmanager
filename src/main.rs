/**
 * main.rs
*/

mod db;
mod password;
mod kdf;
mod aes_gcm;
mod args;
mod init;
mod util;
mod test;

use std::{str::{from_utf8, FromStr},io::Read};
use std::path::PathBuf;

use md5;

use crate::test::Xy;
use args::Subcommands;
use libkvdb::KeyValueDB;
use std::path::Path;

use crate::password::Password;
use crate::init::DbFile;

use std::fs::File;

use serde_json::{Result, Value};

//use dialoguer::{Input, Password};

//use crate::db::db_test;

fn main() {

    //test::test();

    // TODO this will be supplied by user if not supplied
    // default parameters will be used.

    //db::configuration();

    let args = args::arg_parser();
    
    let db_location = util::get_db_location();
    dbg!(&db_location);
    //let mut store = KeyValueDB::open(&path).expect("unable to open file");
    //store.load().expect("unable to load data"); 

    match &args.command {
        Some(Subcommands::Get { key }) => {
            get(&key, &db_location);
        },
        Some(Subcommands::Insert { key, value }) => {
            println!("Insert {} -> {}", key, value);
            insert(&db_location, key, value);
        },
        Some(Subcommands::Delete { key }) => {
            println!("Delete -> {}", key);
            delete(&db_location, key);
        },
        Some(Subcommands::Update { key, value }) => {
            println!("update -> {}, {}", key, value);
            update(&db_location, key, value);
        },
        Some(Subcommands::Init { db_path }) => {
            let path = PathBuf::from(db_path);
            dbg!("init path is -> {}", &db_path);
            init::init(path);
        },
        Some(Subcommands::List {  }) => {
            println!("list argument is supplied.");
            list(&db_location);
        },
        // if required arguments not supplied, 
        //prints out generated help message automatically
        None => {}        
    }

    if args.debug == true {
        //debug();
        test::test_tmp(&db_location);
        // let v1: Vec<u8> = vec![1,3,2];
        // let v2: Vec<u8> = vec![1,3,2];
        // let v3: Vec<u8> = vec![1,3,2,5];
        // let res = Xy::test(v1, (v2,v3));
        // dbg!(res.y.1);
    }


}

fn debug() {
    // dbg!("debug subcommand supplied.");
    // //let rand_pass = password::Password::genpass(32);
    // let pass = String::from("secret_pass");
    // let pass_len = pass.len();
    // let rand_pass = Password::new(pass, pass_len);
    // let derived_key: String = kdf::Argon2::derive_key(rand_pass);

    // // key must be 32 bytes
    // // should we use md5 here ??
    // let digest = md5::compute(derived_key.as_bytes());
    // let key_value = format!("{:x}", digest);
    // dbg!(&key_value);

    // let ciphertext = aes_gcm::AesGcm256::encrypt(&key_value, 
    //     String::from("unique nonce"), String::from("facebook:  12314322342321"));
    // // util::create_file(&String::from("db.pmanager"), ciphertext);
    // dbg!(&ciphertext);

    // let deciphered_values = aes_gcm::AesGcm256::decrypt(&key_value, 
    //     String::from("unique nonce"), ciphertext);

    // let plain_text = from_utf8(&deciphered_values).unwrap();
    // dbg!(&plain_text);
}

fn get(
    key: &String,
    db_location: &PathBuf
) {
    let master_password = util::get_password();
    dbg!(&master_password);

    // try to decrypt the db 
    let f = db::decrypt_db(db_location, &master_password);
    // let x = util::read_as_bytes(&f);
    // dbg!(x);
    

    println!("Get {}", key);

    let mut store = KeyValueDB::open(&f)
        .expect("unable to open file");
    store.load()
        .expect("unable to load data");

    let result = store.get(key.as_bytes()).unwrap().unwrap();
    println!("{:?}", result);

    util::remove_file_from_path(&f);
}

fn list(
    db_location: &PathBuf
) {
    let master_password = util::get_password();
    dbg!(&master_password);

    // try to decrypt the db 
    let f = db::decrypt_db(db_location, &master_password);

    let mut store = KeyValueDB::open(&f)
        .expect("unable to open file");
    store.load()
        .expect("unable to load data");

    let result = store.list();
    println!("{:?}", result);

    util::remove_file_from_path(&f); 
}

fn insert(
    db_location: &PathBuf,
    key: &String,
    value: &String
) {
    let master_password = util::get_password();
    let tmp_path = db::decrypt_db(db_location, &master_password);
    
    let mut store = KeyValueDB::open(&tmp_path)
        .expect("unable to open file");
    store.load()
        .expect("unable to load data");
        
    store.insert(key.as_bytes(), value.as_bytes())
        .expect("Unable to insert to directory");

    //remove previous database file
    util::remove_file_from_path(db_location);

    let f = util::create_empty_file(db_location);

    let encrypted_tmp_file = db::encrypt_db(&tmp_path, &master_password);
    
    let encrypted_data = util::read_as_bytes(&encrypted_tmp_file);

    util::write_bytes_to_file(f, &encrypted_data);

    util::remove_file_from_path(&tmp_path); 
}

fn delete(
    db_location: &PathBuf,
    key: &String
) {
    let master_password = util::get_password();
    let tmp_path = db::decrypt_db(db_location, &master_password);
    
    let mut store = KeyValueDB::open(&tmp_path)
        .expect("unable to open file");
    store.load()
        .expect("unable to load data");
        
    store.delete(key.as_bytes()).unwrap();

    //remove previous database file
    util::remove_file_from_path(db_location);

    let f = util::create_empty_file(db_location);

    let encrypted_tmp_file = db::encrypt_db(&tmp_path, &master_password);
    
    let encrypted_data = util::read_as_bytes(&encrypted_tmp_file);

    util::write_bytes_to_file(f, &encrypted_data);

    util::remove_file_from_path(&tmp_path)
}

fn update(
    db_location: &PathBuf,
    key: &String,
    value: &String
) {
    let master_password = util::get_password();
    let tmp_path = db::decrypt_db(db_location, &master_password);
    
    let mut store = KeyValueDB::open(&tmp_path)
        .expect("unable to open file");
    store.load()
        .expect("unable to load data");
        
    store.update(key.as_bytes(), value.as_bytes())
        .expect("Unable to insert to directory");

    //remove previous database file
    util::remove_file_from_path(db_location);

    let f = util::create_empty_file(db_location);

    let encrypted_tmp_file = db::encrypt_db(&tmp_path, &master_password);
    
    let encrypted_data = util::read_as_bytes(&encrypted_tmp_file);

    util::write_bytes_to_file(f, &encrypted_data);

    util::remove_file_from_path(&tmp_path); 
}