mod aes_gcm;
mod args;
/**
 * main.rs
*/
mod clipboard;
mod db;
mod init;
mod kdf;
mod migrate;
mod password;
mod util;
mod xposed;

use std::process::exit;

use args::Subcommands;
use zeroize::Zeroizing;

use crate::db::{Entry, Format, Vault, LEGACY_FORMAT_MESSAGE};
use crate::init::DbFile;

const DIR_NAME: &str = ".pmanager";
const CONF_NAME: &str = "pmanager_config";

const DB_NAME: &str = "db.pmanager";

const CONF_FILE_EXT: &str = "json";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let args = args::arg_parser();

    let result = match &args.command {
        Subcommands::Get { domain } => get(domain),
        Subcommands::Clip { domain } => clip_password(domain),
        Subcommands::Insert { domain } => insert(domain),
        Subcommands::Delete { domain } => delete(domain),
        Subcommands::Update { domain } => update(domain),
        Subcommands::Init { db_path } => init::init(db_path),
        Subcommands::List {} => list(),
        Subcommands::Leaked { domain } => leaked(domain),
        Subcommands::Migrate {} => migrate::migrate(),
    };

    if let Err(why) = result {
        eprintln!("Error: {}", why);
        exit(1);
    }
}

/// Asks for the master password and decrypts the database. When `for_writing`
/// is set, the database is locked against other pmanager processes until the
/// returned vault is dropped.
fn open_vault(for_writing: bool) -> Result<Vault> {
    let db_location = util::get_db_location()?;
    let lock = match for_writing {
        true => Some(db::lock(&db_location)?),
        false => None,
    };

    // do not ask for the password if the database can not be opened anyway.
    if db::detect_format(&db::read_db_file(&db_location)?)? == Format::Legacy {
        return Err(LEGACY_FORMAT_MESSAGE.into());
    }

    let master_password = util::get_password("Enter your master password: ")?;
    Vault::open(&db_location, &master_password, lock)
}

fn not_found(domain: &str) -> Box<dyn std::error::Error> {
    format!("Specified domain not found: {}", domain).into()
}

/// Looks up an entry. The decrypted database is wiped from memory before returning.
fn find_entry(domain: &str) -> Result<Entry> {
    let mut vault = open_vault(false)?;
    vault.get(domain)?.ok_or_else(|| not_found(domain))
}

/// Asks for the password of an entry, 'g' generates a random one.
fn read_entry_password(prompt: &str) -> Result<Zeroizing<String>> {
    let password = util::get_password(prompt)?;

    if password.as_str() == "g" {
        let size: usize =
            util::get_pass_len("Enter the length of the password you want to generate (8-128) ")?;
        return Ok(Zeroizing::new(password::Password::generate(size).pass));
    }
    Ok(password)
}

fn get(domain: &str) -> Result<()> {
    let entry = find_entry(domain)?;
    clipboard::clipboard_operations(&entry)
}

fn clip_password(domain: &str) -> Result<()> {
    let entry = find_entry(domain)?;
    clipboard::clipboard_operations_password_only(&entry)
}

fn leaked(domain: &str) -> Result<()> {
    let entry = find_entry(domain)?;
    xposed::xposed(&entry.password)
}

fn list() -> Result<()> {
    let mut vault = open_vault(false)?;

    let domains = vault.domains();
    if domains.is_empty() {
        println!("The database is empty.");
    }
    for domain in domains {
        if let Some(entry) = vault.get(&domain)? {
            println!("{:?} -> {:?}", domain, entry.username);
        }
    }
    Ok(())
}

fn insert(domain: &str) -> Result<()> {
    let mut vault = open_vault(true)?;

    let username = util::get_input(&format!("Please enter your username for {}", domain))?;
    let password = read_entry_password(&format!(
        "Enter your password for {} (Type 'g' to generate a random password): ",
        username
    ))?;

    let entry = Entry {
        username,
        password: password.to_string(),
    };
    vault.insert(domain, &entry)?;
    vault.save()?;
    drop(vault);

    println!("Entry for {} is saved.", domain);
    clipboard::clip(&entry.password, "password")
}

fn delete(domain: &str) -> Result<()> {
    let mut vault = open_vault(true)?;

    let choice = util::get_input(&format!(
        "Are you sure you want to delete entry -> {} (yes/no)",
        domain
    ))?;
    if choice == "no" {
        return Ok(());
    }

    vault.remove(domain)?;
    vault.save()?;
    println!("Entry for {} is deleted.", domain);
    Ok(())
}

fn update(domain: &str) -> Result<()> {
    let mut vault = open_vault(true)?;

    let username = util::get_input(&format!("Please enter your username for {}", domain))?;
    let password = read_entry_password(&format!(
        "Enter your password for {} (type 'g' to generate a random password)",
        username
    ))?;

    let entry = Entry {
        username,
        password: password.to_string(),
    };

    let choice = util::get_input(&format!(
        "Are you sure you want to update -> {} (yes/no)",
        domain
    ))?;
    if choice == "no" {
        return Ok(());
    }

    vault.insert(domain, &entry)?;
    vault.save()?;
    println!("Entry for {} is updated.", domain);
    Ok(())
}
