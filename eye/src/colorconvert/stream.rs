use eye_hal::error::Result;
use eye_hal::traits::Stream;

use crate::colorconvert::codec::Codec;

/// A stream converting frames
pub struct CodecStream<S> {
    pub inner: S,
    pub codec: Box<dyn Codec + Send>,
    pub buf: Vec<u8>,
}

impl<'a, S> Stream<'a> for CodecStream<S>
where
    S: Stream<'a, Item = Result<&'a [u8]>>,
{
    type Item = Result<&'a [u8]>;

    fn next(&'a mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        let inbuf = if let Ok(buf) = item {
            buf
        } else {
            return Some(item);
        };

        self.codec.decode(&inbuf, &mut self.buf).unwrap();
        Some(Ok(&self.buf))
    }
}
