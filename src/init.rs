/**
 * init.rs
 * Initializes the db.
 */

use std::path::PathBuf;
use std::str::FromStr;

use serde_derive::{Serialize, Deserialize};
use serde_json;

use crate::util;

// pub fn init_db(config_path: String) {
//     // parse and read the config file and get db name and db location
//     // create db.pmanager according to config file
//     //let config_path = "pmanager_config.json";

//     let file = match File::open(&config_path) {
//         Err(why) => panic!("could not open file {}: {}", config_path, why),
//         Ok(file) => file,
//     };

//     let json: serde_json::Value = match serde_json::from_reader(file) {
//         Err(why) => panic!("could not parse json {}: {}", config_path, why),
//         Ok(json) => json,
//     };

//     let name = json.get("name").expect("could not get index name.")
//         .as_str().unwrap();
        
//     let path = json.get("path").expect("could not get index path.")
//         .as_str().unwrap();

//     let mut final_path = String::new();
//     final_path.push_str(path);
//     final_path.push_str(name);
    
//     dbg!(&final_path);
//     util::create_file_with_data(&final_path, &String::from("\n"));
    
// }

struct Config {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
pub struct DbFile {
    pub name: PathBuf,
    pub path: PathBuf,
}

impl Config {
    fn new(name: &str, path: String) -> Config {
        Config {
            name: String::from(name),
            path: path,
        }
    }

    fn create_config(path_string: &String, json_data: String) {
        util::create_file_with_data(path_string, &json_data);
    }
}

pub fn init(mut db_location: PathBuf) {
    dbg!("init function has run.");

    if db_location == PathBuf::from("."){
        let default_location = util::get_homedir().join("/.pmanager");
        db_location = default_location;
    }
 
    let mut pmanager_folder = util::get_homedir().into_os_string()
        .into_string().unwrap();

    // let b: bool = PathBuf::from_str(&pmanager_folder).unwrap().is_dir();
    // if b == true {
    //     dbg!(".pmanager directory exists skipping init process");
    //     return ;
    // }

    let dirname = "/.pmanager";
    pmanager_folder.insert_str(pmanager_folder.len(), dirname);
    util::create_dir(&pmanager_folder);
    dbg!(&pmanager_folder);
    dbg!(dirname);
    

    // TODO "/tmp/" path will be changed with default home path.
    let config = Config::new("/pmanager_config.json",pmanager_folder);

    let db_pmanager = DbFile {
        name: PathBuf::from_str("db.pmanager").unwrap(),
        path: db_location,
    };

    let display = db_pmanager.name.display();

    // converting db_pmanager variable to json encoded string.
    let as_json: String = match serde_json::to_string(&db_pmanager) {
        Err(why) => panic!("could not encode {} to json : {}", display, why),
        Ok(as_json) => as_json,
    };

    let config_path = format!("{}{}",config.path, config.name);
    dbg!(&config_path);
    Config::create_config(&config_path, as_json);

    //db::init_db(config_path);
}
