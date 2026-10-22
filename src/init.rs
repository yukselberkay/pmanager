/**
 * init.rs
 * Initializes the db.
 */
use std::env;
use std::fs;
use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::db::{self, Format, Vault, LEGACY_FORMAT_MESSAGE};
use crate::util;
use crate::{Result, DB_NAME, DIR_NAME};

const RECOMMENDED_PASSWORD_LEN: usize = 12;

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
    // "~" and "." keep the database in a .pmanager folder inside them.
    let db_location: PathBuf = match db_path {
        "~" => util::get_homedir()?.join(DIR_NAME),
        "." => env::current_dir()?.join(DIR_NAME),
        other => util::expand_path(other)?,
    };
    let db_file = DbFile::new(PathBuf::from(DB_NAME), db_location.clone());
    let db_name: PathBuf = db_location.join(DB_NAME);

    if db_name.exists() {
        util::write_config(&db_file)?;
        println!(
            "Database exists at {}, skipping initialization process. pmanager is configured to use it.",
            db_name.display()
        );
        if let Ok(bytes) = fs::read(&db_name) {
            if let Ok(Format::Legacy) = db::detect_format(&bytes) {
                println!("{}", LEGACY_FORMAT_MESSAGE);
            }
        }
        return Ok(());
    }

    let password = util::get_password(
        "Please enter your master password. This password will be used to encrypt your database: ",
    )?;
    if password.is_empty() {
        return Err("The master password cannot be empty. Initialization is aborted.".into());
    }
    if password.chars().count() < RECOMMENDED_PASSWORD_LEN {
        eprintln!(
            "Warning: your master password is shorter than {} characters. A longer passphrase is much harder to brute force.",
            RECOMMENDED_PASSWORD_LEN
        );
    }
    let password_verification = util::get_password("Please confirm your password: ")?;
    if *password != *password_verification {
        return Err("Password mismatch. Initialization is aborted.".into());
    }

    util::create_private_dir(&db_location)
        .map_err(|why| format!("Could not create {}: {}", db_location.display(), why))?;

    let mut vault = Vault::new(&db_name, &password)?;
    vault.save()?;

    // only point the config at the database once it exists.
    util::write_config(&db_file)?;

    println!("Database initialized at {}.", db_name.display());
    Ok(())
}
