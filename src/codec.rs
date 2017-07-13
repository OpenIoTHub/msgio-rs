use std::io::{self, Cursor};

use bytes::{BigEndian, Buf, BufMut, Bytes, BytesMut};
use varmint::{len_usize_varint, ReadVarInt, WriteVarInt};
use tokio_io::codec::{Decoder, Encoder};

use Codec;

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
pub struct LengthPrefixed(pub Prefix, pub Suffix);

impl Prefix {
    fn decode(self, src: &[u8]) -> io::Result<Option<(usize, usize)>> {
        let mut cursor = Cursor::new(src);
        Ok(match self {
            Prefix::VarInt => cursor.try_read_usize_varint()?,
            Prefix::BigEndianU32 => {
                if src.len() >= 4 {
                    Some(cursor.get_u32::<BigEndian>() as usize)
                } else {
                    None
                }
            }
        }.map(|len| (len, cursor.position() as usize)))
    }

    fn encode(self, len: usize, dst: &mut BytesMut) -> io::Result<()> {
        match self {
            Prefix::VarInt => dst.writer().write_usize_varint(len),
            Prefix::BigEndianU32 => {
                dst.put_u32::<BigEndian>(len as u32);
                Ok(())
            }
        }
    }

    fn encoded_len(self, len: usize) -> usize {
        match self {
            Prefix::VarInt => len_usize_varint(len),
            Prefix::BigEndianU32 => 4,
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

    fn validate(self, src: &[u8]) -> io::Result<()> {
        match self {
            Suffix::None => Ok(()),
            Suffix::NewLine => {
                if src[src.len() - 1] == b'\n' {
                    Ok(())
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "message did not end with '\\n'"))
                }
            }
        }
    }

    fn encode(self, dst: &mut BytesMut) {
        match self {
            Suffix::None => (),
            Suffix::NewLine => dst.put(b'\n'),
        }
    }
}

impl Encoder for LengthPrefixed {
    type Item = Bytes;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data_len = item.len() + self.1.len();
        dst.reserve(data_len + self.0.encoded_len(data_len));
        self.0.encode(data_len, dst)?;
        dst.put(item);
        self.1.encode(dst);
        Ok(())
    }
}

impl Decoder for LengthPrefixed {
    type Item = Bytes;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some((len, len_len)) = self.0.decode(src)? {
            if len + len_len <= src.len() {
                src.split_to(len_len); // discard the length
                let mut msg = src.split_to(len);
                self.1.validate(&mut msg)?;
                msg.split_off(len - self.1.len());
                Ok(Some(msg.freeze()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

impl Codec for LengthPrefixed { }
