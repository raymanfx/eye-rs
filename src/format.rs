use std::cmp::{Eq, PartialEq};
use std::{fmt, str};

#[derive(Default, Copy, Clone, Eq)]
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

#[derive(Default, Copy, Clone)]
/// Image buffer format description
pub struct Format {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Pixelformat
    pub fourcc: FourCC,
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
    /// * `fourcc` - Pixelformat
    ///
    /// # Example
    ///
    /// ```
    /// use eye::format::{Format, FourCC};
    /// let fourcc = FourCC::new(b"RGB8");
    /// let format = Format::new(1280, 720, fourcc);
    /// ```
    pub fn new(width: u32, height: u32, fourcc: FourCC) -> Self {
        Format {
            width,
            height,
            fourcc,
            stride: None,
        }
    }

    /// Returns an image format representation with a custom stride
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `fourcc` - Pixelformat
    /// * `stride` - Length of a pixel row in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use eye::format::{Format, FourCC};
    /// let fourcc = FourCC::new(b"RGB8");
    /// let format = Format::new(1280, 720, fourcc);
    /// ```
    pub fn with_stride(width: u32, height: u32, fourcc: FourCC, stride: usize) -> Self {
        Format {
            width,
            height,
            fourcc,
            stride: Some(stride),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "width  : {}", self.width)?;
        writeln!(f, "height : {}", self.height)?;
        writeln!(f, "fourcc : {}", self.fourcc)?;
        writeln!(f, "stride : {}", self.stride.unwrap_or(0))?;
        Ok(())
    }
}
