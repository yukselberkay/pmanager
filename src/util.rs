/**
 * util.rs
 * Utility functions to avoid code reuse.
 */
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use ::dialoguer::Input;
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroizing;

use crate::password::{MAX_GENERATED_LEN, MIN_GENERATED_LEN};
use crate::{DbFile, Result, CONF_FILE_EXT, CONF_NAME, DIR_NAME};

/// Creates a new file that is only accessible by the current user.
/// Fails if the file already exists, so it never follows a planted symlink.
pub fn create_private_file(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

/// Creates a directory (and its parents) that is only accessible by the current user.
/// Permissions of directories that already exist are left untouched.
pub fn create_private_dir(path: &Path) -> std::io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

/// Restricts an existing file or directory we own to the current user.
pub fn restrict_permissions(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if path.is_dir() { 0o700 } else { 0o600 };
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Replaces the contents of `path` with `data` without ever leaving a
/// partially written file behind: the data is written to a temporary file in
/// the same directory, flushed to disk and then renamed over the target.
pub fn write_file_atomic(path: &Path, data: &[u8]) -> Result<()> {
    // write next to the real file if the path is a symlink, instead of replacing the link.
    let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let dir = parent_dir(&target);
    let file_name = target
        .file_name()
        .ok_or_else(|| format!("Invalid file path {}", target.display()))?
        .to_string_lossy();

    let tmp_path = dir.join(format!(".{}.tmp-{:016x}", file_name, OsRng.next_u64()));
    let write_result = (|| -> std::io::Result<()> {
        let mut file = create_private_file(&tmp_path)?;
        file.write_all(data)?;
        file.sync_all()?;
        fs::rename(&tmp_path, &target)
    })();

    if let Err(why) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!("Could not write {}: {}", target.display(), why).into());
    }

    // make the rename itself durable.
    #[cfg(unix)]
    if let Ok(d) = File::open(&dir) {
        let _ = d.sync_all();
    }

    Ok(())
}

pub fn parent_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

pub fn get_homedir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| "Could not get home directory".into())
}

/// Turns the path given to `init` into an absolute path.
pub fn expand_path(path: &str) -> Result<PathBuf> {
    let expanded = if path == "~" {
        get_homedir()?
    } else if let Some(rest) = path.strip_prefix("~/") {
        get_homedir()?.join(rest)
    } else {
        PathBuf::from(path)
    };

    if expanded.is_absolute() {
        Ok(expanded)
    } else {
        Ok(std::env::current_dir()?.join(expanded))
    }
}

pub fn get_config_dir() -> Result<PathBuf> {
    Ok(get_homedir()?.join(DIR_NAME))
}

pub fn get_config_path() -> Result<PathBuf> {
    let mut conf_path = get_config_dir()?;
    conf_path.push(CONF_NAME);
    conf_path.set_extension(CONF_FILE_EXT);
    Ok(conf_path)
}

pub fn write_config(db_file: &DbFile) -> Result<()> {
    let config_dir = get_config_dir()?;
    create_private_dir(&config_dir)?;
    restrict_permissions(&config_dir)?;

    let as_json = serde_json::to_string(db_file)?;
    write_file_atomic(&get_config_path()?, as_json.as_bytes())
}

// parses pmanager config to get the db file location
pub fn get_db_location() -> Result<PathBuf> {
    let conf_path = get_config_path()?;

    let s = match fs::read_to_string(&conf_path) {
        Ok(s) => s,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
            return Err(
                "pmanager is not initialized. Run 'pmanager init --db-path ~' first.".into(),
            )
        }
        Err(why) => return Err(format!("Could not read {}: {}", conf_path.display(), why).into()),
    };

    let d: DbFile = serde_json::from_str(&s)
        .map_err(|why| format!("Could not parse {}: {}", conf_path.display(), why))?;

    let mut db_location: PathBuf = PathBuf::new();
    db_location.push(d.path);
    db_location.push(d.name);

    Ok(db_location)
}

pub fn get_password(prompt: &str) -> Result<Zeroizing<String>> {
    let password = rpassword::prompt_password(prompt)
        .map_err(|why| format!("An error occured while getting password input: {}", why))?;

    Ok(Zeroizing::new(password))
}

pub fn get_input(prompt: &str) -> Result<String> {
    let input: String = Input::new().with_prompt(prompt).interact_text()?;

    Ok(input)
}

/// Like `get_input`, but pressing enter keeps `default`.
pub fn get_input_with_default(prompt: &str, default: &str) -> Result<String> {
    let input: String = Input::new()
        .with_prompt(prompt)
        .default(default.to_string())
        .interact_text()?;

    Ok(input)
}

pub fn get_pass_len(prompt: &str) -> Result<usize> {
    let input: usize = Input::new()
        .with_prompt(prompt)
        .validate_with(|len: &usize| -> std::result::Result<(), String> {
            if (MIN_GENERATED_LEN..=MAX_GENERATED_LEN).contains(len) {
                Ok(())
            } else {
                Err(format!(
                    "Length must be between {} and {}",
                    MIN_GENERATED_LEN, MAX_GENERATED_LEN
                ))
            }
        })
        .interact_text()?;

    Ok(input)
}

/// Asks a yes/no question. Anything other than an explicit yes is a no.
pub fn confirm(prompt: &str) -> Result<bool> {
    let choice = get_input(&format!("{} (yes/no)", prompt))?;
    let choice = choice.trim().to_lowercase();

    Ok(choice == "yes" || choice == "y")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_path_resolves_home_and_relative_paths() {
        let home = get_homedir().unwrap();
        assert_eq!(expand_path("~").unwrap(), home);
        assert_eq!(expand_path("~/vault").unwrap(), home.join("vault"));
        assert_eq!(expand_path("/abs/path").unwrap(), PathBuf::from("/abs/path"));
        assert_eq!(
            expand_path("rel").unwrap(),
            std::env::current_dir().unwrap().join("rel")
        );
    }

    #[test]
    fn atomic_write_replaces_content_and_is_private() {
        let dir = std::env::temp_dir().join(format!("pmanager-test-{:016x}", OsRng.next_u64()));
        create_private_dir(&dir).unwrap();
        let file = dir.join("data");

        write_file_atomic(&file, b"first").unwrap();
        write_file_atomic(&file, b"second").unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"second");
        // no temporary files are left behind.
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&file).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        fs::remove_dir_all(&dir).unwrap();
    }
}
