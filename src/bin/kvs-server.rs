#![feature(with_options)]

use std::fs;
use std::io::{Read, Write};
use std::net::SocketAddr;

use log::info;
use structopt::StructOpt;

use kvs::{utils, Error, ErrorKind, KvStore, KvsEngine, KvsServer, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "kvs-server", about = "A command-line key-value store server")]
struct ServerOpt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
    /// Engine backend [kvs, sled]
    #[structopt(short, long)]
    engine: String,
    /// IP:PORT
    #[structopt(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

fn check(old: String) -> Result<()> {
    let mut new = String::new();
    if fs::File::open("engine")
        .and_then(|mut file| file.read_to_string(&mut new))
        .is_ok()
        && new != old
    {
        return Err(Error::from(ErrorKind::InvalidEngine));
    }
    let mut save = fs::File::create("engine")?;
    save.write_all(old.as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    utils::logger("server.log")?;
    let opt = ServerOpt::from_args();
    info!(
        "server {} with {} listen on {}",
        env!("CARGO_PKG_VERSION"),
        opt.engine.to_string(),
        opt.addr,
    );
    check(opt.engine)?;
    let store = KvStore::open(".")?;
    let mut serve = KvsServer::listen(store, opt.addr)?;
    serve.serve()?;

    Ok(())
}
