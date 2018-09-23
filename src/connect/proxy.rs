use bytes::{BufMut, Bytes, BytesMut};
use std::io;
use tokio_io::_tokio_codec::{Decoder, Encoder};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct BytesCodec {
    pub count: Arc<AtomicUsize>,
}

impl BytesCodec {
    pub fn new(count: Arc<AtomicUsize>) -> Self {
        Self { count }
    }
}

impl Decoder for BytesCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<BytesMut>, io::Error> {
        if buf.len() > 0 {
            let len = buf.len();
            self.count.fetch_add(len, Ordering::Relaxed);
            Ok(Some(buf.split_to(len)))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for BytesCodec {
    type Item = Bytes;
    type Error = io::Error;

    fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.reserve(data.len());
        buf.put(data);
        Ok(())
    }
}
