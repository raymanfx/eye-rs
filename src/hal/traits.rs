use std::{io, marker::PhantomData, ops::Deref};

use ffimage::packed::dynamic::ImageView;

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
    fn control(&self, id: u32) -> io::Result<Option<control::Value>>;

    /// Sets the control value, returns error for incompatible value types
    fn set_control(&mut self, id: u32, val: &control::Value) -> io::Result<()>;

    /// Returns the current format in use by the device
    fn format(&self) -> io::Result<Format>;

    /// Attempts to match the requested format to a device format on a best-effort basis and
    /// returns the actual format in use
    fn set_format(&mut self, fmt: &Format) -> io::Result<Format>;

    /// Returns a zero-copy stream for direct frame access
    fn stream<'a>(&self) -> io::Result<Box<dyn Stream<Item = ImageView<'a>> + 'a>>;
}

/// Stream item wrapper
///
/// The sole purpose of this wrapper struct is to attach a lifetime to values of type T.
/// This is especially useful for volatile types such as views which provide access to some kind of
/// underlying data.
pub struct StreamItem<'a, T> {
    /// The wrapped item
    item: T,
    // Used to augment the item with a lifetime to benefit from the borrow checker
    _lifetime: PhantomData<&'a mut ()>,
}

impl<'a, T> StreamItem<'a, T> {
    /// Returns a wrapped stream item by moving it into the wrapper
    ///
    /// An explicit lifetime is attached automatically by inserting PhantomData.
    ///
    /// # Arguments
    ///
    /// * `item` - Item to be wrapped
    ///
    /// # Example
    ///
    /// ```
    /// use eye::hal::traits::StreamItem;
    /// let item: u32 = 123;
    /// let wrapper = StreamItem::new(item);
    /// ```
    pub fn new(item: T) -> Self {
        StreamItem {
            item,
            _lifetime: PhantomData,
        }
    }
}

impl<'a, T> Deref for StreamItem<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

/// Stream abstraction
///
/// A stream is a construct which offers one item at a time. Once the next item is available, the
/// previous one is discarded and thus not accessible any longer.
pub trait Stream {
    /// Type of the stream elements
    type Item;

    /// Advances the stream and returns the next item
    fn next<'a>(&'a mut self) -> io::Result<StreamItem<'a, Self::Item>>;
}
