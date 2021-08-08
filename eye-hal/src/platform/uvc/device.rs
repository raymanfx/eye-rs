use std::sync::Arc;

use crate::control;
use crate::error::{Error, ErrorKind, Result};
use crate::format::PixelFormat;
use crate::platform::uvc::control::Control;
use crate::platform::uvc::stream::Handle as StreamHandle;
use crate::stream;
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

    pub fn with_uri<S: Into<String>>(uri: S) -> uvc::Result<Self> {
        let uri = uri.into();
        if uri.starts_with("uvc://") {
            let elems: Vec<&str> = uri[6..].split(':').collect();
            if elems.len() < 2 {
                return Err(uvc::Error::Other);
            }

            let bus_number = if let Ok(index) = elems[0].parse::<u8>() {
                index
            } else {
                return Err(uvc::Error::Other);
            };
            let device_address = if let Ok(addr) = elems[1].parse::<u8>() {
                addr
            } else {
                return Err(uvc::Error::Other);
            };

            Self::new(bus_number, device_address)
        } else {
            Err(uvc::Error::Other)
        }
    }
}

impl<'a> Device<'a> for Handle<'a> {
    type Stream = StreamHandle<'a>;

    fn streams(&self) -> Result<Vec<stream::Descriptor>> {
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
                            streams.push(stream::Descriptor {
                                width: frame_desc.width() as u32,
                                height: frame_desc.height() as u32,
                                pixfmt: pixfmt.clone(),
                                interval,
                            });
                        }
                    });
            });

        Ok(streams)
    }

    fn controls(&self) -> Result<Vec<control::Descriptor>> {
        let controls = Control::all()
            .into_iter()
            .map(|ctrl| <control::Descriptor>::from(&ctrl))
            .collect();
        Ok(controls)
    }

    fn control(&self, id: u32) -> Result<control::State> {
        match Control::from_id(id) {
            Some(ctrl) => ctrl.get(&self.inner.handle),
            None => Err(Error::new(ErrorKind::Other, "unknown control ID")),
        }
    }

    fn set_control(&mut self, _id: u32, _val: &control::State) -> Result<()> {
        Err(Error::from(ErrorKind::NotSupported))
    }

    fn start_stream(&self, desc: &stream::Descriptor) -> Result<Self::Stream> {
        let dev_handle = self.inner.clone();
        let dev_handle_ptr = &*dev_handle.handle as *const uvc::DeviceHandle;
        let dev_handle_ref = unsafe { &*dev_handle_ptr as &uvc::DeviceHandle };

        let desc_fps = (1.0 / desc.interval.as_secs_f64()) as u64;
        let stream_format = self.inner.handle.get_preferred_format(|x, y| {
            if x.width == desc.width && x.height == desc.height && x.fps as u64 >= desc_fps {
                x
            } else if x.width == desc.width && x.height == desc.height {
                x
            } else {
                y
            }
        });

        let stream_format = match stream_format {
            Some(fmt) => {
                if fmt.width != desc.width || fmt.height != desc.height {
                    return Err(Error::new(ErrorKind::Other, "invalid stream descriptor"));
                }

                fmt
            }
            None => return Err(Error::new(ErrorKind::Other, "failed to query formats")),
        };

        let stream_handle = match dev_handle_ref.get_stream_handle_with_format(stream_format) {
            Ok(handle) => handle,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        };

        match StreamHandle::new(dev_handle, stream_handle) {
            Ok(handle) => Ok(handle),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
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
