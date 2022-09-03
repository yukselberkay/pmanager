use clap::{Parser, Subcommand};
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Db file path
    #[clap(short, long)]
    pub path: String,
    #[clap(subcommand)]
    pub command: Option<Subcommands>, // not required argument
}

#[derive(Subcommand)]
pub enum Subcommands {
    /// Get value by key from database
    Get {
        #[clap(short, long, action)]
        key: String,
    },
    /// Insert a key value pair to database
    Insert {
        #[clap(short, long, action)]
        key: String,
        #[clap(short, long, action)]
        value: String
    },
    /// Delete a key value pair from database
    Delete {
        #[clap(short, long, action)]
        key: String,
    },
    /// Update a record from database
    Update {
        #[clap(short, long, action)]
        key: String,
        #[clap(short, long, action)]
        value: String
    },
    /// test argument
    Find {
        #[clap(short, long, action)]
        target: String,
    }
}

pub fn arg_parser() -> Cli{
    let args = Cli::parse();
    args
}
