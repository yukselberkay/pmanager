use clap::{arg, command, Command, ArgMatches};

pub fn arg_parse() -> ArgMatches {
    let matches = command!()
        .arg(arg!(--name [name] "Optional name to operate on"))

        .subcommand(
            Command::new("debug")
                .about("Enables debug mode.")            
        )
        .subcommand(
            Command::new("db_test")
                .about("db test")
                .arg(arg!(-g --get <KEY> "Get value by key.")
                    .required(false)
                )
                .arg(arg!(-i --insert <KEY>"Insert some value with key to DB.")
                    .required(false)
                ),
        )
        .get_matches();
        
    matches
}
