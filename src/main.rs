mod aes_gcm;
mod args;
/**
 * main.rs
*/
mod db;
mod init;
mod kdf;
mod password;
mod util;

use std::env;
use std::path::PathBuf;

use args::Subcommands;
use libkvdb::KeyValueDB;

use crate::init::DbFile;
use crate::password::Password;

const DIR_NAME: &str = ".pmanager";
const CONF_NAME: &str = "pmanager_config";

const TMP_ENC_FILE: &str = ".db_pmanager.enc";
const TMP_DEC_FILE: &str = ".db_pmanager.dec";

const DB_NAME: &str = "db.pmanager";

const CONF_FILE_EXT: &str = "json";

fn debug() {
    let dir = env::temp_dir();
    dbg!(dir);
}

fn main() {
    let args = args::arg_parser();

    match &args.command {
        Some(Subcommands::Get { domain }) => {
            let db_location = util::get_db_location();
            dbg!(&db_location);
            get(&domain, &db_location);
        }
        Some(Subcommands::Insert { domain }) => {
            let db_location = util::get_db_location();
            dbg!(&db_location);
            println!("Insert {} ", domain);
            insert(&db_location, domain);
        }
        Some(Subcommands::Delete { domain }) => {
            let db_location = util::get_db_location();
            dbg!(&db_location);
            println!("Delete -> {}", domain);
            delete(&db_location, domain);
        }
        Some(Subcommands::Update { domain }) => {
            let db_location = util::get_db_location();
            dbg!(&db_location);
            println!("update -> {}", domain);
            update(&db_location, domain);
        }
        Some(Subcommands::Init { db_path }) => {
            let path = PathBuf::from(db_path);
            dbg!("init path is -> {}", &db_path);
            DbFile::init(path);
        }
        Some(Subcommands::List {}) => {
            let db_location = util::get_db_location();
            dbg!(&db_location);
            println!("list argument is supplied.");
            list(&db_location);
        }
        // if required arguments not supplied,
        //prints out generated help message automatically
        None => {}
    }

    if args.debug == true {
        debug();
    }
}

fn get(domain: &String, db_location: &PathBuf) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    dbg!(&master_password);

    // try to decrypt the db
    let f = db::decrypt_db(db_location, &master_password);

    println!("Get {}", domain);

    let mut store = KeyValueDB::open_and_load(&f);

    let result = match store.get(domain.as_bytes()) {
        Ok(None) => {
            eprintln!("Specified domain not found");
            return;
        }
        Ok(result) => result.unwrap(),
        Err(_) => panic!("An error occured while getting data from database."),
    };

    let res_string = String::from_utf8_lossy(&result);

    println!("{}", res_string);

    util::remove_file_from_path(&f);
}

fn list(db_location: &PathBuf) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    dbg!(&master_password);

    // try to decrypt the db
    let f = db::decrypt_db(db_location, &master_password);

    let mut store = KeyValueDB::open_and_load(&f);

    let result = store.list();
    println!("{:?}", result);

    util::remove_file_from_path(&f);
}

fn insert(db_location: &PathBuf, domain: &String) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    let tmp_path = db::decrypt_db(db_location, &master_password);

    let mut prompt = String::from("Please enter your username for ");
    prompt.push_str(&domain);
    let username = util::get_input(&prompt);

    let mut prompt = String::from("Enter your password for ");
    prompt.push_str(&username);
    prompt.push_str(" : ");
    prompt.push_str(" (type 'generate' to generate a random password): ");
    let mut password = util::get_password(&prompt);

    if password == "generate" {
        let prompt: String =
            String::from("Enter the length of the password you want to generate (8-128) ");
        let size: usize = util::get_pass_len(&prompt);
        let random_pass = Password::generate(size);
        password = random_pass.pass;
    }

    let mut res = String::new();
    res.push_str(&username);
    res.push_str(" -> ");
    res.push_str(&password);

    let mut store = KeyValueDB::open_and_load(&tmp_path);

    store
        .insert(domain.as_bytes(), res.as_bytes())
        .expect("Unable to insert to database");

    util::update_encrypted_database_entries(db_location, &master_password, &tmp_path);
}

fn delete(db_location: &PathBuf, domain: &String) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    let tmp_path = db::decrypt_db(db_location, &master_password);

    let mut store = KeyValueDB::open_and_load(&tmp_path);

    let mut prompt = String::from("Are you sure you want to delete entry -> ");
    prompt.push_str(&domain);
    prompt.push_str(" (yes/no)");

    let choice = util::get_input(&prompt);
    if choice == "no" {
        return;
    }

    store.delete(domain.as_bytes()).unwrap();

    util::update_encrypted_database_entries(db_location, &master_password, &tmp_path);
}

fn update(db_location: &PathBuf, domain: &String) {
    let master_password = util::get_password(&String::from("Enter your master password: "));
    let tmp_path = db::decrypt_db(db_location, &master_password);

    let mut prompt = String::from("Please enter your username for ");
    prompt.push_str(&domain);
    let username = util::get_input(&prompt);

    let mut prompt = String::from("Enter your password for ");
    prompt.push_str(&username);
    prompt.push_str(" (type 'generate' to generate a random password)");
    let mut password = util::get_password(&prompt);

    if password == "generate" {
        let prompt: String =
            String::from("Enter the length of the password you want to generate (8-128)");
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
    prompt.push_str(&domain);
    prompt.push_str(" (yes/no)");

    let choice = util::get_input(&prompt);
    if choice == "no" {
        return;
    }

    store
        .update(domain.as_bytes(), res.as_bytes())
        .expect("Unable to insert to directory");

    util::update_encrypted_database_entries(db_location, &master_password, &tmp_path);
}
