use structopt::StructOpt;

use kvs::{ErrorKind, KvStore, Result};

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
    cmd: Cmd,
}

#[derive(Debug, StructOpt)]
enum Cmd {
    /// Set the value of a string key to a string
    Set {
        #[structopt(name = "KEY")]
        key: String,
        #[structopt(name = "VALUE")]
        value: String,
    },
    /// Get the string value of a given string key
    Get {
        #[structopt(name = "KEY")]
        key: String,
    },
    /// Remove a given key
    Rm {
        #[structopt(name = "KEY")]
        key: String,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut store = KvStore::open(".")?;
    match opt.cmd {
        Cmd::Get { key } => {
            let out = store
                .get(key)?
                .unwrap_or_else(|| "Key not found".to_owned());
            println!("{}", out);
        }
        Cmd::Set { key, value } => store.set(key, value)?,
        Cmd::Rm { key } => {
            if let Err(e) = store.remove(key) {
                if let ErrorKind::KeyNotExist = e.kind() {
                    println!("Key not found")
                }
                return Err(e);
            }
        }
    }
    Ok(())
}
