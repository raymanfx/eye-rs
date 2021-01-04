use ffimage::color::{Bgra, Rgb, Rgba};
use ffimage::core::{Pixel, TryConvert};
use ffimage::packed::generic::{ImageBuffer, ImageView};

use crate::format::{Format, PixelFormat};

fn _convert<DP: Pixel + From<Rgb<u8>>>(
    src: &[u8],
    src_fmt: Format,
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

pub fn convert_to_rgba(src: &[u8], src_fmt: Format, dst: &mut Vec<u8>) -> Result<(), &'static str> {
    let mut rgba = ImageBuffer::<Rgba<u8>>::new(src_fmt.width, src_fmt.height);
    match _convert(src, src_fmt, &mut rgba) {
        Ok(()) => {
            *dst = rgba.into_vec();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn convert_to_bgra(src: &[u8], src_fmt: Format, dst: &mut Vec<u8>) -> Result<(), &'static str> {
    let mut bgra = ImageBuffer::<Bgra<u8>>::new(src_fmt.width, src_fmt.height);
    match _convert(src, src_fmt, &mut bgra) {
        Ok(()) => {
            *dst = bgra.into_vec();
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn convert(
    src: &[u8],
    src_fmt: Format,
    dst: &mut Vec<u8>,
    dst_fmt: Format,
) -> Result<(), &'static str> {
    match dst_fmt.pixfmt {
        PixelFormat::Bgra(32) => convert_to_bgra(src, src_fmt, dst),
        PixelFormat::Rgba(32) => convert_to_rgba(src, src_fmt, dst),
        _ => Err("cannot handle target format"),
    }
}
