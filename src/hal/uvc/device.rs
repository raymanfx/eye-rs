use std::io;
use std::ops::Sub;
use std::sync::Arc;

use crate::control;
use crate::format::{Format, FourCC, PixelFormat};
use crate::hal::uvc::stream::PlatformStream;
use crate::traits::{Device, ImageStream};

pub struct PlatformDevice<'a> {
    handle: Arc<uvc::DeviceHandle<'a>>,
    _dev: Arc<uvc::Device<'a>>,
    _ctx: Arc<uvc::Context<'a>>,

    stream_fmts: Vec<Format>,
    stream_fmt: Option<Format>,
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

        let mut dev = PlatformDevice {
            handle,
            _dev: dev,
            _ctx: ctx,
            stream_fmts: Vec::new(),
            stream_fmt: None,
        };

        // query available formats
        if let Ok(fmts) = dev.query_formats() {
            if fmts.len() > 0 {
                dev.stream_fmt = Some(fmts[0]);
                dev.stream_fmts = fmts;
            }
        }

        Ok(dev)
    }
}

impl<'a> Device<'a> for PlatformDevice<'a> {
    fn query_formats(&self) -> io::Result<Vec<Format>> {
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
                                PixelFormat::from(FourCC::new(b"MJPG"))
                            }
                            _ => PixelFormat::from(FourCC::new(b"RGB3")),
                        };

                        let fmt = Format {
                            width: frame_desc.width() as u32,
                            height: frame_desc.height() as u32,
                            pixfmt,
                            stride: None,
                        };

                        formats.push(fmt);
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

    fn format(&self) -> io::Result<Format> {
        match self.stream_fmt {
            Some(fmt) => Ok(fmt),
            None => Err(io::Error::new(io::ErrorKind::Other, "not set")),
        }
    }

    fn set_format(&mut self, fmt: &Format) -> io::Result<Format> {
        if fmt.pixfmt != PixelFormat::Rgb(24) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "only RGB24 is supported",
            ));
        }

        if self.stream_fmts.len() == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "no available formats"));
        }

        // helper - should really be in the standard library..
        fn absdiff<T: Sub<Output = T> + Ord>(x: T, y: T) -> T {
            if y > x {
                y - x
            } else {
                x - y
            }
        }

        // find the best match using L2 error metric
        let mut prev = self.stream_fmts[0];
        for stream_fmt in &self.stream_fmts {
            let prev_error = ((absdiff(fmt.width, prev.width)) as f64).sqrt()
                + ((absdiff(fmt.height, prev.height)) as f64).sqrt();
            let error = ((absdiff(fmt.width, stream_fmt.width)) as f64).sqrt()
                + ((absdiff(fmt.height, stream_fmt.height)) as f64).sqrt();
            if error < prev_error {
                prev = *stream_fmt;
            }

            if error == 0.0 {
                // found the exact format
                break;
            }
        }

        self.stream_fmt = Some(prev);
        Ok(prev)
    }

    fn stream(&self) -> io::Result<Box<ImageStream<'a>>> {
        let format = self.format()?;
        let uvc_fmt = uvc::StreamFormat {
            width: format.width,
            height: format.height,
            fps: 30,
            format: uvc::FrameFormat::Any,
        };

        let handle_ptr = &*self.handle as *const uvc::DeviceHandle;
        let handle_ref = unsafe { &*handle_ptr as &uvc::DeviceHandle };

        let uvc_stream = match handle_ref.get_stream_handle_with_format(uvc_fmt) {
            Ok(handle) => handle,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };

        let stream = PlatformStream::new(uvc_stream, format);
        Ok(Box::new(stream))
    }
}
