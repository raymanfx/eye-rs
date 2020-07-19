#[cfg(feature = "jpeg")]
pub mod jpeg;
pub mod rgb;

use std::io;

use ffimage::packed::{DynamicImageBuffer, DynamicImageView};

use crate::format::{FourCC, PixelFormat};

pub struct Converter {}

impl Converter {
    pub fn convert(
        src: &DynamicImageView,
        src_fmt: PixelFormat,
        dst: &mut DynamicImageBuffer,
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
