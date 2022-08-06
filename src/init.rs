use serde_derive::{Serialize};
use serde_json;

use crate::util;
use crate::db;

struct Config {
    name: String,
    path: String,
}

#[derive(Serialize)]
struct Database {
    name: String,
    path: String,
}

impl Config {
    fn new(name: &str, path: String) -> Config {
        Config {
            name: String::from(name),
            path: path,
        }
    }

    fn create_config(path_string: &String, json_data: String) {
        util::create_file(path_string, json_data);
    }
}

pub fn init(db_location: String) {
    dbg!("init function has run.");

    // TODO "/tmp/" path will be changed with default home path.
    let config = Config::new("pmanager_config.json",
        String::from("/tmp/"));

    let db_pmanager = Database {
        name: String::from("db.pmanager"),
        path: db_location,
    };

    // converting db_pmanager variable to json encoded string.
    let as_json: String = match serde_json::to_string(&db_pmanager) {
        Err(why) => panic!("could not encode {} to json : {}", db_pmanager.name, why),
        Ok(as_json) => as_json,
    };

    let config_path = format!("{}{}",config.path, config.name);
    Config::create_config(&config_path, as_json);

    db::create_db(config_path);
}
