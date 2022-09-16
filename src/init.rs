use std::env;
use std::fs::File;
/**
 * init.rs
 * Initializes the db.
 */
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};
use serde_json;

use crate::db;
use crate::util;
use crate::{CONF_FILE_EXT, CONF_NAME, DB_NAME, DIR_NAME, TMP_ENC_FILE};
use libkvdb::KeyValueDB;

pub fn init_db(config_path: PathBuf) {
    // parse and read the config file and get db name and db location
    // create db.pmanager according to config file

    let display = config_path.display();

    let file = match File::open(&config_path) {
        Err(why) => panic!("Could not open file {}: {}", display, why),
        Ok(file) => file,
    };

    let json: serde_json::Value = match serde_json::from_reader(file) {
        Err(why) => panic!("Could not parse json {}: {}", display, why),
        Ok(json) => json,
    };

    let name = json
        .get("name")
        .expect("Could not get index 'name'.")
        .as_str()
        .unwrap();

    let path = json
        .get("path")
        .expect("Could not get index 'path'.")
        .as_str()
        .unwrap();

    let mut final_path = PathBuf::new();
    final_path.push(path);
    final_path.push(name);

    util::create_file_with_data(&final_path, &String::from("\n"));
}

struct Config {
    name: PathBuf,
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct DbFile {
    pub name: PathBuf,
    pub path: PathBuf,
}

impl Config {
    fn new(name: PathBuf, path: PathBuf) -> Config {
        Config {
            name: name,
            path: path,
        }
    }

    fn create_config(path: &PathBuf, json_data: String) {
        util::create_file_with_data(path, &json_data);
    }
}

impl DbFile {
    fn new(name: PathBuf, path: PathBuf) -> DbFile {
        DbFile {
            name: name,
            path: path,
        }
    }

    pub fn init(mut db_location: PathBuf) {

        if db_location == PathBuf::from("~") {
            let default_location: PathBuf = util::get_homedir().join(DIR_NAME);
            db_location = default_location;
        }

        if db_location == PathBuf::from(".") {
            let mut default_location: PathBuf = env::current_dir().unwrap();
            default_location.push(DIR_NAME);
            db_location = default_location;
        }

        let mut pmanager_folder: PathBuf = util::get_homedir();

        pmanager_folder.push(DIR_NAME);

        util::create_dir(&pmanager_folder);

        let mut conf_name: PathBuf = PathBuf::new();
        conf_name.push(CONF_NAME);
        conf_name.set_extension(CONF_FILE_EXT);

        let config = Config::new(conf_name, pmanager_folder);

        let db_pmanager = DbFile::new(PathBuf::from(DB_NAME), db_location);

        let display = db_pmanager.name.display();

        // converting db_pmanager variable to json encoded string.
        let as_json: String = match serde_json::to_string(&db_pmanager) {
            Err(why) => panic!("Could not encode {} to json : {}", display, why),
            Ok(as_json) => as_json,
        };

        let mut config_path: PathBuf = PathBuf::new();
        config_path.push(config.path);
        config_path.push(config.name);

        Config::create_config(&config_path, as_json);

        let db_name: PathBuf = util::get_db_location();
        let b: bool = db_name.exists();
        if b == true {
            println!("Database exists, skipping initialization process.");
            return;
        }

        init_db(config_path);

        // after initializing the db encrypt it with a master password
        let password: String = util::get_password(
            &String::from("Please enter your master password. This password will be used to encrypt your database : ")
        );
        let db_location: PathBuf = util::get_db_location();
        util::remove_file_from_path(&db_location);

        let f: File = util::create_empty_file(&db_location);

        let mut tmp_path: PathBuf = env::temp_dir();
        tmp_path.push(TMP_ENC_FILE);
        util::create_empty_file(&tmp_path);

        let mut store: KeyValueDB = KeyValueDB::open_and_load(&tmp_path);

        let key = " ";
        let value = " ";
        store
            .insert(key.as_bytes(), value.as_bytes())
            .expect("An error occured while initializing database.");

        let encrypted_tmp_file: PathBuf = db::encrypt_db(&tmp_path, &password);

        let encrypted_data: Vec<u8> = util::read_as_bytes(&encrypted_tmp_file);

        util::write_bytes_to_file(f, &encrypted_data);

        util::remove_file_from_path(&tmp_path)
    }
}
