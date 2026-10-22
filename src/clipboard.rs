/**
 * clipboard.rs
 * Copies secrets to the clipboard and clears them again after a while.
 */
use std::sync::{Mutex, Once};
use std::thread;
use std::time::Duration;

use arboard::Clipboard;
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
use arboard::SetExtLinux;
#[cfg(target_os = "macos")]
use arboard::SetExtApple;
#[cfg(windows)]
use arboard::SetExtWindows;
use zeroize::Zeroizing;

use crate::db::Entry;
use crate::Result;

const DURATION: u64 = 10;

// The secret currently on the clipboard, so that it can be cleared on Ctrl-C.
static ACTIVE_SECRET: Mutex<Option<Zeroizing<String>>> = Mutex::new(None);
static CTRLC_HANDLER: Once = Once::new();

fn install_ctrlc_handler() {
    CTRLC_HANDLER.call_once(|| {
        let result = ctrlc::set_handler(|| {
            if let Ok(mut clipboard) = Clipboard::new() {
                clear_if_ours(&mut clipboard);
            }
            eprintln!("\nInterrupted, the clipboard has been cleared.");
            std::process::exit(130);
        });
        if let Err(why) = result {
            eprintln!("Warning: could not set the Ctrl-C handler: {}", why);
        }
    });
}

/// Clears the clipboard, unless the user has copied something else since.
fn clear_if_ours(clipboard: &mut Clipboard) {
    let secret = match ACTIVE_SECRET.lock() {
        Ok(mut guard) => guard.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    let secret = match secret {
        Some(secret) => secret,
        None => return,
    };

    let still_ours = match clipboard.get_text() {
        Ok(current) => *Zeroizing::new(current) == *secret,
        // non text content was copied over our secret.
        Err(arboard::Error::ContentNotAvailable) => false,
        Err(_) => true,
    };
    if still_ours {
        if let Err(why) = clipboard.clear() {
            eprintln!("Warning: could not clear the clipboard: {}", why);
        }
    }
}

fn set_secret(clipboard: &mut Clipboard, text: &str) -> std::result::Result<(), arboard::Error> {
    let set = clipboard.set();
    // keep secrets out of clipboard managers, clipboard history and cloud sync.
    #[cfg(windows)]
    let set = set.exclude_from_monitoring();
    #[cfg(any(
        target_os = "macos",
        all(
            unix,
            not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
        )
    ))]
    let set = set.exclude_from_history();
    set.text(text)
}

pub fn clip(text_data: &str, text_type: &str) -> Result<()> {
    install_ctrlc_handler();

    let duration = Duration::from_secs(DURATION);
    let mut clipboard = Clipboard::new()
        .map_err(|why| format!("Cannot access the clipboard: {}", why))?;

    *ACTIVE_SECRET.lock().unwrap_or_else(|p| p.into_inner()) =
        Some(Zeroizing::new(text_data.to_string()));
    if let Err(why) = set_secret(&mut clipboard, text_data) {
        clear_if_ours(&mut clipboard);
        return Err(format!("An error occured while copying data to clipboard: {}", why).into());
    }

    println!("Your {} is copied to the clipboard. You can paste it with (CTRL + v) It will be cleared from the clipboard in {} seconds", text_type, DURATION);
    thread::sleep(duration);
    clear_if_ours(&mut clipboard);
    Ok(())
}

pub fn clipboard_operations(entry: &Entry) -> Result<()> {
    clip(&entry.username, "username")?;
    clip(&entry.password, "password")
}

pub fn clipboard_operations_password_only(entry: &Entry) -> Result<()> {
    clip(&entry.password, "password")
}
