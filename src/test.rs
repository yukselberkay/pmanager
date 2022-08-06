use toml::toml;

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
}