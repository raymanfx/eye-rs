//! Hardware Abstraction Layer (HAL)
//!
//! Multiple backends can be implemented for a given platform.

use std::io;

use crate::control;
use crate::frame::Frame;
use crate::stream::Descriptor as StreamDescriptor;
use crate::traits::{Context as ContextTrait, Device as DeviceTrait, Stream as StreamTrait};

#[cfg(target_os = "linux")]
pub(crate) mod v4l2;

#[cfg(feature = "hal-uvc")]
pub(crate) mod uvc;

/// Platform context
pub enum Context<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + ContextTrait + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 context
    V4l2(v4l2::context::Context),
    #[cfg(feature = "hal-uvc")]
    /// Universal Video Class context
    Uvc(uvc::context::Context),
}

impl<'a> ContextTrait for Context<'a> {
    fn query_devices(&self) -> io::Result<Vec<String>> {
        match self {
            Context::Custom(ctx) => ctx.query_devices(),
            #[cfg(target_os = "linux")]
            Context::V4l2(ctx) => ctx.query_devices(),
            #[cfg(feature = "hal-uvc")]
            Context::Uvc(ctx) => ctx.query_devices(),
        }
    }
}

/// Platform device
pub enum Device<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + DeviceTrait<'a> + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 device handle
    V4l2(v4l2::device::Handle),
    #[cfg(feature = "hal-uvc")]
    /// Universal Video Class device handle
    Uvc(uvc::device::Handle<'a>),
}

impl<'a> DeviceTrait<'a> for Device<'a> {
    fn query_streams(&self) -> io::Result<Vec<StreamDescriptor>> {
        match self {
            Device::Custom(dev) => dev.query_streams(),
            #[cfg(target_os = "linux")]
            Device::V4l2(dev) => dev.query_streams(),
            #[cfg(feature = "hal-uvc")]
            Device::Uvc(dev) => dev.query_streams(),
        }
    }

    fn query_controls(&self) -> io::Result<Vec<control::Descriptor>> {
        match self {
            Device::Custom(dev) => dev.query_controls(),
            #[cfg(target_os = "linux")]
            Device::V4l2(dev) => dev.query_controls(),
            #[cfg(feature = "hal-uvc")]
            Device::Uvc(dev) => dev.query_controls(),
        }
    }

    fn read_control(&self, id: u32) -> io::Result<control::State> {
        match self {
            Device::Custom(dev) => dev.read_control(id),
            #[cfg(target_os = "linux")]
            Device::V4l2(dev) => dev.read_control(id),
            #[cfg(feature = "hal-uvc")]
            Device::Uvc(dev) => dev.read_control(id),
        }
    }

    fn write_control(&mut self, id: u32, val: &control::State) -> io::Result<()> {
        match self {
            Device::Custom(dev) => dev.write_control(id, val),
            #[cfg(target_os = "linux")]
            Device::V4l2(dev) => dev.write_control(id, val),
            #[cfg(feature = "hal-uvc")]
            Device::Uvc(dev) => dev.write_control(id, val),
        }
    }

    fn start_stream(&self, desc: &StreamDescriptor) -> io::Result<Stream<'a>> {
        match self {
            Device::Custom(dev) => dev.start_stream(desc),
            #[cfg(target_os = "linux")]
            Device::V4l2(dev) => dev.start_stream(desc),
            #[cfg(feature = "hal-uvc")]
            Device::Uvc(dev) => dev.start_stream(desc),
        }
    }
}

/// Platform stream
pub enum Stream<'a> {
    /// Can be used to wrap your own struct
    Custom(Box<dyn 'a + for<'b> StreamTrait<'b, Item = io::Result<Frame<'b>>> + Send>),
    #[cfg(target_os = "linux")]
    /// Video4Linux2 stream handle
    V4l2(v4l2::stream::Handle<'a>),
    #[cfg(feature = "hal-uvc")]
    /// Universal Video Class stream handle
    Uvc(uvc::stream::Handle<'a>),
}

impl<'a, 'b> StreamTrait<'b> for Stream<'a> {
    type Item = io::Result<Frame<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        match self {
            Stream::Custom(stream) => stream.next(),
            #[cfg(target_os = "linux")]
            Stream::V4l2(stream) => stream.next(),
            #[cfg(feature = "hal-uvc")]
            Stream::Uvc(stream) => stream.next(),
        }
    }
}
