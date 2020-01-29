use kvs::{Error, KvStore, Result};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs", about = "A command-line key-value store client")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
    #[structopt(subcommand)]
    cmd: KvsCmd,
}

/// Possible commands to be executed
#[derive(Debug, Clone, StructOpt)]
pub enum KvsCmd {
    /// Set the value of a string key to a string
    Set {
        /// key to be set
        #[structopt(name = "KEY")]
        key: String,
        /// value to be set
        #[structopt(name = "VALUE")]
        value: String,
    },
    /// Get the string value of a given string key
    Get {
        /// key part of the pair to get
        #[structopt(name = "KEY")]
        key: String,
    },
    /// Remove a given key
    Rm {
        /// key part of the pair to remove
        #[structopt(name = "KEY")]
        key: String,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let path = std::env::current_dir()?;
    let mut kvs = KvStore::open(path)?;
    match opt.cmd {
        KvsCmd::Get { key } => {
            let value = kvs.get(key)?.unwrap_or_else(|| "Key not found".to_owned());
            println!("{}", value);
        }
        KvsCmd::Set { key, value } => {
            kvs.set(key, value)?;
        }
        KvsCmd::Rm { key } => {
            kvs.get(key.clone())?
                .ok_or_else(|| {
                    println!("Key not found");
                    Error::KeyNotFound(key.clone())
                })
                .and_then(|_| kvs.remove(key))?;
        }
    }

    Ok(())
}
