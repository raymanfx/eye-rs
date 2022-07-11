use eye_hal::buffer::Buffer;
use eye_hal::format::PixelFormat;

mod error;
mod rgb;
mod yuv;

#[cfg(feature = "jpeg")]
mod jpeg;

pub use error::{Error, ErrorKind, Result};

/// Returns all available blueprints.
pub fn blueprints() -> Vec<Box<dyn Blueprint>> {
    vec![
        Box::new(rgb::blueprint()),
        #[cfg(feature = "jpeg")]
        Box::new(jpeg::blueprint()),
        Box::new(yuv::blueprint()),
    ]
}

/// Codec blueprint.
///
/// Allows you to identify the codecs' capabilities and instantiate it.
pub trait Blueprint {
    /// Instantiate the codec with a given set of parameters.
    fn instantiate(
        &self,
        inparams: Parameters,
        outparams: Parameters,
    ) -> Result<Box<dyn Codec + Send>>;

    /// Returns all available input formats.
    fn src_fmts(&self) -> Vec<PixelFormat>;

    /// Returns all available output formats.
    fn dst_fmts(&self) -> Vec<PixelFormat>;
}

pub trait Codec {
    /// Convert the input buffer using the output buffer as destination.
    fn decode(&self, inbuf: &Buffer, outbuf: &mut Buffer) -> Result<()>;
}

/// Codec parameters used for instantiation.
#[derive(Debug, Clone)]
pub struct Parameters {
    pub pixfmt: PixelFormat,
    pub width: u32,
    pub height: u32,
}
