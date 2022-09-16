use std::io;

use openpnp_capture as pnp;

use crate::buffer::Buffer;
use crate::error::Result;
use crate::traits::Stream;
use crate::{Error, ErrorKind};

pub struct Handle {
    inner: pnp::Stream,
    buffer: Vec<u8>,
}

impl Handle {
    pub fn new(dev: &pnp::Device, fmt: &pnp::Format) -> io::Result<Self> {
        let pnp_stream = match pnp::Stream::new(dev, fmt) {
            Some(stream) => stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to open pnp stream",
                ))
            }
        };

        Ok(Handle {
            inner: pnp_stream,
            buffer: Vec::new(),
        })
    }
}

impl<'a> Stream<'a> for Handle {
    type Item = Result<Buffer<'a>>;

    fn next(&'a mut self) -> Option<Self::Item> {
        while !self.inner.poll() { /* busy loop */ }
        match self.inner.read(&mut self.buffer) {
            Ok(()) => {}
            Err(e) => return Some(Err(Error::new(ErrorKind::Other, e))),
        }

        Some(Ok(Buffer::from(&self.buffer[..])))
    }
}
