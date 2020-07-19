use std::io;

use ffimage::packed::image::dynamic::{MemoryView, StorageType};
use ffimage::packed::{DynamicImageBuffer, DynamicImageView};

use jpeg_decoder::{Decoder, PixelFormat as JpegFormat};

use crate::format::PixelFormat;
use crate::hal::common::convert::rgb;

pub fn convert_to_rgb(src: &DynamicImageView, dst: &mut DynamicImageBuffer) -> io::Result<()> {
    match src.raw() {
        MemoryView::U8(data) => {
            let mut decoder = Decoder::new(*data);
            let data = decoder.decode();
            if data.is_err() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "failed to decode JPEG",
                ));
            }
            let data = data.unwrap();

            let info = decoder.info();
            if info.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "failed to read JPEG metadata",
                ));
            };
            let info = info.unwrap();

            match info.pixel_format {
                JpegFormat::RGB24 => {
                    *dst = DynamicImageBuffer::from_raw(src.width(), src.height(), data).unwrap();
                    Ok(())
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "cannot handle JPEG format",
                )),
            }
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot decode memory type",
        )),
    }
}

pub fn convert_to_rgba(src: &DynamicImageView, dst: &mut DynamicImageBuffer) -> io::Result<()> {
    let mut intermediate = DynamicImageBuffer::empty(StorageType::U8);
    convert_to_rgb(src, &mut intermediate)?;
    let view = DynamicImageView::new(
        intermediate.raw().as_slice::<u8>().unwrap(),
        src.width(),
        src.height(),
    )
    .unwrap();
    rgb::convert_to_rgba(&view, dst)
}

pub fn convert_to_bgra(src: &DynamicImageView, dst: &mut DynamicImageBuffer) -> io::Result<()> {
    let mut intermediate = DynamicImageBuffer::empty(StorageType::U8);
    convert_to_rgb(src, &mut intermediate)?;
    let view = DynamicImageView::new(
        intermediate.raw().as_slice::<u8>().unwrap(),
        src.width(),
        src.height(),
    )
    .unwrap();
    rgb::convert_to_bgra(&view, dst)
}

pub fn convert(
    src: &DynamicImageView,
    dst: &mut DynamicImageBuffer,
    dst_fmt: PixelFormat,
) -> io::Result<()> {
    match dst_fmt {
        PixelFormat::Bgra(32) => return convert_to_bgra(src, dst),
        PixelFormat::Rgb(24) => return convert_to_rgb(src, dst),
        PixelFormat::Rgba(32) => return convert_to_rgba(src, dst),
        _ => {}
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "cannot handle target format",
    ))
}
