extern crate varmint;
extern crate byteorder;
extern crate tokio_core;

mod vpm;
mod lpm;

pub use vpm::VpmCodec;
pub use lpm::LpmCodec;
