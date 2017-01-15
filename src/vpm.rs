use std::io::{ self, Cursor, Write };

use varmint::{ ReadVarInt, WriteVarInt };
use tokio_core::io::{ Codec, EasyBuf };

pub struct VpmCodec;

impl Codec for VpmCodec {
    type In = EasyBuf;
    type Out = Vec<u8>;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        let (len, len_len) = {
            let mut cursor = Cursor::new(&buf);
            (cursor.read_usize_varint(), cursor.position() as usize)
        };
        match len {
            Ok(len) => {
                if len + len_len < buf.len() {
                    buf.drain_to(len_len); // discard the length
                    Ok(Some(buf.drain_to(len)))
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
        buf.write_usize_varint(msg.len())?;
        buf.write_all(&msg)?;
        Ok(())
    }
}
