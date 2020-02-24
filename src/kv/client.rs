use std::net::{SocketAddr, TcpStream};

use log::{error, info};

use crate::{utils, Error, ErrorKind, Result};

/// kvs client
pub struct KvsClient {
    stream: TcpStream,
}

impl KvsClient {
    /// connect to the given socket address
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        info!("connected to {}", stream.peer_addr()?);
        Ok(Self { stream })
    }

    /// get key
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        bincode::serialize_into(&self.stream, &utils::Request::Get(key))?;
        let res: utils::Respond = bincode::deserialize_from(&self.stream)?;
        info!("received respond {:?}", res);
        match res {
            utils::Respond::Ok(v) => Ok(v),
            utils::Respond::Err(e) => {
                error!("server responded with an error {}", e);
                Err(Error::from(ErrorKind::InvalidCommand))
            }
        }
    }

    /// set key
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        bincode::serialize_into(&self.stream, &utils::Request::Set(key, value))?;
        let res: utils::Respond = bincode::deserialize_from(&self.stream)?;
        match res {
            utils::Respond::Ok(_) => Ok(()),
            utils::Respond::Err(e) => {
                error!("server responded with an error {}", e);
                Err(Error::from(ErrorKind::InvalidCommand))
            }
        }
    }

    /// remove key
    pub fn remove(&mut self, key: String) -> Result<()> {
        bincode::serialize_into(&self.stream, &utils::Request::Rm(key))?;
        let res: utils::Respond = bincode::deserialize_from(&self.stream)?;
        match res {
            utils::Respond::Ok(_) => Ok(()),
            utils::Respond::Err(e) => {
                error!("server responded with an error {}", e);
                Err(Error::from(ErrorKind::InvalidCommand))
            }
        }
    }
}
