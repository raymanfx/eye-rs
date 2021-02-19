use std::sync::Arc;
use std::{io, time};

use crate::control;
use crate::format::PixelFormat;
use crate::hal::uvc::control::Control;
use crate::hal::uvc::stream::Handle as StreamHandle;
use crate::stream::{
    Descriptor as StreamDescriptor, Descriptors as StreamDescriptors, ImageStream,
};
use crate::traits::Device;

pub struct Handle<'a> {
    inner: Arc<UvcHandle<'a>>,
}

impl<'a> Handle<'a> {
    pub fn new(bus_number: u8, device_address: u8) -> uvc::Result<Self> {
        let inner = UvcHandle::new(bus_number, device_address)?;

        Ok(Handle {
            inner: Arc::new(inner),
        })
    }
}

impl<'a> Device<'a> for Handle<'a> {
    fn query_streams(&self) -> io::Result<StreamDescriptors> {
        let mut streams = Vec::new();

        self.inner
            .handle
            .supported_formats()
            .into_iter()
            .for_each(|fmt_desc| {
                fmt_desc
                    .supported_formats()
                    .into_iter()
                    .for_each(|frame_desc| {
                        let pixfmt = match frame_desc.subtype() {
                            uvc::DescriptionSubtype::FormatMJPEG
                            | uvc::DescriptionSubtype::FrameMJPEG => PixelFormat::Jpeg,
                            _ => PixelFormat::Rgb(24),
                        };

                        for interval in frame_desc.intervals_duration() {
                            streams.push(StreamDescriptor {
                                width: frame_desc.width() as u32,
                                height: frame_desc.height() as u32,
                                pixfmt: pixfmt.clone(),
                                interval,
                            });
                        }
                    });
            });

        Ok(StreamDescriptors { streams })
    }

    fn query_controls(&self) -> io::Result<Vec<control::Control>> {
        let controls = Control::all()
            .into_iter()
            .map(|ctrl| <control::Control>::from(&ctrl))
            .collect();
        Ok(controls)
    }

    fn control(&self, id: u32) -> io::Result<control::Value> {
        match Control::from_id(id) {
            Some(ctrl) => ctrl.get(&self.inner.handle),
            None => Err(io::Error::new(io::ErrorKind::Other, "unknown control ID")),
        }
    }

    fn set_control(&mut self, _id: u32, _val: &control::Value) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "not supported"))
    }

    fn preferred_stream(
        &self,
        f: &dyn Fn(StreamDescriptor, StreamDescriptor) -> StreamDescriptor,
    ) -> io::Result<StreamDescriptor> {
        let preferred = self.inner.handle.get_preferred_format(|x, y| {
            let _x = StreamDescriptor {
                width: x.width,
                height: x.height,
                pixfmt: PixelFormat::Rgb(24),
                interval: time::Duration::from_secs_f64(1.0 / x.fps as f64),
            };
            let _y = StreamDescriptor {
                width: y.width,
                height: y.height,
                pixfmt: PixelFormat::Rgb(24),
                interval: time::Duration::from_secs_f64(1.0 / y.fps as f64),
            };
            let _preferred = f(_x, _y);

            if _preferred.width == x.width && _preferred.height == x.height {
                x
            } else {
                y
            }
        });

        let preferred = match preferred {
            Some(fmt) => fmt,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to query formats",
                ))
            }
        };

        Ok(StreamDescriptor {
            width: preferred.width,
            height: preferred.height,
            pixfmt: PixelFormat::Rgb(24),
            interval: time::Duration::from_secs_f64(1.0 / preferred.fps as f64),
        })
    }

    fn start_stream(&self, desc: &StreamDescriptor) -> io::Result<ImageStream<'a>> {
        let dev_handle = self.inner.clone();
        let dev_handle_ptr = &*dev_handle.handle as *const uvc::DeviceHandle;
        let dev_handle_ref = unsafe { &*dev_handle_ptr as &uvc::DeviceHandle };

        let stream_format = self.inner.handle.get_preferred_format(|x, y| {
            if x.width == desc.width && y.width == desc.width {
                if x.fps > y.fps {
                    x
                } else {
                    y
                }
            } else {
                x
            }
        });

        let stream_format = match stream_format {
            Some(fmt) => fmt,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to query formats",
                ))
            }
        };

        let stream_handle = match dev_handle_ref.get_stream_handle_with_format(stream_format) {
            Ok(handle) => handle,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };

        let stream = match StreamHandle::new(dev_handle, stream_handle, stream_format) {
            Ok(stream) => stream,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };
        Ok(ImageStream::new(Box::new(stream)))
    }
}

pub struct UvcHandle<'a> {
    handle: Box<uvc::DeviceHandle<'a>>,
    _dev: Box<uvc::Device<'a>>,
    _ctx: Box<uvc::Context<'a>>,
}

impl<'a> UvcHandle<'a> {
    pub fn new(bus_number: u8, device_address: u8) -> uvc::Result<Self> {
        let ctx = Box::new(uvc::Context::new()?);
        let ctx_ptr = &*ctx as *const uvc::Context;
        let ctx_ref = unsafe { &*ctx_ptr as &uvc::Context };

        let dev =
            if let Some(dev) = ctx_ref.devices()?.into_iter().find(|dev| {
                dev.bus_number() == bus_number && dev.device_address() == device_address
            }) {
                Box::new(dev)
            } else {
                return Err(uvc::Error::NotFound);
            };
        let dev_ptr = &*dev as *const uvc::Device;
        let dev_ref = unsafe { &*dev_ptr as &uvc::Device };

        let handle = Box::new(dev_ref.open()?);

        Ok(UvcHandle {
            handle,
            _dev: dev,
            _ctx: ctx,
        })
    }
}
