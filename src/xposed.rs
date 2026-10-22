/**
 * xposed.rs
 * Password data breach check implementation.
 */
use isahc::prelude::*;
use serde_json::Value;
use sha3::{Digest, Keccak512};

use std::fmt::Write as FmtWrite;

use crate::Result;

pub fn xposed(password: &str) -> Result<()> {
    let mut api_endpoint = String::from("https://passwords.xposedornot.com/api/v1/pass/anon/");

    let mut hasher = Keccak512::new();
    hasher.update(password);
    let result = hasher.finalize();

    // converting hash byte values to hex
    let mut hash_in_hex = String::new();
    for b in result {
        write!(hash_in_hex, "{:02x}", b).unwrap();
    }

    let first_ten_chars = &hash_in_hex[0..10];

    api_endpoint.push_str(first_ten_chars);

    let mut response = isahc::get(api_endpoint).expect("Cannot send request to url.");
    let response_text = response.text().unwrap();

    let data: Value = serde_json::from_str(&response_text).unwrap();

    let json_data = &data.get("SearchPassAnon");
    match json_data {
        Some(_) => json_data.unwrap(),
        None => {
            eprintln!("Password not found in xposedornot data breaches.");
            return Ok(());
        }
    };

    let count_string = json_data.unwrap().get("count").unwrap().as_str().unwrap();

    let count: i32 = count_string.parse().unwrap();

    if count > 0 {
        println!(
            "Password is leaked and observed {} times in xposedornot data breaches. 
            It is advised to change your password for this domain with 'update' command, 
            and generate a random password using 'g' option when prompted.",
            count
        );
    }

    Ok(())
}
