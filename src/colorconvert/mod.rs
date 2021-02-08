mod rgb;

#[cfg(feature = "jpeg")]
mod jpeg;

use crate::format::{pix, ImageFormat, PixelFormat};

pub struct Converter {}

impl Converter {
    pub fn convert(
        src: &[u8],
        src_fmt: &ImageFormat,
        dst: &mut Vec<u8>,
        dst_fmt: &PixelFormat,
    ) -> Result<(), &'static str> {
        if src_fmt.pixfmt == PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)) {
            return rgb::convert(src, src_fmt, dst, dst_fmt);
        }

        #[cfg(feature = "jpeg")]
        if src_fmt.pixfmt.name() == "jpeg" {
            return jpeg::convert(src, src_fmt, dst, dst_fmt);
        }

        Err("cannot handle source format")
    }

    pub fn formats() -> Vec<(PixelFormat, Vec<PixelFormat>)> {
        let mut formats = Vec::new();

        formats.push((
            PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)),
            vec![
                PixelFormat::Uncompressed(pix::Uncompressed::Bgra(32)),
                PixelFormat::Uncompressed(pix::Uncompressed::Rgba(32)),
            ],
        ));

        #[cfg(feature = "jpeg")]
        formats.push((
            PixelFormat::Compressed(pix::Compressed::Jpeg),
            vec![
                PixelFormat::Uncompressed(pix::Uncompressed::Bgra(32)),
                PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)),
                PixelFormat::Uncompressed(pix::Uncompressed::Rgba(32)),
            ],
        ));

        formats
    }
}
