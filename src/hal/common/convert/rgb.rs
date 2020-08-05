use std::io;

use ffimage::color::{Bgra, Rgb, Rgba};
use ffimage::core::{Pixel, TryConvert};
use ffimage::packed::dynamic::{ImageBuffer, ImageView};
use ffimage::packed::generic::{ImageBuffer as GenericImageBuffer, ImageView as GenericImageView};

use crate::format::PixelFormat;

fn _convert<DP: Pixel + From<Rgb<u8>>>(
    src: &ImageView,
    dst: &mut GenericImageBuffer<DP>,
) -> io::Result<()> {
    let data = src.raw().as_slice();
    if data.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to get raw [u8] data",
        ));
    }
    let data = data.unwrap();

    let rgb = GenericImageView::<Rgb<u8>>::new(data, src.width(), src.height());
    if rgb.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "failed to create RGB view",
        ));
    }
    let rgb = rgb.unwrap();

    let res = rgb.try_convert(dst);
    if res.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to convert RGB",
        ));
    }

    Ok(())
}

pub fn convert_to_rgba(src: &ImageView, dst: &mut ImageBuffer) -> io::Result<()> {
    let mut rgba = GenericImageBuffer::<Rgba<u8>>::new(src.width(), src.height());
    let res = _convert(src, &mut rgba);
    if res.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to convert RGB to RGBA",
        ));
    }

    *dst = ImageBuffer::from_raw(src.width(), src.height(), rgba.into()).unwrap();
    Ok(())
}

pub fn convert_to_bgra(src: &ImageView, dst: &mut ImageBuffer) -> io::Result<()> {
    let mut bgra = GenericImageBuffer::<Bgra<u8>>::new(src.width(), src.height());
    let res = _convert(src, &mut bgra);
    if res.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to convert RGB to BGRA",
        ));
    }

    *dst = ImageBuffer::from_raw(src.width(), src.height(), bgra.into()).unwrap();
    Ok(())
}

pub fn convert(src: &ImageView, dst: &mut ImageBuffer, dst_fmt: PixelFormat) -> io::Result<()> {
    match dst_fmt {
        PixelFormat::Bgra(32) => return convert_to_bgra(src, dst),
        PixelFormat::Rgba(32) => return convert_to_rgba(src, dst),
        _ => {}
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "cannot handle target format",
    ))
}
