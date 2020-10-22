use std::convert::TryFrom;
use std::io;

use ffimage::packed::dynamic::ImageView;

use openpnp_capture as pnp;

use crate::format::{Format, FourCC, PixelFormat};
use crate::hal::openpnp::device::PlatformDevice;
use crate::image::CowImage;
use crate::traits::{Device, Stream};

pub struct PlatformStream {
    inner: pnp::Stream,
    buffer: Vec<u8>,
    format: Format,
}

impl PlatformStream {
    pub fn new(dev: &PlatformDevice) -> io::Result<Self> {
        let format = dev.format()?;
        let pnp_fmt = pnp::Format {
            width: format.width,
            height: format.height,
            fourcc: pnp::format::FourCC::new(&FourCC::try_from(format.pixfmt).unwrap().repr),
            bpp: 0,
            fps: 0,
        };
        let pnp_stream = match pnp::Stream::new(dev.inner(), &pnp_fmt) {
            Some(stream) => stream,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to open pnp stream",
                ))
            }
        };

        Ok(PlatformStream {
            inner: pnp_stream,
            buffer: Vec::new(),
            format,
        })
    }
}

impl<'a> Stream<'a> for PlatformStream {
    type Item = io::Result<CowImage<'a>>;

    fn next(&'a mut self) -> Option<Self::Item> {
        while !self.inner.poll() { /* busy loop */ }
        match self.inner.read(&mut self.buffer) {
            Ok(()) => {}
            Err(e) => return Some(Err(e)),
        }

        let view = ImageView::with_stride(
            &self.buffer,
            self.format.width,
            self.format.height,
            self.format.stride.unwrap_or(0),
        )
        .unwrap();
        let pixfmt = PixelFormat::Rgba(24);

        Some(Ok(CowImage::from_view(view, pixfmt)))
    }
}
