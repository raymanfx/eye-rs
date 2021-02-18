use std::{cmp, io, time};

use crate::format::{ImageFormat, PixelFormat};
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
    format: ImageFormat,
}

impl<'a> ImageStream<'a> {
    /// Creates a new stream
    pub fn new(
        inner: Box<dyn 'a + for<'b> Stream<'b, Item = io::Result<CowImage<'b>>> + Send>,
        format: ImageFormat,
    ) -> Self {
        ImageStream { inner, format }
    }

    /// Returns the format of the images produced by the stream
    pub fn format(&self) -> &ImageFormat {
        &self.format
    }
}

impl<'a, 'b> Stream<'b> for ImageStream<'a> {
    type Item = io::Result<CowImage<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[derive(Clone, Debug)]
/// Image stream description collection
pub struct Descriptors {
    /// Streams
    pub(crate) streams: Vec<Descriptor>,
}

impl Descriptors {
    /// Returns the streams sorted by the caller
    pub fn sort_by<F>(&mut self, f: F)
    where
        F: FnMut(&Descriptor, &Descriptor) -> cmp::Ordering,
    {
        self.streams.sort_by(f);
    }

    /// Returns the streams grouped by their resolutions
    pub fn group_by_resolution(self) -> impl IntoIterator<Item = ((u32, u32), Self)> {
        let mut groups: Vec<((u32, u32), Self)> = Vec::new();
        for stream in self.streams {
            let mut inserted = false;

            for group in &mut groups {
                if group.0 == (stream.width, stream.height) {
                    group.1.streams.push(stream.clone());
                    inserted = true;
                }
            }

            if !inserted {
                // create a new group
                groups.push((
                    (stream.width, stream.height),
                    Descriptors {
                        streams: Vec::new(),
                    },
                ))
            }
        }

        groups
    }

    /// Returns the streams grouped by their pixelformats
    pub fn group_by_pixfmt(self) -> impl IntoIterator<Item = (PixelFormat, Self)> {
        let mut groups: Vec<(PixelFormat, Self)> = Vec::new();
        for stream in self.streams {
            let mut inserted = false;

            for group in &mut groups {
                if group.0 == stream.pixfmt {
                    group.1.streams.push(stream.clone());
                    inserted = true;
                }
            }

            if !inserted {
                // create a new group
                groups.push((
                    stream.pixfmt,
                    Descriptors {
                        streams: Vec::new(),
                    },
                ))
            }
        }

        groups
    }
}

impl IntoIterator for Descriptors {
    type Item = Descriptor;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.streams.into_iter()
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
