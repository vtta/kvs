use std::fmt::Display;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::{io, str};

use structopt::StructOpt;

use kvs::Result;

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
    /// The address to bind to
    Server {
        #[structopt(name = "ADDRESS")]
        addr: String,
    },
    /// Send a command to given address
    Client {
        #[structopt(name = "ADDRESS")]
        addr: String,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    match opt.cmd {
        Cmd::Server { addr } => server(addr)?,
        Cmd::Client { addr } => client(addr)?,
    }
    Ok(())
}

fn server(addr: impl ToSocketAddrs) -> Result<()> {
    let _listener = TcpListener::bind(addr)?;
    unimplemented!()
}

fn client(addr: impl ToSocketAddrs + Display) -> Result<()> {
    let mut stream = TcpStream::connect(&addr)?;
    let mut recv = [0u8; 512];
    loop {
        let mut line = String::new();
        let len = io::stdin().read_line(&mut line)?;
        if len == 0 {
            break;
        }
        let line = line.trim();
        let v: Vec<_> = line.split(' ').filter(|s| !s.is_empty()).collect();
        if v.len() == 0 {
            continue;
        }
        let mut buf = format!("*{}\r\n", v.len());
        for str in v.iter() {
            buf += &format!("${}\r\n{}\r\n", str.len(), str);
        }
        stream.write_all(buf.as_bytes())?;
        let len = stream.read(&mut recv)?;
        if let Ok(s) = str::from_utf8(&recv[0..len]) {
            println!("{}>{:?}", addr, s);
        }
    }
    Ok(())
}
