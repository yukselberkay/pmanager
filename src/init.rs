/**
 * init.rs
 * Initializes the db.
 */

use std::path::PathBuf;
use std::fs::File;

use serde_derive::{Serialize, Deserialize};
use serde_json;

use crate::util;
use crate::db;
use libkvdb::KeyValueDB;

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

    let name = json.get("name").expect("Could not get index 'name'.")
        .as_str().unwrap();
        
    let path = json.get("path").expect("Could not get index 'path'.")
        .as_str().unwrap();

    let mut final_path = String::new();
    final_path.push_str(path);
    final_path.push_str(name);
    
    dbg!(&final_path);
    util::create_file_with_data(&final_path, &String::from("\n"));
    
}

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

impl DbFile {
    fn new(name: PathBuf, path: PathBuf) -> DbFile {
        DbFile {
            name: name,
            path: path
        }
    }

    pub fn init(mut db_location: PathBuf) {
        dbg!("init function has run.");

        // TODO ~ .
        if db_location == PathBuf::from("."){
            let default_location = util::get_homedir().join("/.pmanager");
            db_location = default_location;
        }
    
        let mut pmanager_folder = util::get_homedir().into_os_string()
            .into_string().unwrap();


        let db_name = std::path::PathBuf::from("db.encrypted");
        let b: bool = db_name.exists();
        if b == true {
            println!("Database exists, skipping initialization process.");
            return ;
        }

        let dirname = "/.pmanager";
        pmanager_folder.insert_str(pmanager_folder.len(), dirname);
        util::create_dir(&pmanager_folder);
        dbg!(&pmanager_folder);
        dbg!(dirname);
        

        // TODO "/tmp/" path will be changed with default home path.
        let config = Config::new("/pmanager_config.json",pmanager_folder);

        let db_pmanager = DbFile::new(db_name, db_location);

        let display = db_pmanager.name.display();

        // converting db_pmanager variable to json encoded string.
        let as_json: String = match serde_json::to_string(&db_pmanager) {
            Err(why) => panic!("Could not encode {} to json : {}", display, why),
            Ok(as_json) => as_json,
        };

        let config_path = format!("{}{}",config.path, config.name);
        dbg!(&config_path);
        Config::create_config(&config_path, as_json);

        init_db(config_path);

        // after initializing the db encrypt it with a master password
        let password = util::get_password(
            &String::from("Please enter your master password. This will be used to encrypt your database.")
        );
        let db_location = util::get_db_location();
        util::remove_file_from_path(&db_location);

        let f = util::create_empty_file(&db_location);

        // TODO make temp directory platform independent.
        let tmp_path = PathBuf::from("/tmp/.db.enc");
        util::create_empty_file(&tmp_path);
        
        let mut store = KeyValueDB::open(&tmp_path)
            .expect("unable to open file");
        store.load()
            .expect("unable to load data");

        let key = " ";
        let value = " ";
        store.insert(key.as_bytes(), value.as_bytes())
            .expect("An error occured while initializing database.");

        let encrypted_tmp_file = db::encrypt_db(&tmp_path, &password);
        
        let encrypted_data = util::read_as_bytes(&encrypted_tmp_file);

        util::write_bytes_to_file(f, &encrypted_data);

        util::remove_file_from_path(&tmp_path)
    }
}