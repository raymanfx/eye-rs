use std::io;

use crate::control::{Control, Value as ControlValue};
use crate::format::Format;
use crate::image::CowImage;

/// Platform device abstraction
pub trait Device<'a>: Send {
    /// Returns the supported formats
    fn query_formats(&self) -> io::Result<Vec<Format>>;

    /// Returns the supported controls
    fn query_controls(&self) -> io::Result<Vec<Control>>;

    /// Returns the current control value for an ID
    fn control(&self, id: u32) -> io::Result<ControlValue>;

    /// Sets the control value, returns error for incompatible value types
    fn set_control(&mut self, id: u32, val: &ControlValue) -> io::Result<()>;

    /// Returns the current format in use by the device
    fn format(&self) -> io::Result<Format>;

    /// Attempts to match the requested format to a device format on a best-effort basis and
    /// returns the actual format in use
    fn set_format(&mut self, fmt: &Format) -> io::Result<Format>;

    /// Returns a zero-copy stream for direct frame access
    fn stream(&self) -> io::Result<Box<ImageStream<'a>>>;
}

/// Stream abstraction
///
/// A stream is a construct which offers one item at a time. Once the next item is available, the
/// previous one is discarded and thus not accessible any longer.
pub trait Stream<'a>: Send {
    /// Type of the stream elements
    type Item;

    /// Advances the stream and returns the next item
    fn next(&'a mut self) -> Option<Self::Item>;
}

/// A stream producing images
pub type ImageStream<'a> = dyn 'a + for<'b> Stream<'b, Item = io::Result<CowImage<'b>>> + Send;
