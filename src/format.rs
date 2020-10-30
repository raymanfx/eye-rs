use std::{
    cmp::{Eq, PartialEq},
    convert::TryFrom,
    fmt, str,
};

#[derive(Debug, Default, Copy, Clone, Eq)]
/// Four character code representing a pixelformat
pub struct FourCC {
    /// Image format representation as string
    pub repr: [u8; 4],
}

impl FourCC {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    /// Returns a pixelformat as four character code
    ///
    /// # Arguments
    ///
    /// * `repr` - Four characters as raw bytes
    ///
    /// # Example
    ///
    /// ```
    /// use eye::format::FourCC;
    /// let fourcc = FourCC::new(b"RGB8");
    /// ```
    pub fn new(repr: &[u8; 4]) -> FourCC {
        FourCC { repr: *repr }
    }

    /// Returns the string representation of a four character code
    ///
    /// # Example
    ///
    /// ```
    /// use eye::format::FourCC;
    /// let fourcc = FourCC::new(b"RGB8");
    /// let str = fourcc.str().unwrap();
    /// ```
    pub fn str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.repr)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = str::from_utf8(&self.repr);
        if let Ok(string) = string {
            write!(f, "{}", string)?;
        }
        Ok(())
    }
}

impl PartialEq for FourCC {
    fn eq(&self, other: &FourCC) -> bool {
        self.repr.iter().zip(other.repr.iter()).all(|(a, b)| a == b)
    }
}

#[derive(Debug, Copy, Clone)]
/// Pixel format type used to describe image pixels.
///
/// Arbitrary formats can be wrapped in the Custom variant.
/// The other variants have values describing the depth of a whole pixel in bits.
pub enum PixelFormat {
    /// Special type for application defined formats
    Custom(FourCC),

    /// Mono patterns such as grayscale, depth buffers, etc.
    Depth(u32),
    Gray(u32),

    /// RGB
    Bgr(u32),
    Bgra(u32),
    Rgb(u32),
    Rgba(u32),
}

impl PixelFormat {
    /// Returns a unique ID for each variant
    pub fn id(&self) -> u32 {
        match self {
            PixelFormat::Custom(_) => 0,
            PixelFormat::Depth(_) => 1,
            PixelFormat::Gray(_) => 2,
            PixelFormat::Bgr(_) => 3,
            PixelFormat::Bgra(_) => 4,
            PixelFormat::Rgb(_) => 5,
            PixelFormat::Rgba(_) => 6,
        }
    }

    /// Returns the number of bits of a whole pixel
    pub fn bits(&self) -> Option<u32> {
        match self {
            PixelFormat::Custom(fourcc) => match PixelFormat::try_from(*fourcc) {
                Ok(pixfmt) => pixfmt.bits(),
                Err(_) => None,
            },
            PixelFormat::Depth(bits) => Some(*bits),
            PixelFormat::Gray(bits) => Some(*bits),
            PixelFormat::Bgr(bits) => Some(*bits),
            PixelFormat::Bgra(bits) => Some(*bits),
            PixelFormat::Rgb(bits) => Some(*bits),
            PixelFormat::Rgba(bits) => Some(*bits),
        }
    }

    /// Returns the number of channels
    pub fn channels(&self) -> Option<u32> {
        match self {
            PixelFormat::Custom(fourcc) => match PixelFormat::try_from(*fourcc) {
                Ok(pixfmt) => pixfmt.channels(),
                Err(_) => None,
            },
            PixelFormat::Depth(_) | PixelFormat::Gray(_) => Some(1),
            PixelFormat::Bgr(_) | PixelFormat::Rgb(_) => Some(3),
            PixelFormat::Bgra(_) | PixelFormat::Rgba(_) => Some(4),
        }
    }
}

impl Default for PixelFormat {
    fn default() -> PixelFormat {
        PixelFormat::Custom(FourCC::new(b"    "))
    }
}

impl From<FourCC> for PixelFormat {
    fn from(value: FourCC) -> Self {
        // We use the Linux fourccs as defined here:
        // https://www.kernel.org/doc/html/v5.5/media/uapi/v4l/videodev.html.

        // Mono (single-component) formats
        if value == FourCC::new(b"GREY") {
            PixelFormat::Gray(8)
        } else if value == FourCC::new(b"Y16 ") {
            PixelFormat::Gray(16)
        } else if value == FourCC::new(b"Z16 ") {
            PixelFormat::Depth(16)
        }
        // RGB formats
        else if value == FourCC::new(b"BGR3") {
            PixelFormat::Bgr(24)
        } else if value == FourCC::new(b"AR24") {
            PixelFormat::Bgra(32)
        } else if value == FourCC::new(b"RGB3") {
            PixelFormat::Rgb(24)
        } else if value == FourCC::new(b"AB24") {
            PixelFormat::Rgba(32)
        } else {
            PixelFormat::Custom(value)
        }
    }
}

impl TryFrom<PixelFormat> for FourCC {
    type Error = ();

    fn try_from(value: PixelFormat) -> Result<Self, Self::Error> {
        // We use the Linux fourccs as defined here:
        // https://www.kernel.org/doc/html/v5.5/media/uapi/v4l/videodev.html.

        match value {
            // Mono (single-component) formats
            PixelFormat::Custom(fourcc) => Ok(fourcc),
            PixelFormat::Depth(bits) => match bits {
                16 => Ok(FourCC::new(b"Z16 ")),
                _ => Err(()),
            },
            PixelFormat::Gray(bits) => match bits {
                8 => Ok(FourCC::new(b"GREY")),
                16 => Ok(FourCC::new(b"Y16 ")),
                _ => Err(()),
            },
            // RGB formats
            PixelFormat::Bgr(bits) => match bits {
                24 => Ok(FourCC::new(b"BGR3")),
                _ => Err(()),
            },
            PixelFormat::Bgra(bits) => match bits {
                32 => Ok(FourCC::new(b"AR24")),
                _ => Err(()),
            },
            PixelFormat::Rgb(bits) => match bits {
                24 => Ok(FourCC::new(b"RGB3")),
                _ => Err(()),
            },
            PixelFormat::Rgba(bits) => match bits {
                32 => Ok(FourCC::new(b"AB24")),
                _ => Err(()),
            },
        }
    }
}

impl PartialEq for PixelFormat {
    fn eq(&self, other: &PixelFormat) -> bool {
        // do not match the Custom (0) variant here
        if self.id() != 0 && self.id() == other.id() {
            true
        } else {
            let fourcc_1 = FourCC::try_from(*self);
            let fourcc_2 = FourCC::try_from(*other);

            if let (Ok(fourcc_1), Ok(fourcc_2)) = (fourcc_1, fourcc_2) {
                fourcc_1 == fourcc_2
            } else {
                false
            }
        }
    }
}

impl fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PixelFormat::Custom(fourcc) => write!(f, "{}", fourcc),
            x => write!(f, "{:?}", x),
        }
    }
}

#[derive(Default, Copy, Clone)]
/// Image buffer format description
pub struct Format {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// PixelFormat
    pub pixfmt: PixelFormat,
    /// Length of a pixel row in bytes
    pub stride: Option<usize>,
}

impl Format {
    /// Returns an image format representation
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `pixfmt` - PixelFormat
    ///
    /// # Example
    ///
    /// ```
    /// use eye::format::{Format, PixelFormat};
    /// let format = Format::new(1280, 720, PixelFormat::Rgb(24));
    /// ```
    pub fn new(width: u32, height: u32, pixfmt: PixelFormat) -> Self {
        let mut fmt = Format {
            width,
            height,
            pixfmt,
            stride: None,
        };

        if let Some(bits) = pixfmt.bits() {
            fmt.stride = Some(width as usize * bits as usize);
        }
        fmt
    }

    /// Returns an image format representation with a custom stride
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `pixfmt` - PixelFormat
    /// * `stride` - Length of a pixel row in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use eye::format::{Format, PixelFormat};
    /// let format = Format::with_stride(1280, 720, PixelFormat::Rgb(24), 1280 * 3 + 4);
    /// ```
    pub fn with_stride(width: u32, height: u32, pixfmt: PixelFormat, stride: usize) -> Self {
        Format {
            width,
            height,
            pixfmt,
            stride: Some(stride),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "width      : {}", self.width)?;
        writeln!(f, "height     : {}", self.height)?;
        writeln!(f, "pixfmt     : {}", self.pixfmt)?;
        writeln!(f, "stride     : {}", self.stride.unwrap_or(0))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fourcc_new() {
        let fourcc = FourCC::new(b"RGB3");
        assert_eq!(&fourcc.repr, b"RGB3");
        assert_eq!(fourcc.repr.len(), 4);
    }

    #[test]
    fn fourcc_str() {
        let fourcc = FourCC::new(b"RGB3");
        assert_eq!(fourcc.str().unwrap_or(""), "RGB3");
    }

    #[test]
    fn pixelformat_id() {
        assert_eq!(PixelFormat::Custom(FourCC::new(b"RGB3")).id(), 0);
        assert_eq!(PixelFormat::Custom(FourCC::new(b"BGR3")).id(), 0);
        assert_eq!(PixelFormat::Depth(16).id(), 1);
        assert_eq!(PixelFormat::Gray(16).id(), 2);
        assert_eq!(PixelFormat::Bgr(24).id(), 3);
        assert_eq!(PixelFormat::Bgra(32).id(), 4);
        assert_eq!(PixelFormat::Rgb(24).id(), 5);
        assert_eq!(PixelFormat::Rgba(24).id(), 6);
    }

    #[test]
    fn pixelformat_bits() {
        assert_eq!(PixelFormat::Rgb(24).bits().unwrap_or(0), 24);
        assert_eq!(
            PixelFormat::Custom(FourCC::new(b"GREY"))
                .bits()
                .unwrap_or(0),
            8
        );
    }

    #[test]
    fn pixelformat_channels() {
        assert_eq!(PixelFormat::Depth(16).channels().unwrap_or(0), 1);
        assert_eq!(PixelFormat::Gray(16).channels().unwrap_or(0), 1);
        assert_eq!(PixelFormat::Bgr(24).channels().unwrap_or(0), 3);
        assert_eq!(PixelFormat::Bgra(32).channels().unwrap_or(0), 4);
        assert_eq!(PixelFormat::Rgb(24).channels().unwrap_or(0), 3);
        assert_eq!(PixelFormat::Rgba(24).channels().unwrap_or(0), 4);
        assert_eq!(
            PixelFormat::Custom(FourCC::new(b"BGR3"))
                .channels()
                .unwrap_or(0),
            3
        );
    }

    #[test]
    fn pixelformat_eq() {
        assert_eq!(PixelFormat::Bgr(24), PixelFormat::Bgr(24));
        assert_eq!(
            PixelFormat::Rgb(24),
            PixelFormat::Custom(FourCC::new(b"RGB3"))
        );
        assert_ne!(PixelFormat::Bgr(24), PixelFormat::Rgb(24));
        assert_ne!(
            PixelFormat::Custom(FourCC::new(b"BGR3")),
            PixelFormat::Custom(FourCC::new(b"RGB3"))
        );
    }
}
