use std::io;
use std::ops::Sub;
use std::sync::Arc;

use crate::control;
use crate::format::{pix, ImageFormat, PixelFormat};
use crate::hal::uvc::stream::PlatformStream;
use crate::traits::{Device, ImageStream};

pub struct PlatformDevice<'a> {
    handle: Arc<uvc::DeviceHandle<'a>>,
    _dev: Arc<uvc::Device<'a>>,
    _ctx: Arc<uvc::Context<'a>>,

    stream_fmt: uvc::StreamFormat,
}

impl<'a> PlatformDevice<'a> {
    pub fn new(bus_number: u8, device_address: u8) -> uvc::Result<Self> {
        let ctx = Arc::new(uvc::Context::new()?);
        let ctx_ptr = &*ctx as *const uvc::Context;
        let ctx_ref = unsafe { &*ctx_ptr as &uvc::Context };

        let dev =
            if let Some(dev) = ctx_ref.devices()?.into_iter().find(|dev| {
                dev.bus_number() == bus_number && dev.device_address() == device_address
            }) {
                Arc::new(dev)
            } else {
                return Err(uvc::Error::NotFound);
            };
        let dev_ptr = &*dev as *const uvc::Device;
        let dev_ref = unsafe { &*dev_ptr as &uvc::Device };

        let handle = Arc::new(dev_ref.open()?);

        Ok(PlatformDevice {
            handle,
            _dev: dev,
            _ctx: ctx,
            stream_fmt: uvc::StreamFormat {
                width: 1280,
                height: 720,
                fps: 30,
                format: uvc::FrameFormat::Any,
            },
        })
    }
}

impl<'a> Device<'a> for PlatformDevice<'a> {
    fn query_formats(&self) -> io::Result<Vec<ImageFormat>> {
        let mut formats = Vec::new();

        self.handle
            .supported_formats()
            .into_iter()
            .for_each(|fmt_desc| {
                fmt_desc
                    .supported_formats()
                    .into_iter()
                    .for_each(|frame_desc| {
                        let pixfmt = match frame_desc.subtype() {
                            uvc::DescriptionSubtype::FormatMJPEG
                            | uvc::DescriptionSubtype::FrameMJPEG => {
                                PixelFormat::Compressed(pix::Compressed::Jpeg)
                            }
                            _ => PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)),
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
        Err(io::Error::new(io::ErrorKind::Other, "not supported"))
    }

    fn control(&self, _id: u32) -> io::Result<control::Value> {
        Err(io::Error::new(io::ErrorKind::Other, "not supported"))
    }

    fn set_control(&mut self, _id: u32, _val: &control::Value) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Other, "not supported"))
    }

    fn format(&self) -> io::Result<ImageFormat> {
        Ok(ImageFormat::new(
            self.stream_fmt.width,
            self.stream_fmt.height,
            PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)),
        ))
    }

    fn set_format(&mut self, fmt: &ImageFormat) -> io::Result<ImageFormat> {
        if fmt.pixfmt != PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)) {
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

        if let Some(fmt) = self.handle.get_preferred_format(|x, y| {
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

        Ok(ImageFormat::new(
            self.stream_fmt.width,
            self.stream_fmt.height,
            PixelFormat::Uncompressed(pix::Uncompressed::Rgb(24)),
        ))
    }

    fn stream(&self) -> io::Result<Box<ImageStream<'a>>> {
        let handle_ptr = &*self.handle as *const uvc::DeviceHandle;
        let handle_ref = unsafe { &*handle_ptr as &uvc::DeviceHandle };

        let uvc_stream = match handle_ref.get_stream_handle_with_format(self.stream_fmt) {
            Ok(handle) => handle,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };

        let stream = PlatformStream::new(uvc_stream, self.stream_fmt);
        Ok(Box::new(stream))
    }
}
