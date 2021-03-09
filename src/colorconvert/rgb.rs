use ffimage::color::{Bgr, Rgb};
use ffimage::core::{Pixel, TryConvert};
use ffimage::packed::generic::{ImageBuffer, ImageView};

use crate::format::{ImageFormat, PixelFormat};

fn _convert<DP: Pixel + From<Rgb<u8>>>(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut ImageBuffer<DP>,
) -> Result<(), &'static str> {
    let rgb = match ImageView::<Rgb<u8>>::new(src, src_fmt.width, src_fmt.height) {
        Some(view) => view,
        None => return Err("failed to create RGB view"),
    };

    match rgb.try_convert(dst) {
        Ok(()) => Ok(()),
        Err(_) => Err("failed to convert RGB"),
    }
}

pub fn convert_to_bgr(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let mut bgr = ImageBuffer::<Bgr<u8>>::new(src_fmt.width, src_fmt.height);
    match _convert(src, src_fmt, &mut bgr) {
        Ok(()) => {
            *dst = bgr.into_vec();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn convert(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
    dst_fmt: &PixelFormat,
) -> Result<(), &'static str> {
    match dst_fmt {
        PixelFormat::Bgr(24) => convert_to_bgr(src, src_fmt, dst),
        _ => Err("cannot handle target format"),
    }
}
