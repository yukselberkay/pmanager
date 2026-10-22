/**
 * init.rs
 * Initializes the db.
 */
use std::env;
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::db::Vault;
use crate::util;
use crate::{Result, DB_NAME, DIR_NAME};

#[derive(Serialize, Deserialize)]
pub struct DbFile {
    pub name: PathBuf,
    pub path: PathBuf,
}

impl DbFile {
    fn new(name: PathBuf, path: PathBuf) -> DbFile {
        DbFile {
            name: name,
            path: path,
        }
    }
}

pub fn init(db_path: &str) -> Result<()> {
    let mut db_location = PathBuf::from(db_path);
    if db_location == PathBuf::from("~") {
        db_location = util::get_homedir()?.join(DIR_NAME);
    }

    if db_location == PathBuf::from(".") {
        db_location = env::current_dir()?.join(DIR_NAME);
    }

    let db_file = DbFile::new(PathBuf::from(DB_NAME), db_location.clone());
    util::write_config(&db_file)?;

    let db_name: PathBuf = db_location.join(DB_NAME);
    if db_name.exists() {
        println!("Database exists, skipping initialization process.");
        return Ok(());
    }

    // encrypt the new db with a master password
    let password = util::get_password(
        "Please enter your master password. This password will be used to encrypt your database: ",
    )?;
    let password_verification = util::get_password("Please confirm your password: ")?;
    if *password != *password_verification {
        return Err("Password mismatch. Initialization is aborted.".into());
    }

    util::create_private_dir(&db_location)
        .map_err(|why| format!("Could not create {}: {}", db_location.display(), why))?;

    let mut vault = Vault::new(&db_name, &password)?;
    vault.save()?;

    println!("Database initialized at {}.", db_name.display());
    Ok(())
}
