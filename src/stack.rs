use std::io;

use bytes::{BufMut, Bytes, BytesMut};
use tokio_io::codec::{Decoder, Encoder};

use Codec;

pub struct Stacked<T, S>
    where T: Encoder<Error=io::Error> + Decoder<Error=io::Error>,
          S: Encoder<Item=Bytes, Error=io::Error> + Decoder<Item=Bytes, Error=io::Error>
{
    upper: T,
    lower: S,
    decode_buffer: BytesMut,
    encode_buffer: BytesMut,
}

impl<T, S> Stacked<T, S>
    where T: Encoder<Error=io::Error> + Decoder<Error=io::Error>,
          S: Encoder<Item=Bytes, Error=io::Error> + Decoder<Item=Bytes, Error=io::Error>
{
    pub fn new(upper: T, lower: S) -> Stacked<T, S> {
        Stacked {
            upper,
            lower,
            decode_buffer: BytesMut::new(),
            encode_buffer: BytesMut::new(),
        }
    }
}

impl<T, S> Decoder for Stacked<T, S>
    where T: Encoder<Error=io::Error> + Decoder<Error=io::Error>,
          S: Encoder<Item=Bytes, Error=io::Error> + Decoder<Item=Bytes, Error=io::Error>
{
    type Item = <T as Decoder>::Item;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(bytes) = self.lower.decode(src)? {
            self.decode_buffer.reserve(bytes.len());
            self.decode_buffer.put(bytes);
        }
        self.upper.decode(&mut self.decode_buffer)
    }
}

impl<T, S> Encoder for Stacked<T, S>
    where T: Encoder<Error=io::Error> + Decoder<Error=io::Error>,
          S: Encoder<Item=Bytes, Error=io::Error> + Decoder<Item=Bytes, Error=io::Error>
{
    type Item = <T as Encoder>::Item;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.upper.encode(item, &mut self.encode_buffer)?;
        self.lower.encode(self.encode_buffer.take().freeze(), dst)
    }
}

impl<T, S> Codec for Stacked<T, S>
    where T: Codec,
          S: Encoder<Item=Bytes, Error=io::Error> + Decoder<Item=Bytes, Error=io::Error>
{
}
