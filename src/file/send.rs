use hyper::body::{Bytes, Sender};

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    mem,
    net::SocketAddr,
};

use super::ranges::RangesResp;
use crate::consts::MutStatic;

pub async fn send_resp<R>(mut resp: R, mut sender: Sender, addr: SocketAddr)
where
    R: NextChunk + Send + 'static,
{
    loop {
        let res = resp.next_chunk();
        match res {
            Ok(Some(chunk)) => {
                let chunk_len = chunk.len();
                let rest = sender.send_data(chunk).await;

                // debug!("{}'s chunk {}: {:?}", addr, chunk_len, rest);

                if let Err(e) = rest {
                    debug!("{}'s sender.send_data({} B): {:?}", addr, chunk_len, e);
                    break;
                }
            }
            Ok(None) => break,
            Err(e) => {
                debug!("{}'s read_a_range_chunk: {:?}", addr, e);
                sender.abort();
                break;
            }
        }
    }
    debug!("{}'s send_file_with_ranges finish", addr);
}

pub const CHUNK_SIZE: usize = 1024 * 16;

thread_local!(
    pub static BUF: MutStatic<[u8;CHUNK_SIZE]>=  MutStatic::new(unsafe {mem::zeroed()})
);

pub trait NextChunk {
    fn next_chunk(&mut self) -> io::Result<Option<Bytes>>;
}

impl NextChunk for RangesResp<File> {
    fn next_chunk(&mut self) -> io::Result<Option<Bytes>> {
        fn inner(this: &mut RangesResp<File>, buf: &mut [u8], chunk_size: usize) -> io::Result<Option<Bytes>> {
            if this.is_empty() {
                return Ok(this.form.boundary_eof().map(|s| s.into()));
            }

            let range = &mut this.form[0];

            if !range.2.is_empty() {
                let boundary_head = mem::replace(&mut range.2, String::new());
                return Ok(Some(boundary_head.into()));
            }

            let start = range.0;
            let range_size = (range.1 - range.0) as usize;
            let reserve_size = if range_size <= chunk_size {
                this.set_offset(1);
                range_size
            } else {
                range.0 += chunk_size as u64;
                chunk_size
            };

            let seeks = this.content.seek(SeekFrom::Start(start))?;
            debug_assert_eq!(seeks, start);

            let read_bytes = this.content.read(buf)?;

            if read_bytes < reserve_size {
                let msg = "read bytes's len < reserve_size ?";
                error!("{}: {} < {}", msg, read_bytes, reserve_size);
                return Err(io::Error::new(io::ErrorKind::Interrupted, msg));
            }

            // debug!("start: {}, range_size: {}, [0..{})", start, range_size, reserve_size);

            // into Bytes
            Ok(Some(buf[0..reserve_size].to_owned().into()))
        }
        BUF.with(|buf| inner(self, &mut buf.get_mut()[..], CHUNK_SIZE))
    }
}
