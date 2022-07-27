use clap::{Arg, App, SubCommand, ArgMatches};

pub fn arg_parse() -> ArgMatches<'static> {
    let matches = App::new("pmanager")
        .version("0.1.0")
        .author("yukselberkay")
        .about("Password manager")
        .subcommand(SubCommand::with_name("debug"))
        .subcommand(SubCommand::with_name("init_db"))
        .subcommand(SubCommand::with_name("add_entry")
            .arg(Arg::with_name("entry_name")
                .takes_value(true)
                .help("entry to be added")))                
        .subcommand(SubCommand::with_name("get_entry")
            .arg(Arg::with_name("entry_name")
                .takes_value(true)
                .help("entry to be retrieved")))
        .subcommand(SubCommand::with_name("edit_entry")
            .arg(Arg::with_name("entry_name")
                .takes_value(true)
                .help("entry to be edited")))

        .get_matches();

    matches
}
