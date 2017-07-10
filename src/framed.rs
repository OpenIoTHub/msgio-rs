use std::io;

use futures::{Async, AsyncSink, StartSend, Stream, Sink, Poll};
use tokio_core::io::{EasyBuf, Codec};

use MsgIo;

pub struct MsgFramed<S: MsgIo, T: Codec<In=Vec<u8>, Out=Vec<u8>>> {
    transport: S,
    codec: T,
    in_buffer: EasyBuf,
    out_buffer_size: usize,
}

impl<S: MsgIo, T: Codec<In=Vec<u8>, Out=Vec<u8>>> MsgFramed<S, T> {
    pub fn new(transport: S, codec: T) -> MsgFramed<S, T> {
        MsgFramed {
            transport,
            codec,
            in_buffer: EasyBuf::new(),
            out_buffer_size: 1024,
        }
    }

    pub fn into_inner(self) -> S {
        self.transport
    }
}

impl<S: MsgIo, T: Codec<In=Vec<u8>, Out=Vec<u8>>> Stream for MsgFramed<S, T> {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while let Some(buffer) = try_ready!(self.transport.poll()) {
            self.in_buffer.get_mut().extend_from_slice(&buffer);
            if let Some(item) = self.codec.decode(&mut self.in_buffer)? {
                return Ok(Async::Ready(Some(item)));
            }
        }
        Ok(Async::Ready(None))
    }
}

impl<S: MsgIo, T: Codec<In=Vec<u8>, Out=Vec<u8>>> Sink for MsgFramed<S, T> {
    type SinkItem = Vec<u8>;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let mut buffer = Vec::with_capacity(self.out_buffer_size);
        self.codec.encode(item, &mut buffer)?;
        self.out_buffer_size = (self.out_buffer_size + buffer.len()) / 2;
        Ok(match self.transport.start_send(buffer)? {
            AsyncSink::Ready => AsyncSink::Ready,
            AsyncSink::NotReady(buffer) => {
                AsyncSink::NotReady(self.codec.decode(&mut EasyBuf::from(buffer)).unwrap().expect("codec is reversible"))
            }
        })
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.transport.poll_complete()
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.transport.close()
    }
}

impl<S: MsgIo, T: Codec<In=Vec<u8>, Out=Vec<u8>>> MsgIo for MsgFramed<S, T> { }