extern crate byteorder;
#[macro_use]
extern crate futures;
extern crate tokio_core;
extern crate varmint;

use std::io;

use futures::sink::Sink;
use futures::stream::Stream;
use tokio_core::io::{Io, Framed};

mod codec;
pub use codec::{Codec, Prefix, Suffix};

pub trait MsgIo
    : Sink<SinkItem=Vec<u8>, SinkError=io::Error>
    + Stream<Item=Vec<u8>, Error=io::Error> { }

impl<I: Io> MsgIo for Framed<I, Codec> { }
