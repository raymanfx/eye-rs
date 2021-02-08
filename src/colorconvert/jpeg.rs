use jpeg_decoder::{Decoder, PixelFormat as JpegFormat};

use crate::colorconvert::rgb;
use crate::format::{pix, ImageFormat, PixelFormat};

pub fn convert_to_rgb(
    src: &[u8],
    _src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let mut decoder = Decoder::new(src);
    let data = match decoder.decode() {
        Ok(data) => data,
        Err(_) => return Err("failed to decode JPEG"),
    };

    let info = match decoder.info() {
        Some(info) => info,
        None => return Err("failed to read JPEG metadata"),
    };

    match info.pixel_format {
        JpegFormat::RGB24 => {
            *dst = data;
            Ok(())
        }
        _ => Err("cannot handle JPEG format"),
    }
}

pub fn convert_to_rgba(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let mut rgb = Vec::new();
    convert_to_rgb(src, src_fmt, &mut rgb)?;
    rgb::convert_to_rgba(&rgb, src_fmt, dst)
}

pub fn convert_to_bgra(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
) -> Result<(), &'static str> {
    let mut rgb = Vec::new();
    convert_to_rgb(src, src_fmt, &mut rgb)?;
    rgb::convert_to_bgra(&rgb, src_fmt, dst)
}

pub fn convert(
    src: &[u8],
    src_fmt: &ImageFormat,
    dst: &mut Vec<u8>,
    dst_fmt: &PixelFormat,
) -> Result<(), &'static str> {
    match dst_fmt {
        PixelFormat::Uncompressed(pix::Uncompressed::Bgra(32)) => {
            convert_to_bgra(src, src_fmt, dst)
        }
        PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)) => convert_to_rgb(src, src_fmt, dst),
        PixelFormat::Uncompressed(pix::Uncompressed::Rgba(32)) => {
            convert_to_rgba(src, src_fmt, dst)
        }
        _ => Err("cannot handle target format"),
    }
}
