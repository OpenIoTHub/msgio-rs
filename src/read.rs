use std::io;
use varmint::ReadVarInt;

use MAX_MESSAGE_SIZE;

trait ReadHelper {
    fn read_be_u32(&mut self) -> io::Result<u32>;
    fn try_read_be_u32(&mut self) -> io::Result<Option<u32>>;
    fn read_lpm_data(&mut self, len: usize) -> io::Result<Vec<u8>>;
}

/// Varint-prefixed-message
pub trait ReadVpm {
    fn read_vpm(&mut self) -> io::Result<Vec<u8>>;
    fn try_read_vpm(&mut self) -> io::Result<Option<Vec<u8>>>;
}

/// length-(bigendian-u32)-prefixed-message
pub trait ReadLpm {
    fn read_lpm(&mut self) -> io::Result<Vec<u8>>;
    fn try_read_lpm(&mut self) -> io::Result<Option<Vec<u8>>>;
}

fn be_u32(bytes: [u8; 4]) -> u32 {
    ((bytes[0] as u32) << 24)
        + ((bytes[1] as u32) << 16)
        + ((bytes[2] as u32) << 8)
        + (bytes[3] as u32)
}

impl<R> ReadHelper for R where R: io::Read {
    fn read_be_u32(&mut self) -> io::Result<u32> {
        let mut bytes = [0; 4];
        try!(self.read_exact(&mut bytes));
        Ok(be_u32(bytes))
    }

    fn try_read_be_u32(&mut self) -> io::Result<Option<u32>> {
        let mut bytes = [0; 4];
        let len = try!(self.read(&mut bytes));
        if len >= 1 {
            try!(self.read_exact(&mut bytes[len..4]));
            Ok(Some(be_u32(bytes)))
        } else {
            Ok(None)
        }
    }

    fn read_lpm_data(&mut self, len: usize) -> io::Result<Vec<u8>> {
        if len > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(io::ErrorKind::Other, "message exceeded max message size"));
        }
        let mut msg = vec![0; len];
        try!(self.read_exact(&mut msg));
        Ok(msg)
    }
}

impl<R> ReadVpm for R where R: io::Read {
    fn read_vpm(&mut self) -> io::Result<Vec<u8>> {
        let len = try!(self.read_usize_varint());
        self.read_lpm_data(len)
    }

    fn try_read_vpm(&mut self) -> io::Result<Option<Vec<u8>>> {
        if let Some(len) = try!(self.try_read_usize_varint()) {
            Ok(Some(try!(self.read_lpm_data(len))))
        } else {
            Ok(None)
        }
    }
}
impl<R> ReadLpm for R where R: io::Read {
    fn read_lpm(&mut self) -> io::Result<Vec<u8>> {
        let len = try!(self.read_be_u32());
        self.read_lpm_data(len as usize)
    }

    fn try_read_lpm(&mut self) -> io::Result<Option<Vec<u8>>> {
        if let Some(len) = try!(self.try_read_be_u32()) {
            Ok(Some(try!(self.read_lpm_data(len as usize))))
        } else {
            Ok(None)
        }
    }
}
