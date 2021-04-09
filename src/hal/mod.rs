//! Hardware Abstraction Layer (HAL)
//!
//! Multiple backends can be implemented for a given platform.

use std::io;

use crate::control;
use crate::frame::Frame;
use crate::stream::Descriptor as StreamDescriptor;
use crate::traits::{Context, Device, Stream};

#[cfg(target_os = "linux")]
pub(crate) mod v4l2;

#[cfg(feature = "hal-uvc")]
pub(crate) mod uvc;

/// Platform context
///
/// Leaky abstraction: if you require access to platform specific features, match the enum instance
/// to get the underlying HAL implementation.
///
/// A context is used to query platform properties, available devices and more.
pub enum PlatformContext<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + Context + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 context
    V4l2(v4l2::context::Context),
    #[cfg(feature = "hal-uvc")]
    /// Universal Video Class context
    Uvc(uvc::context::Context),
}

impl<'a> Context for PlatformContext<'a> {
    fn query_devices(&self) -> io::Result<Vec<String>> {
        match self {
            Self::Custom(ctx) => ctx.query_devices(),
            #[cfg(target_os = "linux")]
            Self::V4l2(ctx) => ctx.query_devices(),
            #[cfg(feature = "hal-uvc")]
            Self::Uvc(ctx) => ctx.query_devices(),
        }
    }
}

/// Platform device
///
/// Leaky abstraction: if you require access to platform specific features, match the enum instance
/// to get the underlying HAL implementation.
///
/// A device is used to read/write control properties, start video streams and more.
pub enum PlatformDevice<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + Device<'a> + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 device handle
    V4l2(v4l2::device::Handle),
    #[cfg(feature = "hal-uvc")]
    /// Universal Video Class device handle
    Uvc(uvc::device::Handle<'a>),
}

impl<'a> Device<'a> for PlatformDevice<'a> {
    fn query_streams(&self) -> io::Result<Vec<StreamDescriptor>> {
        match self {
            Self::Custom(dev) => dev.query_streams(),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.query_streams(),
            #[cfg(feature = "hal-uvc")]
            Self::Uvc(dev) => dev.query_streams(),
        }
    }

    fn query_controls(&self) -> io::Result<Vec<control::Descriptor>> {
        match self {
            Self::Custom(dev) => dev.query_controls(),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.query_controls(),
            #[cfg(feature = "hal-uvc")]
            Self::Uvc(dev) => dev.query_controls(),
        }
    }

    fn read_control(&self, id: u32) -> io::Result<control::State> {
        match self {
            Self::Custom(dev) => dev.read_control(id),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.read_control(id),
            #[cfg(feature = "hal-uvc")]
            Self::Uvc(dev) => dev.read_control(id),
        }
    }

    fn write_control(&mut self, id: u32, val: &control::State) -> io::Result<()> {
        match self {
            Self::Custom(dev) => dev.write_control(id, val),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.write_control(id, val),
            #[cfg(feature = "hal-uvc")]
            Self::Uvc(dev) => dev.write_control(id, val),
        }
    }

    fn start_stream(&self, desc: &StreamDescriptor) -> io::Result<PlatformStream<'a>> {
        match self {
            Self::Custom(dev) => dev.start_stream(desc),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.start_stream(desc),
            #[cfg(feature = "hal-uvc")]
            Self::Uvc(dev) => dev.start_stream(desc),
        }
    }
}

/// Platform stream
///
/// Leaky abstraction: if you require access to platform specific features, match the enum instance
/// to get the underlying HAL implementation.
///
/// A stream is used read frames from a camera device. Many HAL implementations feature advanced
/// I/O method such as memory mapped streaming, DMA and more. We attempt to automatically select
/// the best method available.
pub enum PlatformStream<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + for<'b> Stream<'b, Item = io::Result<Frame<'b>>> + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 stream handle
    V4l2(v4l2::stream::Handle<'a>),
    #[cfg(feature = "hal-uvc")]
    /// Universal Video Class stream handle
    Uvc(uvc::stream::Handle<'a>),
}

impl<'a, 'b> Stream<'b> for PlatformStream<'a> {
    type Item = io::Result<Frame<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        match self {
            Self::Custom(stream) => stream.next(),
            #[cfg(target_os = "linux")]
            Self::V4l2(stream) => stream.next(),
            #[cfg(feature = "hal-uvc")]
            Self::Uvc(stream) => stream.next(),
        }
    }
}
