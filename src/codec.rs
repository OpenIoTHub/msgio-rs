use std::io::{ self, Cursor, Write };

use byteorder::{ BigEndian, ReadBytesExt, WriteBytesExt };
use varmint::{ ReadVarInt, WriteVarInt };
use tokio_core;
use tokio_core::io::EasyBuf;

#[derive(Copy, Clone, Debug)]
pub enum Prefix {
    VarInt,
    BigEndianU32,
}

#[derive(Copy, Clone, Debug)]
pub enum Suffix {
    None,
    NewLine,
}

#[derive(Copy, Clone, Debug)]
pub struct Codec(pub Prefix, pub Suffix);

impl Prefix {
    fn decode(self, bytes: &[u8]) -> io::Result<(usize, usize)> {
        let mut cursor = Cursor::new(bytes);
        let len = match self {
            Prefix::VarInt => cursor.read_usize_varint()?,
            Prefix::BigEndianU32 => cursor.read_u32::<BigEndian>()? as usize,
        };
        Ok((len, cursor.position() as usize))
    }

    fn encode(self, len: usize, buf: &mut Vec<u8>) -> io::Result<()> {
        match self {
            Prefix::VarInt => buf.write_usize_varint(len),
            Prefix::BigEndianU32 => buf.write_u32::<BigEndian>(len as u32),
        }
    }
}

impl Suffix {
    fn len(self) -> usize {
        match self {
            Suffix::None => 0,
            Suffix::NewLine => 1,
        }
    }

    fn validate(self, msg: &[u8]) -> io::Result<()> {
        match self {
            Suffix::None => Ok(()),
            Suffix::NewLine => {
                if msg[msg.len() - 1] == b'\n' {
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "message did not end with '\\n'"))
                }
            }
        }
    }

    fn encode(self, buf: &mut Vec<u8>) {
        match self {
            Suffix::None => (),
            Suffix::NewLine => buf.push(b'\n'),
        }
    }
}

impl tokio_core::io::Codec for Codec {
    type In = Vec<u8>;
    type Out = Vec<u8>;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        match self.0.decode(buf.as_slice()) {
            Ok((len, len_len)) => {
                if len + len_len <= buf.len() {
                    buf.drain_to(len_len); // discard the length
                    let mut msg = buf.drain_to(len);
                    self.1.validate(msg.as_slice())?;
                    msg.split_off(len - self.1.len());
                    Ok(Some(msg.as_ref().to_vec()))
                } else {
                    Ok(None)
                }
            }
            Err(err) => {
                if err.kind() == io::ErrorKind::UnexpectedEof {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        self.0.encode(msg.len() + self.1.len(), buf)?;
        buf.write_all(&msg)?;
        self.1.encode(buf);
        Ok(())
    }
}
