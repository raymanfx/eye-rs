use std::io;

use ffimage::packed::dynamic::{ImageBuffer, ImageView, MemoryView, StorageType};

use jpeg_decoder::{Decoder, PixelFormat as JpegFormat};

use crate::format::PixelFormat;
use crate::hal::common::convert::rgb;

pub fn convert_to_rgb(src: &ImageView, dst: &mut ImageBuffer) -> io::Result<()> {
    match src.raw() {
        MemoryView::U8(data) => {
            let mut decoder = Decoder::new(*data);
            let data = match decoder.decode() {
                Ok(data) => data,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            };

            let info = match decoder.info() {
                Some(info) => info,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "failed to read JPEG metadata",
                    ))
                }
            };

            match info.pixel_format {
                JpegFormat::RGB24 => {
                    *dst = ImageBuffer::from_raw(src.width(), src.height(), data).unwrap();
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

pub fn convert_to_rgba(src: &ImageView, dst: &mut ImageBuffer) -> io::Result<()> {
    let mut intermediate = ImageBuffer::empty(StorageType::U8);
    convert_to_rgb(src, &mut intermediate)?;
    let view = ImageView::new(
        intermediate.raw().as_slice::<u8>().unwrap(),
        src.width(),
        src.height(),
    )
    .unwrap();
    rgb::convert_to_rgba(&view, dst)
}

pub fn convert_to_bgra(src: &ImageView, dst: &mut ImageBuffer) -> io::Result<()> {
    let mut intermediate = ImageBuffer::empty(StorageType::U8);
    convert_to_rgb(src, &mut intermediate)?;
    let view = ImageView::new(
        intermediate.raw().as_slice::<u8>().unwrap(),
        src.width(),
        src.height(),
    )
    .unwrap();
    rgb::convert_to_bgra(&view, dst)
}

pub fn convert(src: &ImageView, dst: &mut ImageBuffer, dst_fmt: PixelFormat) -> io::Result<()> {
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
