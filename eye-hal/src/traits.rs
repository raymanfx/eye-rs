use crate::control;
use crate::device;
use crate::error::Result;
use crate::stream;

/// Platform context abstraction
pub trait Context<'a> {
    /// Device type supported by this platform
    type Device: Device<'a>;

    /// Returns all devices currently available
    fn devices(&self) -> Result<Vec<device::Description>>;

    /// Opens a device handle
    fn open_device(&self, uri: &str) -> Result<Self::Device>;
}

/// Platform device abstraction
pub trait Device<'a> {
    /// Capture stream type created by this device
    type Stream: Stream<'a>;

    /// Returns the supported streams
    fn streams(&self) -> Result<Vec<stream::Descriptor>>;

    /// Returns a stream which produces images
    fn start_stream(&self, desc: &stream::Descriptor) -> Result<Self::Stream>;

    /// Returns the supported controls
    fn controls(&self) -> Result<Vec<control::Descriptor>>;

    /// Returns the current control value for an ID
    fn control(&self, id: u32) -> Result<control::State>;

    /// Sets the control value, returns error for incompatible value types
    fn set_control(&mut self, id: u32, val: &control::State) -> Result<()>;
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
