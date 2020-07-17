use std::io;

use ffimage::color::{Bgra, Rgb, Rgba};
use ffimage::core::{Pixel, TryConvert};
use ffimage::packed::{DynamicImageBuffer, DynamicImageView, GenericImageBuffer, GenericImageView};

use crate::format::FourCC;

fn _convert<DP: Pixel + From<Rgb<u8>>>(
    src: &DynamicImageView,
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

pub fn convert_to_rgba(src: &DynamicImageView, dst: &mut DynamicImageBuffer) -> io::Result<()> {
    let mut rgba = GenericImageBuffer::<Rgba<u8>>::new(src.width(), src.height());
    let res = _convert(src, &mut rgba);
    if res.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to convert RGB to RGBA",
        ));
    }

    *dst = DynamicImageBuffer::from_raw(src.width(), src.height(), rgba.into()).unwrap();
    Ok(())
}

pub fn convert_to_bgra(src: &DynamicImageView, dst: &mut DynamicImageBuffer) -> io::Result<()> {
    let mut bgra = GenericImageBuffer::<Bgra<u8>>::new(src.width(), src.height());
    let res = _convert(src, &mut bgra);
    if res.is_err() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to convert RGB to BGRA",
        ));
    }

    *dst = DynamicImageBuffer::from_raw(src.width(), src.height(), bgra.into()).unwrap();
    Ok(())
}

pub fn convert(
    src: &DynamicImageView,
    dst: &mut DynamicImageBuffer,
    dst_fmt: FourCC,
) -> io::Result<()> {
    if dst_fmt == FourCC::new(b"AB24") {
        return convert_to_rgba(src, dst);
    } else if dst_fmt == FourCC::new(b"AR24") {
        return convert_to_bgra(src, dst);
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "cannot handle target format",
    ))
}
