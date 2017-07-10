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
mod framed;
pub use codec::{Codec, Prefix, Suffix};
pub use framed::MsgFramed;

pub trait MsgIo
    : Sink<SinkItem=Vec<u8>, SinkError=io::Error>
    + Stream<Item=Vec<u8>, Error=io::Error>
{
    fn framed<T: tokio_core::io::Codec<In=Vec<u8>, Out=Vec<u8>>>(self, codec: T) -> MsgFramed<Self, T> where Self: Sized {
        MsgFramed::new(self, codec)
    }
}

impl<I: Io> MsgIo for Framed<I, Codec> { }
