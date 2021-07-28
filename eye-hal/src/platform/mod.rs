//! Hardware Abstraction Layer (HAL)
//!
//! Multiple backends can be implemented for a given platform.

use std::array;

use crate::buffer::Buffer;
use crate::control;
use crate::device;
use crate::error::Result;
use crate::stream::Descriptor as StreamDescriptor;
use crate::traits::{Context as ContextTrait, Device as DeviceTrait, Stream as StreamTrait};

#[cfg(target_os = "linux")]
pub(crate) mod v4l2;

#[cfg(feature = "plat-uvc")]
pub(crate) mod uvc;

/// Platform context
///
/// Leaky abstraction: if you require access to platform specific features, match the enum instance
/// to get the underlying HAL implementation.
///
/// A context is used to query platform properties, available devices and more.
pub enum Context<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + ContextTrait + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 context
    V4l2(v4l2::context::Context),
    #[cfg(feature = "plat-uvc")]
    /// Universal Video Class context
    Uvc(uvc::context::Context),
}

impl<'a> Context<'a> {
    pub fn all() -> impl Iterator<Item = Context<'a>> {
        array::IntoIter::new([
            #[cfg(target_os = "linux")]
            Context::V4l2(v4l2::context::Context {}),
            #[cfg(feature = "plat-uvc")]
            Context::Uvc(uvc::context::Context {}),
        ])
    }
}

impl<'a> Default for Context<'a> {
    #[allow(unreachable_code)]
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        return Context::V4l2(v4l2::context::Context {});
        #[cfg(feature = "plat-uvc")]
        return Context::Uvc(uvc::context::Context {});
    }
}

impl<'a> ContextTrait for Context<'a> {
    fn devices(&self) -> Result<Vec<device::Description>> {
        match self {
            Self::Custom(ctx) => ctx.devices(),
            #[cfg(target_os = "linux")]
            Self::V4l2(ctx) => ctx.devices(),
            #[cfg(feature = "plat-uvc")]
            Self::Uvc(ctx) => ctx.devices(),
        }
    }

    fn open_device<'b>(&self, uri: &str) -> Result<Device<'b>> {
        match self {
            Self::Custom(ctx) => ctx.open_device(uri),
            #[cfg(target_os = "linux")]
            Self::V4l2(ctx) => ctx.open_device(uri),
            #[cfg(feature = "plat-uvc")]
            Self::Uvc(ctx) => ctx.open_device(uri),
        }
    }
}

/// Platform device
///
/// Leaky abstraction: if you require access to platform specific features, match the enum instance
/// to get the underlying HAL implementation.
///
/// A device is used to read/write control properties, start video streams and more.
pub enum Device<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + DeviceTrait<'a> + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 device handle
    V4l2(v4l2::device::Handle),
    #[cfg(feature = "plat-uvc")]
    /// Universal Video Class device handle
    Uvc(uvc::device::Handle<'a>),
}

impl<'a> DeviceTrait<'a> for Device<'a> {
    fn streams(&self) -> Result<Vec<StreamDescriptor>> {
        match self {
            Self::Custom(dev) => dev.streams(),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.streams(),
            #[cfg(feature = "plat-uvc")]
            Self::Uvc(dev) => dev.streams(),
        }
    }

    fn controls(&self) -> Result<Vec<control::Descriptor>> {
        match self {
            Self::Custom(dev) => dev.controls(),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.controls(),
            #[cfg(feature = "plat-uvc")]
            Self::Uvc(dev) => dev.controls(),
        }
    }

    fn control(&self, id: u32) -> Result<control::State> {
        match self {
            Self::Custom(dev) => dev.control(id),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.control(id),
            #[cfg(feature = "plat-uvc")]
            Self::Uvc(dev) => dev.control(id),
        }
    }

    fn set_control(&mut self, id: u32, val: &control::State) -> Result<()> {
        match self {
            Self::Custom(dev) => dev.set_control(id, val),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.set_control(id, val),
            #[cfg(feature = "plat-uvc")]
            Self::Uvc(dev) => dev.set_control(id, val),
        }
    }

    fn start_stream(&self, desc: &StreamDescriptor) -> Result<Stream<'a>> {
        match self {
            Self::Custom(dev) => dev.start_stream(desc),
            #[cfg(target_os = "linux")]
            Self::V4l2(dev) => dev.start_stream(desc),
            #[cfg(feature = "plat-uvc")]
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
pub enum Stream<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + for<'b> StreamTrait<'b, Item = Result<Buffer<'b>>> + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 stream handle
    V4l2(v4l2::stream::Handle<'a>),
    #[cfg(feature = "plat-uvc")]
    /// Universal Video Class stream handle
    Uvc(uvc::stream::Handle<'a>),
}

impl<'a, 'b> StreamTrait<'b> for Stream<'a> {
    type Item = Result<Buffer<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        match self {
            Self::Custom(stream) => stream.next(),
            #[cfg(target_os = "linux")]
            Self::V4l2(stream) => stream.next(),
            #[cfg(feature = "plat-uvc")]
            Self::Uvc(stream) => stream.next(),
        }
    }
}
