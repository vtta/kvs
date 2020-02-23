use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use log::info;
use structopt::StructOpt;

use kvs::{utils, Error, ErrorKind, Resp, Result};
use std::env;

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-client", about = "A command-line key-value store client")]
struct ClientOpt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
    /// IP:PORT
    #[structopt(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
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
    utils::logger("client.log")?;
    let opt = ClientOpt::from_args();
    info!("client {} target {}", env!("CARGO_PKG_VERSION"), opt.addr);
    let mut stream = TcpStream::connect(opt.addr)?;
    info!("connected to {}", stream.peer_addr()?);
    // let mut store = KvStore::open(".")?;
    // match opt.cmd {
    //     Cmd::Get { key } => {
    //         let out = store
    //             .get(key)?
    //             .unwrap_or_else(|| "Key not found".to_owned());
    //         println!("{}", out);
    //     }
    //     Cmd::Set { key, value } => store.set(key, value)?,
    //     Cmd::Rm { key } => {
    //         if let Err(e) = store.remove(key) {
    //             if let ErrorKind::KeyNotExist = e.kind() {
    //                 println!("Key not found")
    //             }
    //             return Err(e);
    //         }
    //     }
    // }
    let mut args = Vec::new();
    for arg in env::args() {
        args.push(Resp::Simple(arg));
    }
    stream.write_all(&Resp::Array(args).ser()?[..])?;

    let mut buf = [0u8; 1024];
    let _len = stream.read(&mut buf)?;
    let respond = Resp::de(&buf[..])?;
    match respond {
        Resp::Simple(s) => {
            println!("{}", s);
        }
        Resp::Error(s) => {
            println!("{}", s);
            return Err(Error::from(ErrorKind::InvalidCommand));
        }
        _ => {}
    }
    Ok(())
}
