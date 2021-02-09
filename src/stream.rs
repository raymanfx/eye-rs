use std::io;

use crate::image::CowImage;
use crate::traits::Stream;

pub struct Map<S, F> {
    stream: S,
    f: F,
}

impl<S, F> Map<S, F> {
    pub(super) fn new(stream: S, f: F) -> Self {
        Map { stream, f }
    }
}

impl<'a, S, F, B> Stream<'a> for Map<S, F>
where
    S: Stream<'a>,
    F: Fn(S::Item) -> B,
{
    type Item = B;

    fn next(&'a mut self) -> Option<Self::Item> {
        match self.stream.next() {
            Some(item) => Some((self.f)(item)),
            None => None,
        }
    }
}

/// A stream producing images
///
/// The output type is COW, meaning it will accept existing memory or allocate its own.
pub struct ImageStream<'a> {
    inner: Box<dyn 'a + for<'b> Stream<'b, Item = io::Result<CowImage<'b>>> + Send>,
}

impl<'a> ImageStream<'a> {
    /// Creates a new stream
    pub fn new(
        inner: Box<dyn 'a + for<'b> Stream<'b, Item = io::Result<CowImage<'b>>> + Send>,
    ) -> Self {
        ImageStream { inner }
    }

    /// Maps the stream output items
    pub fn map<F>(self, f: F) -> Map<Self, F>
    where
        F: Fn(io::Result<CowImage>) -> io::Result<CowImage>,
    {
        Map::new(self, f)
    }
}

impl<'a, 'b> Stream<'b> for ImageStream<'a> {
    type Item = io::Result<CowImage<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
