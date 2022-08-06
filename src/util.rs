use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn create_file(path_string: &String, data: String) {
    let path = Path::new(&path_string);
    
    // display is a helper struct for safely printing paths
    let display = path.display();

    // open a file in write only mode
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