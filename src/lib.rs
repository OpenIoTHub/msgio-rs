extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate varmint;

mod codec;

pub use codec::{LengthPrefixed, Prefix, Suffix};
