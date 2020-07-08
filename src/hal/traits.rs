use std::io;

use ffimage::packed::DynamicImageView;

use crate::control;
use crate::device::{ControlInfo, FormatInfo};
use crate::format::Format;

/// Platform device abstraction
pub trait Device {
    /// Returns the supported formats
    fn query_formats(&self) -> io::Result<Vec<FormatInfo>>;

    /// Returns the supported controls
    fn query_controls(&self) -> io::Result<Vec<ControlInfo>>;

    /// Returns the current control value for an ID
    fn get_control(&mut self, id: u32) -> io::Result<control::Value>;

    /// Sets the control value, returns error for incompatible value types
    fn set_control(&mut self, id: u32, val: &control::Value) -> io::Result<()>;

    /// Returns the current format in use by the device
    fn get_format(&mut self) -> io::Result<Format>;

    /// Attempts to match the requested format to a device format on a best-effort basis and
    /// returns the actual format in use
    fn set_format(&mut self, fmt: &Format) -> io::Result<Format>;

    /// Returns a zero-copy stream for direct frame access
    fn stream<'a>(&'a mut self) -> io::Result<Box<dyn Stream<Item = DynamicImageView> + 'a>>;
}

/// Stream abstraction
///
/// A stream is a construct which offers one item at a time. Once the next item is available, the
/// previous one is discarded and thus not accessible any longer.
pub trait Stream {
    /// Type of the stream elements
    type Item;

    /// Advances the stream and returns the next item
    fn next(&mut self) -> io::Result<Self::Item>;
}
