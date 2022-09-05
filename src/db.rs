/**
 * log structured append only key value database
 */

use dirs;
use std::fs::File;
use std::fs;

use serde_json;

use crate::util;

pub fn db_test(get: &String) {
    dbg!(get);

    let db_name = "db.pmanager";

    let path = std::path::Path::new(&db_name);

    // let mut store = 

}

pub fn init_db(config_path: String) {
    // parse and read the config file and get db name and db location
    // create db.pmanager according to config file
    //let config_path = "pmanager_config.json";

    let file = match File::open(&config_path) {
        Err(why) => panic!("could not open file {}: {}", config_path, why),
        Ok(file) => file,
    };

    let json: serde_json::Value = match serde_json::from_reader(file) {
        Err(why) => panic!("could not parse json {}: {}", config_path, why),
        Ok(json) => json,
    };

    let name = json.get("name").expect("could not get index name.")
        .as_str().unwrap();
        
    let path = json.get("path").expect("could not get index path.")
        .as_str().unwrap();

    let mut final_path = String::new();
    final_path.push_str(path);
    final_path.push_str(name);
    
    dbg!(&final_path);
    util::create_file(&final_path, &String::from("\n"));
    
}
