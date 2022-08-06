use toml::toml;
use std::fs;

use crate::util;

pub fn test() {
    println!("test function has run.");


    let config = toml! {
        [database]
        location = "/home/test"
    };

    println!("{:?}", config);

    // combine two strings
    let mut x = String::from("test");
    x.insert_str(x.len(), &String::from("ing"));
    dbg!(x);


    let homedir = util::get_homedir();
    dbg!(homedir);

}