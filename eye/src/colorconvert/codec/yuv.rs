use ffimage::{
    color::Rgb,
    iter::{BytesExt, ColorConvertExt, PixelsExt},
};
use ffimage_yuv::{yuv::Yuv, yuv422::Yuv422};

use eye_hal::format::{ImageFormat, PixelFormat};

use super::{Blueprint, Codec, Error, ErrorKind, Parameters, Result};

pub fn blueprint() -> impl Blueprint {
    Builder::default()
}

#[derive(Debug, Clone, Default)]
pub struct Builder {}

impl Blueprint for Builder {
    fn instantiate(
        &self,
        inparams: Parameters,
        outparams: Parameters,
    ) -> Result<Box<dyn Codec + Send>> {
        if !self
            .src_fmts()
            .iter()
            .any(|pixfmt| *pixfmt == inparams.pixfmt)
            || !self
                .dst_fmts()
                .iter()
                .any(|pixfmt| *pixfmt == outparams.pixfmt)
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
        vec![
            PixelFormat::Custom(String::from("YUYV")),
            PixelFormat::Custom(String::from("IYU2")),
        ]
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
            (PixelFormat::Custom(ident), PixelFormat::Rgb(24)) => {
                let fmt = ImageFormat {
                    width: self.inparams.width,
                    height: self.inparams.height,
                    pixfmt: self.inparams.pixfmt.clone(),
                    stride: None,
                };

                match ident.as_str() {
                    "YUYV" => yuv422_to_rgb(inbuf, &fmt, outbuf),
                    "IYU2" => yuv444_to_rgb(inbuf, &fmt, outbuf),
                    _ => return Err(Error::from(ErrorKind::UnsupportedFormat)),
                }
            }
            _ => Err(Error::from(ErrorKind::UnsupportedFormat)),
        }
    }
}

pub fn yuv444_to_rgb(src: &[u8], src_fmt: &ImageFormat, dst: &mut Vec<u8>) -> Result<()> {
    let src_len = (src_fmt.width * src_fmt.height * 3) as usize;
    let dst_len = (src_fmt.width * src_fmt.height * 3) as usize;
    if src_len != src.len() {
        return Err(Error::from(ErrorKind::InvalidBuffer));
    }

    dst.resize(dst_len, 0);
    src.iter()
        .copied()
        .pixels::<Yuv<u8>>()
        .colorconvert::<Rgb<u8>>()
        .bytes()
        .write(dst);

    Ok(())
}

pub fn yuv422_to_rgb(src: &[u8], src_fmt: &ImageFormat, dst: &mut Vec<u8>) -> Result<()> {
    let src_len = (src_fmt.width * src_fmt.height * 2) as usize;
    let dst_len = (src_fmt.width * src_fmt.height * 3) as usize;
    if src_len != src.len() {
        return Err(Error::from(ErrorKind::InvalidBuffer));
    }

    dst.resize(dst_len, 0);
    src.iter()
        .copied()
        .pixels::<Yuv422<u8, 0, 2, 1, 3>>()
        .colorconvert::<[Yuv<u8>; 2]>()
        .flatten()
        .colorconvert::<Rgb<u8>>()
        .bytes()
        .write(dst);

    Ok(())
}
