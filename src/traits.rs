use std::io;

use crate::control::{Control, Value as ControlValue};
use crate::format::ImageFormat;
use crate::stream::ImageStream;

/// Platform context abstraction
pub trait Context {
    /// Returns all devices currently available
    fn enumerate_devices() -> Vec<String>;
}

/// Platform device abstraction
pub trait Device<'a> {
    /// Returns the supported formats
    fn query_formats(&self) -> io::Result<Vec<ImageFormat>>;

    /// Returns the supported controls
    fn query_controls(&self) -> io::Result<Vec<Control>>;

    /// Returns the current control value for an ID
    fn control(&self, id: u32) -> io::Result<ControlValue>;

    /// Sets the control value, returns error for incompatible value types
    fn set_control(&mut self, id: u32, val: &ControlValue) -> io::Result<()>;

    /// Returns the current format in use by the device
    fn format(&self) -> io::Result<ImageFormat>;

    /// Sets the active format
    ///
    /// The actual behavior is HAL specific: some may try to match the requested format on a
    /// best-effort basis while others may return an error if the exact format is not available.
    fn set_format(&mut self, fmt: &ImageFormat) -> io::Result<()>;

    /// Returns a zero-copy stream for direct frame access
    fn stream(&self) -> io::Result<ImageStream<'a>>;
}

/// Stream abstraction
///
/// A stream is a construct which offers one item at a time. Once the next item is available, the
/// previous one is discarded and thus not accessible any longer.
pub trait Stream<'a> {
    /// Type of the stream elements
    type Item;

    /// Advances the stream and returns the next item
    fn next(&'a mut self) -> Option<Self::Item>;
}
