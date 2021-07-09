use jpeg_decoder::{Decoder, PixelFormat as JpegFormat};

use eye_hal::buffer::Buffer;
use eye_hal::format::PixelFormat;

use super::{Blueprint, Codec, Error, ErrorKind, Parameters, Result};

pub fn blueprint() -> impl Blueprint {
    Builder::default()
}

#[derive(Debug, Clone)]
pub struct Builder {}

impl Default for Builder {
    fn default() -> Self {
        Builder {}
    }
}

impl Blueprint for Builder {
    fn instantiate(
        &self,
        inparams: Parameters,
        outparams: Parameters,
    ) -> Result<Box<dyn Codec + Send>> {
        if self
            .src_fmts()
            .iter()
            .find(|pixfmt| **pixfmt == inparams.pixfmt)
            .is_none()
            || self
                .dst_fmts()
                .iter()
                .find(|pixfmt| **pixfmt == outparams.pixfmt)
                .is_none()
        {
            return Err(Error::from(ErrorKind::UnsupportedFormat));
        }

        if inparams.width != outparams.width || inparams.height != outparams.height {
            return Err(Error::from(ErrorKind::InvalidParam));
        }

        Ok(Box::new(Instance {
            inparams,
            outparams,
        }))
    }

    fn src_fmts(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::Jpeg]
    }

    fn dst_fmts(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::Rgb(24)]
    }
}

pub struct Instance {
    inparams: Parameters,
    outparams: Parameters,
}

impl Codec for Instance {
    fn decode(&self, inbuf: &Buffer, outbuf: &mut Buffer) -> Result<()> {
        match (&self.inparams.pixfmt, &self.outparams.pixfmt) {
            (PixelFormat::Jpeg, PixelFormat::Rgb(24)) => {
                let mut buf = Vec::new();
                match convert_to_rgb(inbuf.as_bytes(), &mut buf) {
                    Ok(()) => {
                        *outbuf = Buffer::from(buf);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            _ => Err(Error::from(ErrorKind::UnsupportedFormat)),
        }
    }
}

pub fn convert_to_rgb(src: &[u8], dst: &mut Vec<u8>) -> Result<()> {
    let mut decoder = Decoder::new(src);
    let data = match decoder.decode() {
        Ok(data) => data,
        Err(_) => return Err(Error::new(ErrorKind::Other, "failed to decode JPEG")),
    };

    let info = match decoder.info() {
        Some(info) => info,
        None => return Err(Error::new(ErrorKind::Other, "failed to read JPEG metadata")),
    };

    match info.pixel_format {
        JpegFormat::RGB24 => {
            *dst = data;
            Ok(())
        }
        _ => Err(Error::new(ErrorKind::Other, "cannot handle JPEG format")),
    }
}
