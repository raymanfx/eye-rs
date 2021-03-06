use std::collections::HashMap;
use std::hash::Hash;
use std::{cmp, io, time};

use itertools::Itertools;

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

    /// Returns the streams grouped by a field
    pub fn group_by<K, F>(self, key: F) -> HashMap<K, Self>
    where
        F: FnMut(&Descriptor) -> K,
        K: Eq + Hash + Clone,
    {
        let mut groups: HashMap<K, Self> = HashMap::new();
        self.streams
            .into_iter()
            .group_by(key)
            .into_iter()
            .for_each(|(key, group)| {
                for member in group {
                    let group = groups.entry(key.clone()).or_insert(Descriptors {
                        streams: Vec::new(),
                    });
                    group.streams.push(member);
                }
            });

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
