#[cfg(feature = "jpeg")]
pub mod jpeg;
pub mod rgb;

use std::io;

use ffimage::packed::{DynamicImageBuffer, DynamicImageView};

use crate::format::FourCC;

pub struct Converter {}

impl Converter {
    pub fn convert(
        src: &DynamicImageView,
        src_fmt: FourCC,
        dst: &mut DynamicImageBuffer,
        dst_fmt: FourCC,
    ) -> io::Result<()> {
        if src_fmt == FourCC::new(b"RGB3") {
            return rgb::convert(src, dst, dst_fmt);
        }

        #[cfg(feature = "jpeg")]
        if src_fmt == FourCC::new(b"MJPG") {
            return jpeg::convert(src, dst, dst_fmt);
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot handle source format",
        ))
    }

    pub fn formats() -> Vec<(FourCC, Vec<FourCC>)> {
        let mut formats = Vec::new();

        formats.push((FourCC::new(b"RGB3"), vec![FourCC::new(b"AB24")]));

        #[cfg(feature = "jpeg")]
        formats.push((
            FourCC::new(b"MJPG"),
            vec![FourCC::new(b"RGB3"), FourCC::new(b"AB24")],
        ));

        formats
    }
}
