/**
 * migrate.rs
 * Upgrades databases created by pmanager <= 0.9.7 to the current format.
 */
use std::env;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::db::{self, Entry, Format, Vault};
use crate::util;
use crate::Result;

// Temporary files older versions left behind. The .dec file holds the whole
// database in plaintext.
const LEGACY_TMP_FILES: [&str; 2] = [".db_pmanager.enc", ".db_pmanager.dec"];

pub fn migrate() -> Result<()> {
    let db_location = util::get_db_location()?;
    let _lock = db::lock(&db_location)?;

    harden_config_permissions();
    remove_legacy_tmp_files();

    let bytes = db::read_db_file(&db_location)?;
    if db::detect_format(&bytes)? == Format::Current {
        println!(
            "The database at {} is already in the current format, nothing to migrate.",
            db_location.display()
        );
        return Ok(());
    }

    println!("Migrating {} to the new database format.", db_location.display());
    let password = util::get_password("Enter your master password: ")?;
    let mut warnings = Vec::new();
    let entries = db::read_legacy(&bytes, &password, &mut warnings)?;
    println!("Decrypted {} entries.", entries.len());

    let new_password = util::get_password(
        "Enter a new master password, or press enter to keep your current one: ",
    )?;
    let password = if new_password.is_empty() {
        password
    } else {
        let verification = util::get_password("Please confirm your new password: ")?;
        if *new_password != *verification {
            return Err("Password mismatch. Migration is aborted, nothing was changed.".into());
        }
        new_password
    };

    let backup = write_backup(&db_location, &bytes)?;

    let mut vault = Vault::new(&db_location, &password)?;
    for (domain, entry) in &entries {
        vault.insert(domain, entry)?;
    }
    vault.save()?;
    drop(vault);

    if let Err(why) = verify(&db_location, &password, &entries) {
        util::write_file_atomic(&db_location, &bytes)?;
        return Err(format!(
            "Verification of the migrated database failed ({}). The original database has been restored.",
            why
        )
        .into());
    }

    for warning in warnings {
        eprintln!("Warning: {}", warning);
    }
    println!(
        "Migration complete, {} entries were converted to the new format.",
        entries.len()
    );
    println!(
        "A backup of the old database was saved to {}. It is still protected by the old, weaker encryption. Once you have checked that everything works, delete it.",
        backup.display()
    );
    Ok(())
}

/// Saves the original file next to the database, without overwriting older backups.
fn write_backup(db_location: &Path, bytes: &[u8]) -> Result<PathBuf> {
    let file_name = db_location
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    for i in 0..100 {
        let name = match i {
            0 => format!("{}.v1.bak", file_name),
            i => format!("{}.v1.bak.{}", file_name, i),
        };
        let backup = db_location.with_file_name(name);

        let mut file = match util::create_private_file(&backup) {
            Ok(file) => file,
            Err(why) if why.kind() == ErrorKind::AlreadyExists => continue,
            Err(why) => {
                return Err(format!("Cannot create backup {}: {}", backup.display(), why).into())
            }
        };
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|why| format!("Cannot write backup {}: {}", backup.display(), why))?;
        return Ok(backup);
    }

    Err("Cannot create a backup file, too many backups exist already.".into())
}

fn verify(db_location: &Path, password: &str, entries: &[(String, Entry)]) -> Result<()> {
    let mut vault = Vault::open(db_location, password, None)?;
    if vault.domains().len() != entries.len() {
        return Err("entry count mismatch".into());
    }
    for (domain, expected) in entries {
        let entry = vault
            .get(domain)?
            .ok_or_else(|| format!("{} is missing", domain))?;
        if entry.username != expected.username || entry.password != expected.password {
            return Err(format!("{} does not match", domain).into());
        }
    }
    Ok(())
}

/// Older versions created the config directory and file readable by everyone.
fn harden_config_permissions() {
    let paths = [util::get_config_dir(), util::get_config_path()];
    for path in paths.iter().flatten() {
        if path.exists() {
            if let Err(why) = util::restrict_permissions(path) {
                eprintln!(
                    "Warning: could not restrict permissions of {}: {}",
                    path.display(),
                    why
                );
            }
        }
    }
}

fn remove_legacy_tmp_files() {
    for name in LEGACY_TMP_FILES {
        let path = env::temp_dir().join(name);
        if fs::symlink_metadata(&path).is_err() {
            continue;
        }
        match fs::remove_file(&path) {
            Ok(()) => println!("Removed leftover temporary file {}.", path.display()),
            Err(why) => eprintln!(
                "Warning: could not remove leftover temporary file {}: {}",
                path.display(),
                why
            ),
        }
    }
}
