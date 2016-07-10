use std::io;
use varint::{ ReadVarInt, WriteVarInt };

pub trait WriteLpm {
    fn write_lpm(&mut self, msg: &[u8]) -> io::Result<()>;
}

impl<W> WriteLpm for W where W: io::Write {
    fn write_lpm(&mut self, msg: &[u8]) -> io::Result<()> {
        try!(self.write_usize_varint(msg.len() + 1));
        try!(self.write_all(msg));
        try!(self.write_all(b"\n"));
        Ok(())
    }
}
