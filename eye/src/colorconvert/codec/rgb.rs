use ffimage::{
    color::{Bgr, Rgb},
    iter::{BytesExt, ColorConvertExt, PixelsExt},
};

use eye_hal::format::{ImageFormat, PixelFormat};

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
        vec![PixelFormat::Rgb(24)]
    }

    fn dst_fmts(&self) -> Vec<PixelFormat> {
        vec![PixelFormat::Bgr(24)]
    }
}

pub struct Instance {
    inparams: Parameters,
    outparams: Parameters,
}

impl Codec for Instance {
    fn decode(&self, inbuf: &[u8], outbuf: &mut Vec<u8>) -> Result<()> {
        match (&self.inparams.pixfmt, &self.outparams.pixfmt) {
            (PixelFormat::Rgb(24), PixelFormat::Bgr(24)) => {
                let fmt = ImageFormat {
                    width: self.inparams.width,
                    height: self.inparams.height,
                    pixfmt: self.inparams.pixfmt.clone(),
                    stride: None,
                };
                convert_to_bgr(inbuf, &fmt, outbuf)
            }
            _ => Err(Error::from(ErrorKind::UnsupportedFormat)),
        }
    }
}

pub fn convert_to_bgr(src: &[u8], src_fmt: &ImageFormat, dst: &mut Vec<u8>) -> Result<()> {
    let src_len = (src_fmt.width * src_fmt.height * 3) as usize;
    let dst_len = (src_fmt.width * src_fmt.height * 3) as usize;
    if src_len != src.len() {
        return Err(Error::from(ErrorKind::InvalidBuffer));
    }

    dst.resize(dst_len, 0);
    src.iter()
        .copied()
        .pixels::<Rgb<u8>>()
        .colorconvert::<Bgr<u8>>()
        .bytes()
        .write(dst);

    Ok(())
}
