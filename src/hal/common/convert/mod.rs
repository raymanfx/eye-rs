mod rgb;

#[cfg(feature = "jpeg")]
mod jpeg;

use std::io;

use ffimage::packed::dynamic::{ImageBuffer, ImageView};

use crate::format::{FourCC, PixelFormat};

pub struct Converter {}

impl Converter {
    pub fn convert(
        src: &ImageView,
        src_fmt: PixelFormat,
        dst: &mut ImageBuffer,
        dst_fmt: PixelFormat,
    ) -> io::Result<()> {
        if src_fmt == PixelFormat::Rgb(24) {
            return rgb::convert(src, dst, dst_fmt);
        }

        #[cfg(feature = "jpeg")]
        if src_fmt == PixelFormat::Custom(FourCC::new(b"MJPG")) {
            return jpeg::convert(src, dst, dst_fmt);
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot handle source format",
        ))
    }

    pub fn formats() -> Vec<(PixelFormat, Vec<PixelFormat>)> {
        let mut formats = Vec::new();

        formats.push((
            PixelFormat::Rgb(24),
            vec![PixelFormat::Bgra(32), PixelFormat::Rgba(32)],
        ));

        #[cfg(feature = "jpeg")]
        formats.push((
            PixelFormat::Custom(FourCC::new(b"MJPG")),
            vec![
                PixelFormat::Rgb(24),
                PixelFormat::Bgra(32),
                PixelFormat::Rgba(32),
            ],
        ));

        formats
    }
}
