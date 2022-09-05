use std::fs;
use std::io::prelude::*;
use std::path::Path;

use std::path::PathBuf;

use dirs;

// TODO make this a generic function so it can write any data 
// inside a file regardless of its type.
pub fn create_file(path_string: &String, data: &String) {
    let path = Path::new(&path_string);
    
    // display is a helper struct for safely printing paths
    let display = path.display();

    // open a file in write only mode
    let mut file = match fs::File::create(&path) {
        Err(why) => panic!("could not create {}: {}", display, why),
        Ok(file) => file,
    };

    // write to file
    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("could not write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

pub fn create_dir(dir_path: &String) {
    match fs::create_dir_all(&dir_path) {
        Err(why) => panic!("could not create dirs {}: {}", &dir_path, why),
        Ok(_) => println!("directories created successfully : {}.",dir_path),
    };
}

pub fn get_homedir() -> PathBuf {
    let homedir = dirs::home_dir()
        .expect("could not get home directory");    

    homedir
}

pub fn does_exists(file: &String) -> bool {
    let path = Path::new(file);

    if path.exists() {
        return true
    }

    false
}
