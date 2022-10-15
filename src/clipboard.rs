use std::time::Duration;
use std::thread;

use arboard::Clipboard;
use ctrlc;

const DURATION: u64 = 15;

pub fn clipboard_operation(s: &String) {
    let duration = Duration::from_secs(DURATION);

    ctrlc::set_handler(move || {
        println!("Program will terminate automatically when the clipboard operations are finished.");
    })
    .expect("An error occured during the setting of Ctrl-C handler");

    let res = s.split(" -> ");
    let pair: Vec<&str> = res.collect();


    let username = pair[0];
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(username)
        .expect("An error occured while putting data to clipboard.");
    println!("Your username is copied to the clipboard. You can paste it with (CTRL + v) It will be cleared from the clipboard in {} seconds", DURATION);
    thread::sleep(duration);
    clipboard.clear().unwrap();


    let password = pair[1];
    clipboard.set_text(password)
        .expect("An error occured while putting data to clipboard.");
    println!("Your password is copied to the clipboard. You can paste it with (CTRL + v) It will be cleared from the clipboard in {} seconds", DURATION);
    thread::sleep(duration);

    clipboard.clear().unwrap();

}