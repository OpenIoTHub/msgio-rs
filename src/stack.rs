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

    pub fn split(self) -> (T, S) {
        println!("self.decode_buffer: {:?}", self.decode_buffer);
        assert!(self.decode_buffer.is_empty());
        (self.upper, self.lower)
    }
}

impl<T, S> Decoder for Stacked<T, S>
    where T: Encoder<Error=io::Error> + Decoder<Error=io::Error>,
          S: Encoder<Item=Bytes, Error=io::Error> + Decoder<Item=Bytes, Error=io::Error>,
          <T as Decoder>::Item: ::std::fmt::Debug
{
    type Item = <T as Decoder>::Item;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        println!("stack::decode({:?})", src);
        // Need to give upper codec a chance to decode first in case there
        // were multiple upper items in the last lower item.
        if let Some(result) = self.upper.decode(&mut self.decode_buffer)? {
            println!("upper decoded {:?}", result);
            Ok(Some(result))
        } else {
            if let Some(bytes) = self.lower.decode(src)? {
                println!("lower decoded {:?}", bytes);
                // New lower item, append it to the buffer and give upper codec
                // a chance to decode it
                self.decode_buffer.reserve(bytes.len());
                self.decode_buffer.put(bytes);
                if let Some(result) = self.upper.decode(&mut self.decode_buffer)? {
                    println!("upper decoded {:?}", result);
                    Ok(Some(result))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
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
