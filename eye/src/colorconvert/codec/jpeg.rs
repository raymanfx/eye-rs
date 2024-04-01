use jpeg_decoder::{Decoder, PixelFormat as JpegFormat};

use eye_hal::format::PixelFormat;

use super::{Blueprint, Codec, Error, ErrorKind, Parameters, Result};

pub fn blueprint() -> impl Blueprint {
    Builder::default()
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct Builder {}


impl Blueprint for Builder {
    fn instantiate(
        &self,
        inparams: Parameters,
        outparams: Parameters,
    ) -> Result<Box<dyn Codec + Send>> {
        if !self
            .src_fmts()
            .iter().any(|pixfmt| *pixfmt == inparams.pixfmt)
            || !self
                .dst_fmts()
                .iter().any(|pixfmt| *pixfmt == outparams.pixfmt)
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
    fn decode(&self, inbuf: &[u8], outbuf: &mut Vec<u8>) -> Result<()> {
        match (&self.inparams.pixfmt, &self.outparams.pixfmt) {
            (PixelFormat::Jpeg, PixelFormat::Rgb(24)) => convert_to_rgb(inbuf, outbuf),
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
