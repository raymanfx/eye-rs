use ffimage::color::Rgb;
use ffimage::packed::{ImageBuffer, ImageView};
use ffimage::traits::Convert;
use ffimage_yuv::{yuv::Yuv, yuyv::Yuyv};

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
        vec![PixelFormat::Custom(String::from("YUYV"))]
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
                if *ident != String::from("YUYV") {
                    return Err(Error::from(ErrorKind::UnsupportedFormat));
                }

                let fmt = ImageFormat {
                    width: self.inparams.width,
                    height: self.inparams.height,
                    pixfmt: self.inparams.pixfmt.clone(),
                    stride: None,
                };
                yuv422_to_rgb(inbuf, &fmt, outbuf)
            }
            _ => Err(Error::from(ErrorKind::UnsupportedFormat)),
        }
    }
}

pub fn yuv444_to_rgb(src: &[u8], src_fmt: &ImageFormat, dst: &mut Vec<u8>) -> Result<()> {
    let yuv444 = match ImageView::<Yuv<u8>>::from_buf(src, src_fmt.width, src_fmt.height) {
        Some(view) => view,
        None => return Err(Error::from(ErrorKind::InvalidBuffer)),
    };
    let mut rgb = ImageBuffer::<Rgb<u8>>::new(src_fmt.width, src_fmt.height, 0u8);
    yuv444.convert(&mut rgb);
    *dst = rgb.into_buf();

    Ok(())
}

pub fn yuv422_to_rgb(src: &[u8], src_fmt: &ImageFormat, dst: &mut Vec<u8>) -> Result<()> {
    // First stage conversion: YUV 4:2:0 --> YUV 4:4:4
    let yuv422 = match ImageView::<Yuyv<u8>>::from_buf(src, src_fmt.width, src_fmt.height) {
        Some(view) => view,
        None => return Err(Error::from(ErrorKind::InvalidBuffer)),
    };
    let mut yuv444 = ImageBuffer::<Yuv<u8>>::new(src_fmt.width, src_fmt.height, 0u8);
    yuv422.convert(&mut yuv444);

    // Second stage conversion: YUV 4:4:4 --> RGB
    yuv444_to_rgb(yuv444.as_ref(), src_fmt, dst)
}
