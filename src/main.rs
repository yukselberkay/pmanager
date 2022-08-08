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
use dialoguer::{Input, Password};

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

    //test::test();

    // TODO this will be supplied by user if not supplied
    // default parameters will be used.
    //let db_location = String::from("/tmp/");
    //init::init(db_location);

    let args = args::arg_parse();
    //db::configuration();

    if args.is_present("init_db") {
        db::init_db();
    }

    if args.is_present("add_entry") {
        dbg!("add_entry argument supplied.");
        //add_entry();
    }

    if args.is_present("get_entry") {
        dbg!("get_entry argument supplied");
        //get_entry();
    }
        
    if args.is_present("debug") {
        dbg!("Debug mode enabled");
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
        util::create_file(&String::from("db.pmanager"), ciphertext);
        dbg!(&ciphertext);

        let deciphered_values = aes_gcm::AesGcm256::decrypt(&key_value, String::from("unique nonce"), ciphertext);

        let plain_text = from_utf8(&deciphered_values).unwrap();
        dbg!(&plain_text);

    }

}
