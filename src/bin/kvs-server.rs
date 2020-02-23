#![feature(with_options)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::{error, fmt, fs, result};

use log::{debug, info};
use serde::export::Formatter;
use structopt::StructOpt;

use kvs::{utils, Error, ErrorKind, KvStore, KvsEngine, Resp, Result};

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
    engine: Engine,
    /// IP:PORT
    #[structopt(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
}

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
    /// IP:PORT
    #[structopt(short, long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,
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

#[derive(Debug, PartialEq)]
enum Engine {
    Kvs,
    Sled,
}

impl FromStr for Engine {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "kvs" => Ok(Engine::Kvs),
            "sled" => Ok(Engine::Sled),
            _ => Err(Error::from(ErrorKind::InvalidEngine)),
        }
    }
}

impl ToString for Engine {
    fn to_string(&self) -> String {
        match self {
            Engine::Kvs => "kvs".to_owned(),
            Engine::Sled => "sled".to_owned(),
        }
    }
}

fn check(e: Engine) -> Result<()> {
    let mut engine = String::new();
    if fs::File::open("engine")
        .and_then(|mut file| file.read_to_string(&mut engine))
        .is_ok()
    {
        if let Ok(engine) = Engine::from_str(&engine) {
            if engine != e {
                return Err(Error::from(ErrorKind::InvalidEngine));
            }
        }
    }
    let mut save = fs::File::create("engine")?;
    save.write_all(e.to_string().as_bytes())?;
    Ok(())
}

#[derive(Debug)]
enum RequestError {
    InvalidCommand,
    KeyNotFound,
}

impl error::Error for RequestError {}

impl RequestError {
    fn as_str(&self) -> &str {
        match self {
            RequestError::InvalidCommand => "Invalid command",
            RequestError::KeyNotFound => "Key not found",
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Error> for RequestError {
    fn from(_: Error) -> Self {
        RequestError::InvalidCommand
    }
}

fn handle(store: &mut KvStore, stream: &mut TcpStream) -> result::Result<Resp, RequestError> {
    let mut buf = [0u8; 1024];
    let _len = stream
        .read(&mut buf)
        .map_err(|_| RequestError::InvalidCommand)?;
    let request = Resp::de(&buf[..])?;
    debug!("received request {:?}", request);
    if let Resp::Array(vec) = request {
        let mut args = Vec::new();
        for val in vec {
            if let Resp::Simple(s) = val {
                args.push(s);
            }
        }
        debug!("stripped {:?}", args);
        match ClientOpt::from_iter(args).cmd {
            ClientCmd::Get { key } => Ok(Resp::Simple(
                store
                    .get(key)
                    .map_err(|_| RequestError::InvalidCommand)?
                    .ok_or(RequestError::KeyNotFound)?,
            )),
            ClientCmd::Set { key, value } => store
                .set(key, value)
                .map(|_| Resp::NullArray)
                .map_err(|_| RequestError::InvalidCommand),
            ClientCmd::Rm { key } => {
                store
                    .remove(key)
                    .map(|_| Resp::NullArray)
                    .map_err(|e| match e.kind() {
                        ErrorKind::KeyNotExist => RequestError::KeyNotFound,
                        _ => RequestError::InvalidCommand,
                    })
            }
        }
    } else {
        Err(RequestError::InvalidCommand)
    }
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
    let mut store = KvStore::open(".")?;
    let listener = TcpListener::bind(opt.addr)?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        info!("peer incoming {}", stream.peer_addr()?);
        let respond =
            handle(&mut store, &mut stream).unwrap_or_else(|e| Resp::Error(e.as_str().to_owned()));
        stream.write_all(&respond.ser()?[..])?;
    }

    Ok(())
}
