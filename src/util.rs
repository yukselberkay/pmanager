use std::fs::{read, create_dir_all, File};
use std::path::{Path,PathBuf};
use std::io::prelude::*;
use dirs;

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

pub fn create_empty_file(path: PathBuf) -> File{
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
