use eye_hal::buffer::Buffer;
use eye_hal::error::{Error, ErrorKind, Result};
use eye_hal::format::{ImageFormat, PixelFormat};
use eye_hal::stream::Descriptor;
use eye_hal::traits::Stream;

use crate::colorconvert::Converter;

/// A stream converting frames
pub struct ConvertStream<S> {
    pub inner: S,
    pub desc: Descriptor,
    pub map: (PixelFormat, PixelFormat),
}

impl<'a, S> Stream<'a> for ConvertStream<S>
where
    S: Stream<'a, Item = Result<Buffer<'a>>>,
{
    type Item = Result<Buffer<'a>>;

    fn next(&'a mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        let frame = if let Ok(frame) = item {
            frame
        } else {
            return Some(item);
        };

        let mut buf = Vec::new();
        let src_fmt = ImageFormat::new(self.desc.width, self.desc.height, self.map.0.clone());
        let dst_fmt = ImageFormat::new(self.desc.width, self.desc.height, self.map.1.clone());
        let res = if let Err(msg) =
            Converter::convert(frame.as_bytes(), &src_fmt, &mut buf, &dst_fmt.pixfmt)
        {
            Err(Error::new(
                ErrorKind::Other,
                format!("failed to convert stream item: {}", msg),
            ))
        } else {
            Ok(Buffer::from(buf))
        };

        Some(res)
    }
}
