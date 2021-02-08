use std::{cmp::PartialEq, fmt};

pub mod pix {
    #[derive(Debug, Clone, PartialEq)]
    /// Compressed pixel formats
    pub enum Compressed {
        /// JPEG compression
        Jpeg,
    }

    impl Compressed {
        /// Returns the name
        pub fn name(&self) -> &str {
            match self {
                Compressed::Jpeg => "Jpeg",
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    /// Uncompressed pixel formats
    pub enum Uncompressed {
        /// Z buffers
        Depth(u32),
        /// Grayscale
        Gray(u32),

        /// Blue, Green, Red
        Bgr(u32),
        /// Blue, Green, Red, Alpha
        Bgra(u32),
        /// Red, Green, Blue
        Rgb(u32),
        /// Red, Green, Blue, Alpha
        Rgba(u32),
    }

    impl Uncompressed {
        /// Returns the name
        pub fn name(&self) -> &str {
            match self {
                Uncompressed::Depth(_) => "Depth",
                Uncompressed::Gray(_) => "Gray",
                Uncompressed::Bgr(_) => "Bgr",
                Uncompressed::Bgra(_) => "Bgra",
                Uncompressed::Rgb(_) => "Rgb",
                Uncompressed::Rgba(_) => "Rgba",
            }
        }

        /// Returns the number of bits of a whole pixel
        pub fn bits(&self) -> u32 {
            match self {
                Uncompressed::Depth(bits) => *bits,
                Uncompressed::Gray(bits) => *bits,
                Uncompressed::Bgr(bits) => *bits,
                Uncompressed::Bgra(bits) => *bits,
                Uncompressed::Rgb(bits) => *bits,
                Uncompressed::Rgba(bits) => *bits,
            }
        }

        /// Returns the number of channels
        pub fn channels(&self) -> u32 {
            match self {
                Uncompressed::Depth(_) | Uncompressed::Gray(_) => 1,
                Uncompressed::Bgr(_) | Uncompressed::Rgb(_) => 3,
                Uncompressed::Bgra(_) | Uncompressed::Rgba(_) => 4,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Pixel format type used to describe image pixels.
///
/// Arbitrary formats can be wrapped in the Custom variant.
/// The other variants have values describing the depth of a whole pixel in bits.
pub enum PixelFormat {
    /// Special type for application defined formats
    Custom(String),

    /// Compressed formats
    Compressed(pix::Compressed),

    /// Uncompressed formats
    Uncompressed(pix::Uncompressed),
}

impl PixelFormat {
    /// Returns the name
    pub fn name(&self) -> &str {
        match self {
            PixelFormat::Custom(desc) => desc,
            PixelFormat::Compressed(variant) => variant.name(),
            PixelFormat::Uncompressed(variant) => variant.name(),
        }
    }
}

impl fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.name())
    }
}

#[derive(Clone)]
/// Image buffer format description
pub struct ImageFormat {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// PixelFormat
    pub pixfmt: PixelFormat,
    /// Length of a pixel row in bytes
    pub stride: Option<usize>,
}

impl ImageFormat {
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
    /// use eye::format::{pix, ImageFormat, PixelFormat};
    /// let format = ImageFormat::new(1280, 720, PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)));
    /// ```
    pub fn new(width: u32, height: u32, pixfmt: PixelFormat) -> Self {
        let stride = if let PixelFormat::Uncompressed(variant) = &pixfmt {
            Some((width * (variant.bits() / 8)) as usize)
        } else {
            None
        };

        ImageFormat {
            width,
            height,
            pixfmt,
            stride,
        }
    }

    /// Builder pattern constructor
    ///
    /// # Arguments
    ///
    /// * `stride` - Length of a pixel row in bytes
    pub fn stride(mut self, stride: usize) -> Self {
        self.stride = Some(stride);
        self
    }
}

impl fmt::Display for ImageFormat {
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
    fn pixelformat_eq() {
        assert_eq!(
            PixelFormat::Compressed(pix::Compressed::Jpeg),
            PixelFormat::Compressed(pix::Compressed::Jpeg)
        );
        assert_ne!(
            PixelFormat::Uncompressed(pix::Uncompressed::Bgr(24)),
            PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24))
        );
    }
}
