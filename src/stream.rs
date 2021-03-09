use std::borrow::Cow;
use std::{io, time};

use bitflags::bitflags;

use crate::colorconvert::Converter;
use crate::format::{ImageFormat, PixelFormat};
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

/// A stream converting frames
pub struct ConvertStream<S> {
    pub inner: S,
    pub map: (PixelFormat, PixelFormat),
}

impl<'a, S> Stream<'a> for ConvertStream<S>
where
    S: Stream<'a, Item = io::Result<Frame<'a>>>,
{
    type Item = io::Result<Frame<'a>>;

    fn next(&'a mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        let frame = if let Ok(frame) = item {
            frame
        } else {
            return Some(item);
        };

        let mut buf = Vec::new();
        let src_fmt = ImageFormat::new(frame.width(), frame.height(), frame.pixfmt().clone());
        let dst_fmt = ImageFormat::new(frame.width(), frame.height(), self.map.1.clone());
        let res = if let Err(msg) =
            Converter::convert(frame.as_bytes(), &src_fmt, &mut buf, &dst_fmt.pixfmt)
        {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("failed to convert stream item: {}", msg),
            ))
        } else {
            Ok(Frame {
                buffer: Cow::Owned(buf),
                format: dst_fmt,
            })
        };

        Some(res)
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
    /// Miscellaneous flags
    pub flags: Flags,
}

impl Descriptor {
    /// Whether the stream is emulated
    pub fn is_emulated(&self) -> bool {
        self.flags & Flags::EMULATED == Flags::EMULATED
    }
}

bitflags! {
    /// Control state flags
    pub struct Flags: u32 {
        /// No flags are set
        const NONE                  = 0x000;
        /// Stream is emulated by converting items
        const EMULATED              = 0x001;
    }
}
