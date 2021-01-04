mod rgb;

#[cfg(feature = "jpeg")]
mod jpeg;

use crate::format::{Format, FourCC, PixelFormat};

pub struct Converter {}

impl Converter {
    pub fn convert(
        src: &[u8],
        src_fmt: Format,
        dst: &mut Vec<u8>,
        dst_fmt: Format,
    ) -> Result<(), &'static str> {
        if src_fmt.pixfmt == PixelFormat::Rgb(24) {
            return rgb::convert(src, src_fmt, dst, dst_fmt);
        }

        #[cfg(feature = "jpeg")]
        if src_fmt.pixfmt == PixelFormat::Custom(FourCC::new(b"MJPG")) {
            return jpeg::convert(src, src_fmt, dst, dst_fmt);
        }

        Err("cannot handle source format")
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
