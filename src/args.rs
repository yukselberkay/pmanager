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
    /// Get value by domain from database
    Get {
        #[clap(short, long, action)]
        domain: String,
    },
    /// Insert a Domain -> user:pass pair to database
    Insert {
        #[clap(short, long, action)]
        /// Domain to be inserted
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
        /// The domain to be updated associated with the record
        domain: String,
    },
    /// Lists every record in the database
    List {},
    /// Check if a password associated with your domain is leaked (uses xposedornot api)
    Leaked {
        #[clap(short, long, action)]
        /// Input domain associated with password
        domain: String,
    },
}

pub fn arg_parser() -> Cli {
    let args: Cli = Cli::parse();
    args
}
