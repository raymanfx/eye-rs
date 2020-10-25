use std::io;

use crate::format::{Format, PixelFormat};
use crate::hal::common::convert::Converter;
use crate::image::CowImage;
use crate::traits::{ImageStream, Stream};

/// A transparent wrapper for native platform streams.
pub struct TransparentStream<'a> {
    stream: Box<ImageStream<'a>>,
    format: Format,
    mapping: Option<(PixelFormat, PixelFormat)>,
}

impl<'a> TransparentStream<'a> {
    pub fn new(stream: Box<ImageStream<'a>>, format: Format) -> Self {
        TransparentStream {
            stream,
            format,
            mapping: None,
        }
    }

    /// Tell the stream to emulate a format by requesting the compatible source format from the
    /// native device.
    ///
    /// # Arguments
    ///
    /// * `src` - Source format to be set on the device
    /// * `dst` - Target format to emulate
    ///
    pub fn map(&mut self, src: PixelFormat, dst: PixelFormat) {
        self.mapping = Some((src, dst))
    }
}

impl<'a, 'b> Stream<'b> for TransparentStream<'a> {
    type Item = io::Result<CowImage<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        let image = match self.stream.next() {
            Some(res) => match res {
                Ok(img) => img,
                Err(e) => return Some(Err(e)),
            },
            None => return None,
        };

        // emulate format by converting the buffer if necessary
        if let Some(map) = self.mapping {
            let mut buf = Vec::new();
            let mut buf_fmt = self.format;
            buf_fmt.pixfmt = map.1;

            if let Err(e) = Converter::convert(image.as_bytes(), self.format, &mut buf, buf_fmt) {
                return Some(Err(io::Error::new(io::ErrorKind::Other, e)));
            }

            let mut fmt = self.format;
            fmt.pixfmt = map.1;
            Some(Ok(CowImage::from_bytes(buf.into_iter(), fmt)))
        } else {
            Some(Ok(image))
        }
    }
}
