/**
 * xposed.rs
 * Password data breach check implementation.
 */
use isahc::prelude::*;
use serde_json::Value;
use sha3::{Digest, Keccak512};

use std::process::exit;
use std::{fmt::Write as FmtWrite, path::PathBuf};

use crate::db;
use crate::util;
use libkvdb::KeyValueDB;

pub fn get_password_by_domain(domain: &String, db_location: &PathBuf) -> String {
    let master_password = util::get_password(&String::from("Enter your master password: "));

    // try to decrypt the db
    let f = db::decrypt_db(db_location, &master_password);

    let mut store = KeyValueDB::open_and_load(&f);

    let result = match store.get(domain.as_bytes()) {
        Ok(None) => {
            eprintln!("Specified domain not found");
            exit(1);
        }
        Ok(result) => result.unwrap(),
        Err(_) => panic!("An error occured while getting data from database."),
    };

    let res_string = String::from_utf8_lossy(&result);

    let res = res_string.split(" -> ");

    let pair: Vec<&str> = res.collect();

    let password = pair[1];
    let pass = String::from(password);

    println!("{}", res_string);

    util::remove_file_from_path(&f);

    pass
}

pub fn xposed(domain: &String, db_location: &PathBuf) {
    let mut api_endpoint = String::from("https://passwords.xposedornot.com/api/v1/pass/anon/");

    let password = get_password_by_domain(&domain, db_location);

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
            return;
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
}
