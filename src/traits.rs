use std::io;

use crate::format::Format;

/// Stream abstraction
///
/// A stream is a construct which offers one item at a time. Once the next item is available, the
/// previous one is discarded and thus not accessible any longer.
pub trait Stream {
    /// Type of the stream elements
    type Item;

    /// Format of stream items (image buffers)
    fn format(&self) -> Format;

    /// Advances the stream and returns the next item
    fn next(&mut self) -> io::Result<Self::Item>;
}
