extern crate varmint;

mod read;
mod write;

const MAX_MESSAGE_SIZE: usize = 8 * 1024 * 1024;

pub use read::{ ReadLpm, ReadVpm };
pub use write::{ WriteLpm, WriteVpm };
