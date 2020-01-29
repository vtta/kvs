use kvs::{Error, KvStore, KvsCmd, Result};

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

fn main() -> Result<()> {
    let mut path = std::env::current_dir()?;
    let opt = Opt::from_args();
    match opt.cmd.clone() {
        KvsCmd::Get { key } => {
            let mut kvs = KvStore::open(path)?;
            let value = kvs.get(key)?.unwrap_or_else(|| "Key not found".to_owned());
            println!("{}", value);
        }
        KvsCmd::Set { .. } => {
            path.push(kvs::config::DB_FILE_NAME);
            let mut db_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            let ser = bson::to_bson(&opt.cmd)?;
            if let bson::Bson::Document(doc) = ser {
                bson::encode_document(&mut db_file, &doc)?;
            }
        }
        KvsCmd::Rm { key } => {
            let mut kvs = KvStore::open(path)?;
            kvs.get(key.clone())?
                .ok_or_else(|| {
                    println!("Key not found");
                    Error::InvalidCmd(opt.cmd)
                })
                .and_then(|_| kvs.remove(key))?;
        }
    }

    Ok(())
}
