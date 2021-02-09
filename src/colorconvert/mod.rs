mod rgb;

#[cfg(feature = "jpeg")]
mod jpeg;

use crate::format::{ImageFormat, PixelFormat};

pub struct Converter {}

impl Converter {
    pub fn convert(
        src: &[u8],
        src_fmt: &ImageFormat,
        dst: &mut Vec<u8>,
        dst_fmt: &PixelFormat,
    ) -> Result<(), &'static str> {
        if src_fmt.pixfmt == PixelFormat::Rgb(24) {
            return rgb::convert(src, src_fmt, dst, dst_fmt);
        }

        #[cfg(feature = "jpeg")]
        if src_fmt.pixfmt == PixelFormat::Jpeg {
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
            PixelFormat::Jpeg,
            vec![
                PixelFormat::Bgra(32),
                PixelFormat::Rgb(24),
                PixelFormat::Rgba(32),
            ],
        ));

        formats
    }
}
