use std::{io, time};

use crate::format::PixelFormat;
use crate::frame::Frame;
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

/// A stream producing frames
pub struct FrameStream<'a> {
    inner: Box<dyn 'a + for<'b> Stream<'b, Item = io::Result<Frame<'b>>> + Send>,
}

impl<'a> FrameStream<'a> {
    /// Creates a new stream
    pub fn new(
        inner: Box<dyn 'a + for<'b> Stream<'b, Item = io::Result<Frame<'b>>> + Send>,
    ) -> Self {
        FrameStream { inner }
    }
}

impl<'a, 'b> Stream<'b> for FrameStream<'a> {
    type Item = io::Result<Frame<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[derive(Clone, Debug)]
/// Image stream description
pub struct Descriptor {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// PixelFormat
    pub pixfmt: PixelFormat,
    /// Frame timing as duration
    pub interval: time::Duration,
}
