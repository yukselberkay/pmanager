/**
 * util.rs
 * Utility functions to avoid code reuse.
 */

use std::fs::{remove_file, read, create_dir_all, File};
use std::path::{Path,PathBuf};
use std::io::prelude::*;
use dirs;
use rpassword;
use::dialoguer::Input;

use crate::{DbFile, DIR_NAME, CONF_NAME};

// TODO make this a generic function so it can write any data 
// inside a file regardless of its type.
pub fn create_file_with_data(path_string: &String, data: &String) {
    let path = Path::new(&path_string);
    
    // display is a helper struct for safely printing paths
    let display = path.display();

    // open a file 
    let mut file = match File::create(&path) {
        Err(why) => panic!("could not create {}: {}", display, why),
        Ok(file) => file,
    };

    // write to file
    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("could not write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

pub fn create_empty_file(path: &PathBuf) -> File{
    let display = path.display();

    let file = match File::create(&path) {
        Err(why) => panic!("cannot create file at {}: {}",display, why),
        Ok(file) => file
    };

    file
}

pub fn write_bytes_to_file(mut file: File, data: &Vec<u8>) {
    match file.write_all(data) {
        Err(why) => panic!("cannot write data to file: {}", why),
        Ok(_) => (),
    }
}

pub fn read_as_bytes(path: &PathBuf) -> Vec<u8> {
    let display = path.display();

    let bytes = match read(&path) {
        Err(why) => panic!("cannot read {}: {}", why, display),
        Ok(bytes) => bytes,
    };

    bytes
}

pub fn create_dir(dir_path: &String) {
    match create_dir_all(&dir_path) {
        Err(why) => panic!("could not create dirs {}: {}", &dir_path, why),
        Ok(_) => println!("directories created successfully : {}.",dir_path),
    };
}

pub fn get_homedir() -> PathBuf {
    let homedir = dirs::home_dir()
        .expect("could not get home directory");    

    homedir
}

// TODO: refactoring
// parse pmanager config to get the db file location
pub fn get_db_location() -> PathBuf {
    let mut conf_path = PathBuf::new();

    let home_dir = get_homedir();

    conf_path.push(home_dir);
    conf_path.push(DIR_NAME);
    conf_path.push(CONF_NAME);
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

    let d: DbFile = serde_json::from_str(&s).unwrap();

    let mut db_location: PathBuf = PathBuf::new();
    db_location.push(d.path);
    db_location.push(d.name);

    dbg!(&db_location);
    
    db_location
}

pub fn get_password(prompt: &String) -> String {
    let password = rpassword::prompt_password(prompt)
        .expect("An error occured while getting password input");

    password
}

pub fn remove_file_from_path(path: &PathBuf) {
    remove_file(path).expect("Failed to remove the file.");
}

pub fn get_input(prompt: &String) -> String {
    let input : String = Input::new()
        .with_prompt(prompt)
        .interact_text().unwrap();

    input
}