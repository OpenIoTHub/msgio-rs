use std::io;
use varint::WriteVarInt;

use MAX_MESSAGE_SIZE;

trait WriteHelper {
    fn write_be_u32(&mut self, val: u32) -> io::Result<()>;
}

/// Varint-prefixed-message
pub trait WriteVpm {
    fn write_vpm(&mut self, msg: &[u8]) -> io::Result<()>;
}

/// length-(bigendian-u32)-prefixed-message
pub trait WriteLpm {
    fn write_lpm(&mut self, msg: &[u8]) -> io::Result<()>;
}

impl<W> WriteHelper for W where W: io::Write {
    fn write_be_u32(&mut self, val: u32) -> io::Result<()> {
        let bytes = [
            (val >> 24) as u8,
            (val >> 16) as u8,
            (val >> 8) as u8,
            val as u8,
        ];
        self.write_all(&bytes)
    }
}

impl<W> WriteVpm for W where W: io::Write {
    fn write_vpm(&mut self, msg: &[u8]) -> io::Result<()> {
        try!(self.write_usize_varint(msg.len()));
        try!(self.write_all(msg));
        Ok(())
    }
}

impl<W> WriteLpm for W where W: io::Write {
    fn write_lpm(&mut self, msg: &[u8]) -> io::Result<()> {
        if msg.len() > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(io::ErrorKind::Other, "message exceeded max message size"));
        }
        try!(self.write_be_u32(msg.len() as u32));
        try!(self.write_all(msg));
        Ok(())
    }
}
