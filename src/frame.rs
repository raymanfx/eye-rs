use std::borrow::Cow;

use crate::colorconvert::Converter;
use crate::format::{ImageFormat, PixelFormat};

#[derive(Clone)]
pub struct Frame<'a> {
    data: Cow<'a, [u8]>,
    fmt: ImageFormat,
}

impl<'a> Frame<'a> {
    /// Converts an image view into a cow image
    pub(crate) fn from_slice(slice: &'a [u8], fmt: ImageFormat) -> Self {
        Frame {
            data: Cow::Borrowed(slice),
            fmt,
        }
    }

    /// Converts an image buffer into a cow image
    pub(crate) fn from_bytes<I>(bytes: I, fmt: ImageFormat) -> Self
    where
        I: Iterator<Item = u8>,
    {
        Frame {
            data: Cow::Owned(bytes.collect()),
            fmt,
        }
    }

    /// Returns the raw pixel bytes
    pub fn as_bytes(&self) -> &[u8] {
        match &self.data {
            Cow::Borrowed(slice) => slice,
            Cow::Owned(buf) => &buf,
        }
    }

    /// Returns the raw pixel bytes
    pub fn into_bytes(self) -> impl Iterator<Item = u8> {
        match self.data {
            Cow::Borrowed(slice) => slice.to_vec().into_iter(),
            Cow::Owned(buf) => buf.into_iter(),
        }
    }

    /// Returns the width in pixels
    pub fn width(&self) -> u32 {
        self.fmt.width
    }

    /// Returns the height in pixels
    pub fn height(&self) -> u32 {
        self.fmt.height
    }

    /// Returns the amount of bytes per pixel row
    pub fn stride(&self) -> Option<usize> {
        self.fmt.stride
    }

    /// Returns the format of the image pixels
    pub fn pixfmt(&self) -> &PixelFormat {
        &self.fmt.pixfmt
    }

    /// Returns an instance that is guaranteed to own its data
    ///
    /// If the instance currently borrows the data, it is cloned and transferred. Otherwise, no
    /// allocation is needed and the owned data is reused.
    pub fn own<'b>(self) -> Frame<'b> {
        Frame {
            data: Cow::Owned(self.data.into_owned()),
            fmt: self.fmt,
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
                data: Cow::Owned(buf),
                fmt: dst_fmt,
            })
        }
    }
}
