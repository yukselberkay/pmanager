mod password;
mod kdf;
mod aes_gcm;
mod test;

use clap::{Arg, App};
use md5;
use dirs;
use std::fs;
use std::env;
use std::process::exit;

use dialoguer::{Input, Password};

fn init_db() {
    //get home directory
    let dir_name = ".pmanager";
    let home_dir = dirs::home_dir().unwrap();
    let db_dir = home_dir.join(dir_name);
    if !db_dir.exists() {
        fs::create_dir(&db_dir).unwrap();
        println!("Database directory created to: {}", db_dir.display());
    } else {
        println!("Database directory already exists: {}", db_dir.display());
        // create file
        let db_file = db_dir.join("db.pmanager");
        if !db_file.exists() {
            fs::File::create(&db_file).unwrap();
            println!("Database file created to: {}", db_file.display());
        } else {
            println!("Database file already exists: {}", db_file.display());
        }
    }
}

fn add_entry() {
    let appname: String = Input::new()
        .with_prompt("Application name ")
        .interact()
        .unwrap();

    dbg!(&appname);

    let username: String = Input::new()
        .with_prompt("Username ")
        .interact()
        .unwrap();

    dbg!(&username);

    let password = Password::new()
        .with_prompt("Password ")
        .with_confirmation("Confirm password", "Passwords mismatching")
        .interact()
        .unwrap();

    dbg!(&password);
}

fn get_entry() {
    let password = Password::new()
        .with_prompt("Enter your master password ")
        .with_confirmation("Confirm master password", "Passwords mismatching")
        .interact()
        .unwrap();
}

fn main() {
    let matches = App::new("pmanager")
        .version("0.1.0")
        .author("yukselberkay")
        .about("Password manager")
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Enable debug mode"))
        .arg(Arg::with_name("initdb")
            .short("i")
            .long("init-db")
            .help("Initialize database"))
        .arg(Arg::with_name("add_entry")
            .short("a")
            .long("add-entry")
            .help("Add entry to database"))
        .arg(Arg::with_name("get_entry")
            .short("g")
            .long("get-entry")
            .help("Get entry from database"))
        .arg(Arg::with_name("edit entry")
            .short("e")
            .long("edit-entry")
            .help("Edit entry from database"))
        .get_matches();

    let arg_count: usize = env::args().count();
    if arg_count > 2 {
        println!("Only one argument is allowed at a time.");
        exit(1);
    }

    if matches.is_present("initdb") {
        init_db();
    }

    if matches.is_present("add_entry") {
        add_entry();
    }

    if matches.is_present("get_entry") {
        get_entry();
    }
        
    if matches.is_present("debug") {
        println!("Debug mode enabled");
        let rand_pass = password::Password::genpass(32);
        let derived_key: String = kdf::Argon2::derive_key(rand_pass);
    
        // key must be 32 bytes
        let digest = md5::compute(derived_key.as_bytes());
        let key_value = format!("{:x}", digest);
        dbg!(&key_value);
    
        let ciphertext = aes_gcm::AesGcm256::encrypt(&key_value, String::from("unique nonce"), String::from("test"));

        let plaintext = aes_gcm::AesGcm256::decrypt(key_value, String::from("unique nonce"), ciphertext);

        test::test();

        dbg!(&plaintext);

    }

}
