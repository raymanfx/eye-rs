use std::io;
use std::ops::Sub;
use std::sync::Arc;

use crate::control;
use crate::format::{ImageFormat, PixelFormat};
use crate::hal::uvc::control::Control;
use crate::hal::uvc::stream::Handle as StreamHandle;
use crate::stream::ImageStream;
use crate::traits::Device;

pub struct Handle<'a> {
    inner: Arc<UvcHandle<'a>>,
    stream_fmt: uvc::StreamFormat,
}

impl<'a> Handle<'a> {
    pub fn new(bus_number: u8, device_address: u8) -> uvc::Result<Self> {
        let inner = UvcHandle::new(bus_number, device_address)?;

        Ok(Handle {
            inner: Arc::new(inner),
            stream_fmt: uvc::StreamFormat {
                width: 1280,
                height: 720,
                fps: 30,
                format: uvc::FrameFormat::Any,
            },
        })
    }
}

impl<'a> Device<'a> for Handle<'a> {
    fn query_formats(&self) -> io::Result<Vec<ImageFormat>> {
        let mut formats = Vec::new();

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

                        formats.push(ImageFormat::new(
                            frame_desc.width() as u32,
                            frame_desc.height() as u32,
                            pixfmt,
                        ));
                    });
            });

        Ok(formats)
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

    fn format(&self) -> io::Result<ImageFormat> {
        Ok(ImageFormat::new(
            self.stream_fmt.width,
            self.stream_fmt.height,
            PixelFormat::Rgb(24),
        ))
    }

    fn set_format(&mut self, fmt: &ImageFormat) -> io::Result<()> {
        if fmt.pixfmt != PixelFormat::Rgb(24) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only RGB24 is supported",
            ));
        }

        // helper - should really be in the standard library..
        fn absdiff<T: Sub<Output = T> + Ord>(x: T, y: T) -> T {
            if y > x {
                y - x
            } else {
                x - y
            }
        }

        if let Some(fmt) = self.inner.handle.get_preferred_format(|x, y| {
            // match against the resolution
            let error_x = ((absdiff(fmt.width, x.width)) as f64).sqrt()
                + ((absdiff(fmt.height, x.height)) as f64).sqrt();
            let error_y = ((absdiff(fmt.width, y.width)) as f64).sqrt()
                + ((absdiff(fmt.height, y.height)) as f64).sqrt();

            if error_x == error_y {
                // prefer lower frame times
                if x.fps >= y.fps {
                    x
                } else {
                    y
                }
            } else if error_x <= error_y {
                x
            } else {
                y
            }
        }) {
            self.stream_fmt = fmt;
        }

        Ok(())
    }

    fn stream(&self) -> io::Result<ImageStream<'a>> {
        let dev_handle = self.inner.clone();
        let dev_handle_ptr = &*dev_handle.handle as *const uvc::DeviceHandle;
        let dev_handle_ref = unsafe { &*dev_handle_ptr as &uvc::DeviceHandle };

        let stream_handle = match dev_handle_ref.get_stream_handle_with_format(self.stream_fmt) {
            Ok(handle) => handle,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };

        let format = self.format()?;
        let stream = match StreamHandle::new(dev_handle, stream_handle, self.stream_fmt) {
            Ok(stream) => stream,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };
        Ok(ImageStream::new(Box::new(stream), format))
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
