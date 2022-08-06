use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, Display};

use serde_derive::{Serialize};
use serde_json;

#[derive(Serialize)]
struct Database {
    name: String,
    path: String,
}

fn create_file(path_string: String, data: String) {
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

fn create_config(json_data: String) {
    let config_name = String::from("pmanager_config.json");
    
    create_file(config_name, json_data);
}

fn create_db() {
    // parse and read the config file and get db name and db location
    // create db.pmanager according to config file
    let config_path = "pmanager_config.json";

    let file = match File::open(config_path) {
        Err(why) => panic!("could not open file {}: {}", config_path, why),
        Ok(file) => file,
    };

    let json: serde_json::Value = match serde_json::from_reader(file) {
        Err(why) => panic!("could not parse json {}: {}", config_path, why),
        Ok(json) => json,
    };

    let name = json.get("name").expect("could not get index name.")
        .to_string();
    let mut path = json.get("path").expect("could not get index path.")
        .to_string();
    
    path.insert_str(path.len(), &name);
    
    dbg!(path);
}

pub fn init(db_location: String) {
    dbg!("init function has run.");

    let db_pmanager = Database {
        name: String::from("db.pmanager"),
        path: db_location,
    };

    // converting db_pmanager variable to json encoded string.
    let as_json: String = match serde_json::to_string(&db_pmanager) {
        Err(why) => panic!("could not encode {} to json : {}", db_pmanager.name, why),
        Ok(as_json) => as_json,
    };

    create_config(as_json);
    create_db();
}
