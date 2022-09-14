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

use std::path::PathBuf;

use args::Subcommands;
use libkvdb::KeyValueDB;

use crate::password::Password;
use crate::init::DbFile;

const DIR_NAME: &str = ".pmanager";
const CONF_NAME: &str = "pmanager_config";

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
        Some(Subcommands::Insert { key }) => {
            println!("Insert {} ", key);
            insert(&db_location, key);
        },
        Some(Subcommands::Delete { key }) => {
            println!("Delete -> {}", key);
            delete(&db_location, key);
        },
        Some(Subcommands::Update { key }) => {
            println!("update -> {}", key);
            update(&db_location, key);
        },
        Some(Subcommands::Init { db_path }) => {
            let path = PathBuf::from(db_path);
            dbg!("init path is -> {}", &db_path);
            DbFile::init(path);
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
        //test::test_tmp(&db_location);
        // let v1: Vec<u8> = vec![1,3,2];
        // let v2: Vec<u8> = vec![1,3,2];
        // let v3: Vec<u8> = vec![1,3,2,5];
        // let res = Xy::test(v1, (v2,v3));
        // dbg!(res.y.1);

    }


}

// fn debug() {
//     // dbg!("debug subcommand supplied.");
//     // //let rand_pass = password::Password::genpass(32);
//     // let pass = String::from("secret_pass");
//     // let pass_len = pass.len();
//     // let rand_pass = Password::new(pass, pass_len);
//     // let derived_key: String = kdf::Argon2::derive_key(rand_pass);

//     // // key must be 32 bytes
//     // // should we use md5 here ??
//     // let digest = md5::compute(derived_key.as_bytes());
//     // let key_value = format!("{:x}", digest);
//     // dbg!(&key_value);

//     // let ciphertext = aes_gcm::AesGcm256::encrypt(&key_value, 
//     //     String::from("unique nonce"), String::from("facebook:  12314322342321"));
//     // // util::create_file(&String::from("db.pmanager"), ciphertext);
//     // dbg!(&ciphertext);

//     // let deciphered_values = aes_gcm::AesGcm256::decrypt(&key_value, 
//     //     String::from("unique nonce"), ciphertext);

//     // let plain_text = from_utf8(&deciphered_values).unwrap();
//     // dbg!(&plain_text);
// }

fn get(
    key: &String,
    db_location: &PathBuf
) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    dbg!(&master_password);

    // try to decrypt the db 
    let f = db::decrypt_db(db_location, &master_password);
    
    println!("Get {}", key);

    let mut store = KeyValueDB::open_and_load(&f);

    let result = match store.get(key.as_bytes()) {
        Ok(None) => {
            eprintln!("Specified key not found");
            return;
        },
        Ok(result) => result.unwrap(),
        Err(_) => panic!("An error occured while getting data from database."),
    };
    
    let res_string = String::from_utf8_lossy(&result);

    println!("{}", res_string);

    util::remove_file_from_path(&f);
}

fn list(
    db_location: &PathBuf
) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    dbg!(&master_password);

    // try to decrypt the db 
    let f = db::decrypt_db(db_location, &master_password);

    let mut store = KeyValueDB::open_and_load(&f);

    let result = store.list();
    println!("{:?}", result);

    util::remove_file_from_path(&f); 
}

fn insert(
    db_location: &PathBuf,
    key: &String,
) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    let tmp_path = db::decrypt_db(db_location, &master_password);

    let mut prompt = String::from("Please enter your username for ");
    prompt.push_str(&key);
    let username = util::get_input(&prompt);

    let mut prompt = String::from("Enter your password for ");
    prompt.push_str(&username);
    prompt.push_str(" : ");
    prompt.push_str(" (type 'generate' to generate a random password): ");
    let mut password = util::get_password(&prompt);

    if password == "generate" {
        let prompt: String = String::from("Enter the length of the password you want to generate (8-128) ");
        let size: usize = util::get_pass_len(&prompt);
        let random_pass = Password::generate(size);
        password = random_pass.pass;
    }

    let mut res = String::new();
    res.push_str(&username);
    res.push_str(" -> ");
    res.push_str(&password);
        
    let mut store = KeyValueDB::open_and_load(&tmp_path);
   
    store.insert(key.as_bytes(), res.as_bytes())
        .expect("Unable to insert to database");
    
    util::update_encrypted_database_entries(db_location, &master_password, &tmp_path);
}

fn delete(
    db_location: &PathBuf,
    key: &String
) {

    let master_password = util::get_password(&String::from("Enter your master password: "));
    let tmp_path = db::decrypt_db(db_location, &master_password);
    
    let mut store = KeyValueDB::open_and_load(&tmp_path);

    let mut prompt = String::from("Are you sure you want to delete entry -> ");
    prompt.push_str(&key);
    prompt.push_str(" (yes/no)");

    let choice = util::get_input(&prompt);
    if choice == "no" {return;}
 
    store.delete(key.as_bytes()).unwrap();

    //remove previous database file
    util::remove_file_from_path(db_location);

    let f = util::create_empty_file(db_location);

    let encrypted_tmp_file = db::encrypt_db(&tmp_path, &master_password);
    
    let encrypted_data = util::read_as_bytes(&encrypted_tmp_file);

    util::write_bytes_to_file(f, &encrypted_data);

    util::remove_file_from_path(&tmp_path);
}

fn update(
    db_location: &PathBuf,
    key: &String,
) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    let tmp_path = db::decrypt_db(db_location, &master_password);

    let mut prompt = String::from("Please enter your username for ");
    prompt.push_str(&key);
    let username = util::get_input(&prompt);

    let mut prompt = String::from("Enter your password for ");
    prompt.push_str(&username);
    prompt.push_str(" (type 'generate' to generate a random password)");
    let mut password = util::get_password(&prompt);

  
    if password == "generate" {
        let prompt: String = String::from("Enter the length of the password you want to generate (8-128)");
        let size: usize = util::get_pass_len(&prompt);
        let random_pass = Password::generate(size);
        password = random_pass.pass;
    }

    let mut res = String::new();
    res.push_str(&username);
    res.push_str(" -> ");
    res.push_str(&password);
        
    let mut store = KeyValueDB::open_and_load(&tmp_path);
   
    let mut prompt = String::from("Are you sure you want to update -> ");
    prompt.push_str(&key);
    prompt.push_str(" (yes/no)");
    
    let choice = util::get_input(&prompt);
    if choice == "no" {return;}

    store.update(key.as_bytes(), res.as_bytes())
        .expect("Unable to insert to directory"); 

    //remove previous database file
    util::remove_file_from_path(db_location);

    let f = util::create_empty_file(db_location);

    let encrypted_tmp_file = db::encrypt_db(&tmp_path, &master_password);
    
    let encrypted_data = util::read_as_bytes(&encrypted_tmp_file);

    util::write_bytes_to_file(f, &encrypted_data);

    util::remove_file_from_path(&tmp_path); 
}
