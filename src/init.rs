use std::path::PathBuf;
use std::str::FromStr;

use serde_derive::{Serialize, Deserialize};
use serde_json;

use crate::util;
use crate::db;

struct Config {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
pub struct DbFile {
    name: String,
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
        util::create_file(path_string, &json_data);
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

    let b: bool = PathBuf::from_str(&pmanager_folder).unwrap().is_dir();
    if b == true {
        dbg!(".pmanager directory exists skipping init process");
        return ;
    }

    let dirname = "/.pmanager";
    pmanager_folder.insert_str(pmanager_folder.len(), dirname);
    util::create_dir(&pmanager_folder);
    dbg!(&pmanager_folder);
    dbg!(dirname);
    

    // TODO "/tmp/" path will be changed with default home path.
    let config = Config::new("/pmanager_config.json",pmanager_folder);

    let db_pmanager = DbFile {
        name: String::from("db.pmanager"),
        path: db_location,
    };

    // converting db_pmanager variable to json encoded string.
    let as_json: String = match serde_json::to_string(&db_pmanager) {
        Err(why) => panic!("could not encode {} to json : {}", db_pmanager.name, why),
        Ok(as_json) => as_json,
    };

    let config_path = format!("{}{}",config.path, config.name);
    dbg!(&config_path);
    Config::create_config(&config_path, as_json);

    //db::init_db(config_path);
}
