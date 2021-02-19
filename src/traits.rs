use std::io;

use crate::control::{Control, Value as ControlValue};
use crate::stream::{
    Descriptor as StreamDescriptor, Descriptors as StreamDescriptors, ImageStream, Map,
};

/// Platform context abstraction
pub trait Context {
    /// Returns all devices currently available
    fn enumerate_devices() -> Vec<String>;
}

/// Platform device abstraction
pub trait Device<'a> {
    /// Returns the supported streams
    fn query_streams(&self) -> io::Result<StreamDescriptors>;

    /// Returns the supported controls
    fn query_controls(&self) -> io::Result<Vec<Control>>;

    /// Returns the current control value for an ID
    fn control(&self, id: u32) -> io::Result<ControlValue>;

    /// Sets the control value, returns error for incompatible value types
    fn set_control(&mut self, id: u32, val: &ControlValue) -> io::Result<()>;

    /// Returns the current format in use by the device
    fn preferred_stream(
        &self,
        f: &dyn Fn(StreamDescriptor, StreamDescriptor) -> StreamDescriptor,
    ) -> io::Result<StreamDescriptor>;

    /// Returns a stream which produces images
    fn start_stream(&self, desc: &StreamDescriptor) -> io::Result<ImageStream<'a>>;
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
