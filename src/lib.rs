extern crate bytes;
#[macro_use]
extern crate futures;
extern crate tokio_io;
extern crate varmint;

use std::io;

use bytes::Bytes;
use futures::sink::Sink;
use futures::stream::Stream;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Decoder, Encoder, Framed};

mod codec;
mod framed;

pub use codec::{LengthPrefixed, Prefix, Suffix};
pub use framed::MsgFramed;

pub trait Codec
    : Decoder<Item=Bytes, Error=io::Error>
    + Encoder<Item=Bytes, Error=io::Error>
{
}

pub trait MsgIo
    : Sink<SinkItem=Bytes, SinkError=io::Error>
    + Stream<Item=Bytes, Error=io::Error>
{
    fn framed<T: Codec>(self, codec: T) -> MsgFramed<Self, T> where Self: Sized {
        MsgFramed::new(self, codec)
    }
}

impl<T: AsyncRead + AsyncWrite, S: Codec> MsgIo for Framed<T, S> { }
