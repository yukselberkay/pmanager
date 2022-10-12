use std::time::Duration;
use std::thread;

use arboard::Clipboard;
use ctrlc;

const DURATION: u64 = 15;

pub fn clipboard_operation(s: &String) {
    ctrlc::set_handler(move || {
        println!("Program will terminate automatically when the clipboard is cleared.");
    })
    .expect("Error setting Ctrl-C handler");

    let res = s.split(" -> ");
    let pair: Vec<&str> = res.collect();

    let username = pair[0];
    println!("Username -> {}", username);

    let password = pair[1];

    let mut clipboard = Clipboard::new().unwrap();

    clipboard.set_text(password).unwrap();

    println!("Your password is copied to the clipboard. You can paste it with (CTRL + v) It will be cleared from clipboard in {} seconds", DURATION);

    let duration = Duration::from_secs(DURATION);
    thread::sleep(duration);



    clipboard.clear().unwrap();

}