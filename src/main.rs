mod db;
mod password;
mod kdf;
mod aes_gcm;
mod args;
mod init;
mod util;
mod test;

use std::{str::{from_utf8, FromStr}, path::PathBuf, io::Read};

use md5;


use args::Subcommands;
use libkvdb::KeyValueDB;
use std::path::Path;

use crate::init::Database;

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
    
    if args.debug == true {
        debug();
    } 

    match &args.command {
        Some(Subcommands::Get { key }) => {
            println!("Get {}", key);
            //let result = store.get(key.as_bytes()).unwrap().unwrap();
            //println!("{:?}", result);
        },
        Some(Subcommands::Insert { key, value }) => {
            println!("Insert {} -> {}", key, value);
            //store.insert(key.as_bytes(), value.as_bytes()).unwrap();
        },
        Some(Subcommands::Delete { key }) => {
            println!("Delete -> {}", key);
        },
        Some(Subcommands::Update { key, value }) => {
            println!("update -> {}, {}", key, value);
        },
        Some(Subcommands::Init { db_path }) => {
            let path = PathBuf::from(db_path);
            dbg!("init path is -> {}", &db_path);
            init::init(path);
        },
        Some(Subcommands::List {  }) => {
            println!("list argument is supplied.");
        },
        // if required arguments not supplied, 
        //prints out generated help message automatically
        None => {}        
    }
    
    let path = get_db_location();
    dbg!(&path);

    //let mut store = KeyValueDB::open(&path).expect("unable to open file");
    //store.load().expect("unable to load data");
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

// TODO: refactoring
// parse pmanager config to get the db file location
fn get_db_location() -> PathBuf {
    let mut conf_path = PathBuf::new();

    let home_dir = util::get_homedir();

    conf_path.push(home_dir);
    conf_path.push(".pmanager");
    conf_path.push("pmanager_config");
    conf_path.set_extension("json");

    dbg!(&conf_path);

    // make pathbuf printable.
    let display = conf_path.display();

    // parse the configuration and get the db location
    let mut s = String::new();
    let mut file = match File::open(&conf_path) {
        Err(why) => panic!("could not open : {} {}", display, why),
        Ok(file) => file,
    };
   
    match file.read_to_string(&mut s) {
        Err(why) => panic!("could not read as string: {} {}", display, why),
        Ok(file) => file,
    };

    // let v: Value = serde_json::from_str(&s).unwrap();

    // let db_path = &v["path"];
    // let x = serde_json::to_string(db_path).unwrap();
    
    // let res = PathBuf::from_str(&x).unwrap();

    let d: Database = serde_json::from_str(&s).unwrap();

    dbg!(&d.path);
    
    d.path
}