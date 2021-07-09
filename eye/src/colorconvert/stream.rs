use eye_hal::buffer::Buffer;
use eye_hal::error::Result;
use eye_hal::traits::Stream;

use crate::colorconvert::codec::Codec;

/// A stream converting frames
pub struct CodecStream<S> {
    pub inner: S,
    pub codec: Box<dyn Codec + Send>,
}

impl<'a, S> Stream<'a> for CodecStream<S>
where
    S: Stream<'a, Item = Result<Buffer<'a>>>,
{
    type Item = Result<Buffer<'a>>;

    fn next(&'a mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        let inbuf = if let Ok(buf) = item {
            buf
        } else {
            return Some(item);
        };

        let mut outbuf = Buffer::from(Vec::new());
        self.codec.decode(&inbuf, &mut outbuf).unwrap();

        Some(Ok(outbuf))
    }
}
