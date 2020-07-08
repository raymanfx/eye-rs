use std::io;

use ffimage::packed::image::dynamic::MemoryView;
use ffimage::packed::{DynamicImageBuffer, DynamicImageView};

use jpeg_decoder::{Decoder, PixelFormat};

use crate::format::FourCC;

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
                PixelFormat::RGB24 => {
                    *dst = DynamicImageBuffer::with_raw(src.width(), src.height(), &data).unwrap();
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

pub fn convert(
    src: &DynamicImageView,
    dst: &mut DynamicImageBuffer,
    dst_fmt: FourCC,
) -> io::Result<()> {
    if dst_fmt == FourCC::new(b"RGB3") {
        return convert_to_rgb(src, dst);
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "cannot handle target format",
    ))
}
