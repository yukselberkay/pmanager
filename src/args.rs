/**
 * args.rs
 * Parses command line arguments.
 */

use clap::{Parser, Subcommand};
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(short, long, action)]
    pub debug: bool,
    #[clap(subcommand)]
    pub command: Option<Subcommands>, // not required argument
}

#[derive(Subcommand)]
pub enum Subcommands {
    /// Initialize pmanager
    Init {
        #[clap(short, long, action)]
        db_path: String,
    },
    /// Get value by key from database
    Get {
        #[clap(short, long, action)]
        domain: String,
    },
    /// Insert a key value pair to database
    Insert {
        #[clap(short, long, action)]
        /// Key to be inserted
        domain: String,
    },
    /// Delete a key value pair from database
    Delete {
        #[clap(short, long, action)]
        /// The key to be deleted associated with the record
        domain: String,
    },
    /// Update a record from database
    Update {
        #[clap(short, long, action)]
        /// The key to be updated associated with the record
        domain: String,
    },
    /// Lists every record in the database
    List {}
}

pub fn arg_parser() -> Cli{
    let args: Cli = Cli::parse();
    args
}
