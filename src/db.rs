use dirs;
use std::fs;
use config::Config;
use std::collections::HashMap;


pub fn configuration() {
    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("servers"))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    
    //println!("{:?}", settings.try_deserialize::<HashMap<String, String>>().unwrap());
    dbg!(settings);
}


pub fn init_db() {
    //get home directory
    let dir_name = ".pmanager";
    let home_dir = dirs::home_dir().unwrap();
    let db_dir = home_dir.join(dir_name);
    if !db_dir.exists() {
        fs::create_dir(&db_dir).unwrap();
        println!("Database directory created to: {}", db_dir.display());
    } else {
        println!("Database directory already exists: {}", db_dir.display());
        // create file
        let db_file = db_dir.join("db.pmanager");
        if !db_file.exists() {
            fs::File::create(&db_file).unwrap();
            println!("Database file created to: {}", db_file.display());
        } else {
            println!("Database file already exists: {}", db_file.display());
        }
    }
}