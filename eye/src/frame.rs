use std::borrow::Cow;

use crate::colorconvert::Converter;
use crate::format::{ImageFormat, PixelFormat};

#[derive(Clone)]
pub struct Frame<'a> {
    pub(crate) buffer: Cow<'a, [u8]>,
    pub(crate) format: ImageFormat,
}

impl<'a> Frame<'a> {
    /// Returns the raw pixel bytes
    pub fn as_bytes(&self) -> &[u8] {
        match &self.buffer {
            Cow::Borrowed(slice) => slice,
            Cow::Owned(buf) => &buf,
        }
    }

    /// Returns the raw pixel bytes
    pub fn into_bytes(self) -> impl Iterator<Item = u8> {
        match self.buffer {
            Cow::Borrowed(slice) => slice.to_vec().into_iter(),
            Cow::Owned(buf) => buf.into_iter(),
        }
    }

    /// Returns the width in pixels
    pub fn width(&self) -> u32 {
        self.format.width
    }

    /// Returns the height in pixels
    pub fn height(&self) -> u32 {
        self.format.height
    }

    /// Returns the amount of bytes per pixel row
    pub fn stride(&self) -> Option<usize> {
        self.format.stride
    }

    /// Returns the format of the image pixels
    pub fn pixfmt(&self) -> &PixelFormat {
        &self.format.pixfmt
    }

    /// Returns an instance that is guaranteed to own its data
    ///
    /// If the instance currently borrows the data, it is cloned and transferred. Otherwise, no
    /// allocation is needed and the owned data is reused.
    pub fn own<'b>(self) -> Frame<'b> {
        Frame {
            buffer: Cow::Owned(self.buffer.into_owned()),
            format: self.format,
        }
    }

    /// Converts the image into a different format
    ///
    /// # Arguments
    ///
    /// * `pixfmt` - Target pixelFormat
    pub fn convert(self, pixfmt: &PixelFormat) -> Result<Frame<'a>, &'static str> {
        if pixfmt == self.pixfmt() {
            Ok(self)
        } else {
            let mut buf = Vec::new();
            let src_fmt = ImageFormat::new(self.width(), self.height(), self.pixfmt().clone());
            let dst_fmt = ImageFormat::new(self.width(), self.height(), pixfmt.clone());
            Converter::convert(self.as_bytes(), &src_fmt, &mut buf, &dst_fmt.pixfmt)?;

            Ok(Frame {
                buffer: Cow::Owned(buf),
                format: dst_fmt,
            })
        }
    }
}
