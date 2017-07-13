use std::io;

use futures::{Async, AsyncSink, StartSend, Stream, Sink, Poll};
use bytes::{Bytes, BytesMut};

use {MsgIo, Codec};

pub struct MsgFramed<S: MsgIo, T: Codec> {
    transport: S,
    codec: T,
    in_buffer: BytesMut,
    out_buffer_size: usize,
}

impl<S: MsgIo, T: Codec> MsgFramed<S, T> {
    pub fn new(transport: S, codec: T) -> MsgFramed<S, T> {
        MsgFramed {
            transport,
            codec,
            in_buffer: BytesMut::new(),
            out_buffer_size: 1024,
        }
    }

    pub fn into_inner(self) -> S {
        self.transport
    }
}

impl<S: MsgIo, T: Codec> Stream for MsgFramed<S, T> {
    type Item = Bytes;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while let Some(buffer) = try_ready!(self.transport.poll()) {
            self.in_buffer.extend_from_slice(&buffer);
            if let Some(item) = self.codec.decode(&mut self.in_buffer)? {
                return Ok(Async::Ready(Some(item)));
            }
        }
        Ok(Async::Ready(None))
    }
}

impl<S: MsgIo, T: Codec> Sink for MsgFramed<S, T> {
    type SinkItem = Bytes;
    type SinkError = io::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let mut buffer = BytesMut::with_capacity(self.out_buffer_size);
        self.codec.encode(item, &mut buffer)?;
        self.out_buffer_size = (self.out_buffer_size + buffer.len()) / 2;
        Ok(match self.transport.start_send(buffer.freeze())? {
            AsyncSink::Ready => AsyncSink::Ready,
            AsyncSink::NotReady(buffer) => {
                AsyncSink::NotReady(self.codec.decode(&mut buffer.try_mut().unwrap()).unwrap().expect("codec is reversible"))
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

impl<S: MsgIo, T: Codec> MsgIo for MsgFramed<S, T> { }
