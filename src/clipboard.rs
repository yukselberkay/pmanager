use ctrlc;

pub mod clipboards {
    use std::thread;
    use std::time::Duration;

    const DURATION: u64 = 10;

    #[cfg(any(windows))]
    pub fn clip(text_data: &str, text_type: &str) {
        use clipboard_win::{formats, set_clipboard};

        let duration = Duration::from_secs(DURATION);
        set_clipboard(formats::Unicode, text_data).unwrap();
        println!("Your {} is copied to the clipboard. You can paste it with (CTRL + v) It will be cleared from the clipboard in {} seconds", text_type, DURATION);
        thread::sleep(duration);
        //clear the OS level clipboard
        set_clipboard(formats::Unicode, "").unwrap();
    }

    #[cfg(any(unix))]
    pub fn clip(text_data: &str, text_type: &str) {
        use arboard::Clipboard;

        let duration = Duration::from_secs(DURATION);
        let mut clipboard = Clipboard::new().unwrap();
        clipboard
            .set_text(text_data)
            .expect("An error occured while copying data to clipboard.");
        println!("Your {} is copied to the clipboard. You can paste it with (CTRL + v) It will be cleared from the clipboard in {} seconds", text_type, DURATION);
        thread::sleep(duration);
        clipboard.clear().unwrap();
    }
}

pub fn clipboard_operations(user_pass_pair: &String) {
    ctrlc::set_handler(move || {
        println!(
            "Program will terminate automatically when the clipboard operations are finished."
        );
    })
    .expect("An error occured during the setting of Ctrl-C handler");

    let res = user_pass_pair.split(" -> ");
    let pair: Vec<&str> = res.collect();
    let username = pair[0];
    let password = pair[1];

    clipboards::clip(username, "username");
    clipboards::clip(password, "password");
}

pub fn clipboard_operations_password_only(user_pass_pair: &String) {
    ctrlc::set_handler(move || {
        println!(
            "Program will terminate automatically when the clipboard operations are finished."
        );
    })
    .expect("An error occured during the setting of Ctrl-C handler");

    let res = user_pass_pair.split(" -> ");
    let pair: Vec<&str> = res.collect();
    let password = pair[1];

    clipboards::clip(password, "password");
}