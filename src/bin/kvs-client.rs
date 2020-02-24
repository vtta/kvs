use std::env;
use std::net::SocketAddr;

use log::info;
use structopt::StructOpt;

use kvs::{utils, KvsClient, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-client", about = "A command-line key-value store client")]
struct ClientOpt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
    #[structopt(subcommand)]
    cmd: ClientCmd,
}

#[derive(Debug, StructOpt)]
enum ClientCmd {
    /// Set the value of a string key to a string
    Set {
        #[structopt(name = "KEY")]
        key: String,
        #[structopt(name = "VALUE")]
        value: String,
        /// IP:PORT
        #[structopt(short, long, default_value = "127.0.0.1:4000")]
        addr: SocketAddr,
    },
    /// Get the string value of a given string key
    Get {
        #[structopt(name = "KEY")]
        key: String,
        /// IP:PORT
        #[structopt(short, long, default_value = "127.0.0.1:4000")]
        addr: SocketAddr,
    },
    /// Remove a given key
    Rm {
        #[structopt(name = "KEY")]
        key: String,
        /// IP:PORT
        #[structopt(short, long, default_value = "127.0.0.1:4000")]
        addr: SocketAddr,
    },
}

fn main() -> Result<()> {
    utils::logger("client.log")?;
    let opt = ClientOpt::from_args();
    match opt.cmd {
        ClientCmd::Get { key, addr } => {
            info!("client {} target {}", env!("CARGO_PKG_VERSION"), addr);
            let mut client = KvsClient::connect(addr)?;
            let out = client
                .get(key)?
                .unwrap_or_else(|| "Key not found".to_owned());
            println!("{}", out);
        }
        ClientCmd::Set { key, value, addr } => {
            info!("client {} target {}", env!("CARGO_PKG_VERSION"), addr);
            let mut client = KvsClient::connect(addr)?;
            client.set(key, value)?;
        }
        ClientCmd::Rm { key, addr } => {
            info!("client {} target {}", env!("CARGO_PKG_VERSION"), addr);
            let mut client = KvsClient::connect(addr)?;
            client.remove(key)?;
        }
    }

    Ok(())
}
