use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{io, thread};

use log::info;
use signal_hook::SIGINT;

use crate::{utils, Error, KvsEngine, Result};

/// server
pub struct KvsServer<T: KvsEngine> {
    engine: T,
    listener: TcpListener,
}

impl<T: KvsEngine> KvsServer<T> {
    /// listen to the socket address
    pub fn listen(engine: T, addr: SocketAddr) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self { engine, listener })
    }

    /// serve
    pub fn serve(&mut self) -> Result<()> {
        self.listener.set_nonblocking(true)?;
        let term = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(SIGINT, Arc::clone(&term))?;

        for stream in self.listener.incoming() {
            match stream {
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if term.load(Ordering::SeqCst) {
                        break;
                    }
                    thread::sleep(Duration::from_secs_f64(0.1));
                    continue;
                }
                Err(e) => return Err(Error::from(e)),
                Ok(stream) => {
                    info!("peer connected {}", stream.peer_addr()?);
                    let req: utils::Request = bincode::deserialize_from(&stream)?;
                    match req {
                        utils::Request::Get(key) => {
                            info!("incoming request GET {}", key);
                            bincode::serialize_into(
                                &stream,
                                &self
                                    .engine
                                    .get(key.clone())
                                    .map(utils::Respond::Ok)
                                    .unwrap_or_else(|e| utils::Respond::Err(e.to_string())),
                            )?;
                        }
                        utils::Request::Set(key, value) => {
                            info!("incoming request SET {} {}", key, value);
                            bincode::serialize_into(
                                &stream,
                                &self
                                    .engine
                                    .set(key, value)
                                    .map(|_| utils::Respond::Ok(None))
                                    .unwrap_or_else(|e| utils::Respond::Err(e.to_string())),
                            )?;
                        }
                        utils::Request::Rm(key) => {
                            info!("incoming request RM {}", key);
                            bincode::serialize_into(
                                &stream,
                                &self
                                    .engine
                                    .remove(key)
                                    .map(|_| utils::Respond::Ok(None))
                                    .unwrap_or_else(|e| utils::Respond::Err(e.to_string())),
                            )?;
                        }
                    };
                }
            }
        }
        Ok(())
    }
}
