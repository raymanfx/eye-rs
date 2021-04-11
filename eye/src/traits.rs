use crate::control;
use crate::error::Result;
use crate::hal::PlatformStream;
use crate::stream::{Descriptor as StreamDescriptor, Map};

/// Platform context abstraction
pub trait Context {
    /// Returns all devices currently available
    fn query_devices(&self) -> Result<Vec<String>>;
}

/// Platform device abstraction
pub trait Device<'a> {
    /// Returns the supported streams
    fn query_streams(&self) -> Result<Vec<StreamDescriptor>>;

    /// Returns the supported controls
    fn query_controls(&self) -> Result<Vec<control::Descriptor>>;

    /// Returns the current control value for an ID
    fn read_control(&self, id: u32) -> Result<control::State>;

    /// Sets the control value, returns error for incompatible value types
    fn write_control(&mut self, id: u32, val: &control::State) -> Result<()>;

    /// Returns a stream which produces images
    fn start_stream(&self, desc: &StreamDescriptor) -> Result<PlatformStream<'a>>;
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

    /// Maps the stream output items
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        Map::new(self, f)
    }
}
