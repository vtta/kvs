use crate::{Error, ErrorKind, Result};

use std::io::Write;
use std::str;

/// RESP protocol specifications
#[derive(Debug, PartialEq)]
pub enum Resp {
    /// bulk string with length -1
    NullBulk,
    /// array with length -1
    NullArray,
    /// simple one line string without CR or LF
    Simple(String),
    /// string of error message
    Error(String),
    /// integer
    Integer(i64),
    /// bulk string, may be used to contain binary data safely
    Bulk(Vec<u8>),
    /// recursive array
    Array(Vec<Resp>),
}

impl Resp {
    /// get the byte representation of the serialized data
    pub fn ser(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.ser_impl(&mut buf)?;
        Ok(buf)
    }

    /// deserialize data from byte buffer
    pub fn de(buf: &[u8]) -> Result<Self> {
        let (_, val) = Self::de_impl(buf)?;
        Ok(val)
    }

    fn ser_impl(&self, buf: &mut Vec<u8>) -> Result<()> {
        match self {
            Resp::NullBulk => buf.write_all(b"$-1\r\n")?,
            Resp::NullArray => buf.write_all(b"*-1\r\n")?,
            Resp::Simple(s) => {
                buf.write_all(b"+")?;
                buf.write_all(s.as_bytes())?;
                buf.write_all(b"\r\n")?;
            }
            Resp::Error(s) => {
                buf.write_all(b"-")?;
                buf.write_all(s.as_bytes())?;
                buf.write_all(b"\r\n")?;
            }
            Resp::Integer(i) => {
                buf.write_all(b":")?;
                buf.write_all(i.to_string().as_bytes())?;
                buf.write_all(b"\r\n")?;
            }
            Resp::Bulk(b) => {
                buf.write_all(b"$")?;
                buf.write_all(b.len().to_string().as_bytes())?;
                buf.write_all(b"\r\n")?;
                buf.write_all(&b)?;
                buf.write_all(b"\r\n")?;
            }
            Resp::Array(arr) => {
                buf.write_all(b"*")?;
                buf.write_all(arr.len().to_string().as_bytes())?;
                buf.write_all(b"\r\n")?;
                for val in arr {
                    val.ser_impl(&mut *buf)?;
                }
            }
        }
        Ok(())
    }

    fn de_impl(buf: &[u8]) -> Result<(usize, Self)> {
        let mut next = 0usize;
        match buf[next] {
            b'+' => {
                // simple string
                let begin = next + 1;
                let mut end = begin;
                while end < buf.len() && buf[end] != b'\r' {
                    end += 1;
                }
                if end > buf.len() {
                    return Err(Error::from(ErrorKind::InvalidResp));
                }
                next = end + 2;
                Ok((
                    next,
                    Resp::Simple(str::from_utf8(&buf[begin..end])?.to_owned()),
                ))
            }
            b'-' => {
                // error message
                let begin = next + 1;
                let mut end = begin;
                while end < buf.len() && buf[end] != b'\r' {
                    end += 1;
                }
                if end > buf.len() {
                    return Err(Error::from(ErrorKind::InvalidResp));
                }
                next = end + 2;
                Ok((
                    next,
                    Resp::Error(str::from_utf8(&buf[begin..end])?.to_owned()),
                ))
            }
            b':' => {
                // integer
                let begin = next + 1;
                let mut end = begin;
                while end < buf.len() && buf[end] != b'\r' {
                    end += 1;
                }
                if end > buf.len() {
                    return Err(Error::from(ErrorKind::InvalidResp));
                }
                next = end + 2;
                Ok((
                    next,
                    Resp::Integer(str::from_utf8(&buf[begin..end])?.parse()?),
                ))
            }
            b'$' => {
                // bulk string
                let mut begin = next + 1;
                let mut end = begin;
                while end < buf.len() && buf[end] != b'\r' {
                    end += 1;
                }
                if end > buf.len() {
                    return Err(Error::from(ErrorKind::InvalidResp));
                }
                next = end + 2;
                let len: i64 = str::from_utf8(&buf[begin..end])?.parse()?;
                if len < 0 {
                    return Ok((next, Resp::NullBulk));
                }
                begin = next;
                end = begin + len as usize;
                next = end + 2;
                Ok((next, Resp::Bulk(buf[begin..end].to_owned())))
            }
            b'*' => {
                // array
                let begin = next + 1;
                let mut end = begin;
                while end < buf.len() && buf[end] != b'\r' {
                    end += 1;
                }
                if end > buf.len() {
                    return Err(Error::from(ErrorKind::InvalidResp));
                }
                next = end + 2;
                let len: i64 = str::from_utf8(&buf[begin..end])?.parse()?;
                if len < 0 {
                    return Ok((next, Resp::NullArray));
                }
                let mut vec = Vec::new();
                for _ in 0..len {
                    let (new, val) = Self::de_impl(&buf[next..])?;
                    next += new;
                    vec.push(val);
                }
                Ok((next, Resp::Array(vec)))
            }
            _ => {
                // not gonna happen
                Err(Error::from(ErrorKind::InvalidResp))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let val = Resp::Simple("foo bar".to_owned());
        let buf = Resp::ser(&val).unwrap();
        assert_eq!(&buf, b"+foo bar\r\n");
        let de = Resp::de(&buf).unwrap();
        assert_eq!(de, val);
    }

    #[test]
    fn error() {
        let val = Resp::Error("foo bar".to_owned());
        let buf = Resp::ser(&val).unwrap();
        assert_eq!(&buf, b"-foo bar\r\n");
        let de = Resp::de(&buf).unwrap();
        assert_eq!(de, val);
    }

    #[test]
    fn int() {
        let val = Resp::Integer(1234567890);
        let buf = Resp::ser(&val).unwrap();
        assert_eq!(&buf, b":1234567890\r\n");
        let de = Resp::de(&buf).unwrap();
        assert_eq!(de, val);
    }

    #[test]
    fn bulk() {
        let mut buf = Vec::new();
        buf.write_all(b"1234567890").unwrap();
        let val = Resp::Bulk(buf);
        let buf = Resp::ser(&val).unwrap();
        assert_eq!(&buf, b"$10\r\n1234567890\r\n");
        let de = Resp::de(&buf).unwrap();
        assert_eq!(de, val);
    }

    #[test]
    fn array() {
        let mut buf = Vec::new();
        buf.write_all(b"bulk").unwrap();
        let val = Resp::Bulk(buf);

        let val = Resp::Array(vec![
            val,
            Resp::Simple("str".to_owned()),
            Resp::Error("err".to_owned()),
            Resp::Integer(1),
            Resp::NullBulk,
            Resp::NullArray,
        ]);
        let val = Resp::Array(vec![
            Resp::Simple("str".to_owned()),
            Resp::Error("err".to_owned()),
            val,
            Resp::Integer(1),
            Resp::NullBulk,
            Resp::NullArray,
        ]);
        let buf = Resp::ser(&val).unwrap();
        println!("{:?}", String::from_utf8(buf.clone()).unwrap());
        //assert_eq!(&buf, b":1234567890\r\n");
        let de = Resp::de(&buf).unwrap();
        assert_eq!(de, val);
    }
}
