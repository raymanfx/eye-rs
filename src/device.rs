use std::collections::HashMap;
use std::io;

use crate::colorconvert::Converter;
use crate::control;
use crate::format::PixelFormat;
use crate::stream::{
    ConvertStream, Descriptor as StreamDescriptor, Flags as StreamFlags, FrameStream,
};
use crate::traits::Device as DeviceTrait;

/// A transparent wrapper type for native platform devices.
pub struct Device<'a> {
    // actual platform device implementation
    inner: Box<dyn 'a + DeviceTrait<'a>>,
    // pixelformat emulation
    emulated_formats: HashMap<PixelFormat, PixelFormat>,
}

impl<'a> Device<'a> {
    pub fn with_uri<S: AsRef<str>>(_uri: S) -> io::Result<Self> {
        let _uri = _uri.as_ref();
        let mut inner: Option<Box<dyn 'a + DeviceTrait<'a>>> = None;

        #[cfg(target_os = "linux")]
        if _uri.starts_with("v4l://") {
            let path = _uri[6..].to_string();
            inner = Some(Box::new(crate::hal::v4l2::device::Handle::with_path(path)?));
        }

        #[cfg(feature = "hal-uvc")]
        if _uri.starts_with("uvc://") {
            let elems: Vec<&str> = _uri[6..].split(':').collect();
            if elems.len() < 2 {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to open device",
                ));
            }

            let bus_number = if let Ok(index) = elems[0].parse::<u8>() {
                index
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid URI"));
            };
            let device_address = if let Ok(addr) = elems[1].parse::<u8>() {
                addr
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid URI"));
            };

            inner = if let Ok(inner) =
                crate::hal::uvc::device::Handle::new(bus_number, device_address)
            {
                Some(Box::new(inner))
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to create UVC context",
                ));
            };
        }

        let inner = if let Some(dev) = inner {
            dev
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No suitable backend available",
            ));
        };

        let native_streams = inner.query_streams()?;
        let converter_formats = Converter::formats();
        let mut emulated_formats = HashMap::new();

        converter_formats.into_iter().for_each(|mappings| {
            if let Some(stream) = native_streams
                .iter()
                .find(|stream| stream.pixfmt == mappings.0)
            {
                mappings.1.iter().for_each(|pixfmt| {
                    // check whether there is already a native stream with this format
                    if let Some(_) = native_streams
                        .iter()
                        .find(|stream| &stream.pixfmt == pixfmt)
                    {
                        return;
                    }

                    if !emulated_formats.contains_key(pixfmt) {
                        emulated_formats
                            .entry(pixfmt.clone())
                            .or_insert(stream.pixfmt.clone());
                    }
                })
            }
        });

        Ok(Device {
            inner,
            emulated_formats,
        })
    }
}

impl<'a> DeviceTrait<'a> for Device<'a> {
    fn query_streams(&self) -> io::Result<Vec<StreamDescriptor>> {
        // get all the native streams
        let native = self.inner.query_streams()?;

        // now check which formats we can emulate
        let mut emulated: Vec<StreamDescriptor> = Vec::new();
        self.emulated_formats.iter().for_each(|mapping| {
            let streams = native.iter().filter_map(|stream| {
                if &stream.pixfmt == mapping.1 {
                    Some(stream)
                } else {
                    None
                }
            });

            streams.for_each(|stream| {
                emulated.push(StreamDescriptor {
                    width: stream.width,
                    height: stream.height,
                    pixfmt: mapping.0.clone(),
                    interval: stream.interval,
                    flags: stream.flags | StreamFlags::EMULATED,
                });
            });
        });

        let streams = native.into_iter().chain(emulated.into_iter()).collect();
        Ok(streams)
    }

    fn query_controls(&self) -> io::Result<Vec<control::Control>> {
        self.inner.query_controls()
    }

    fn control(&self, id: u32) -> io::Result<control::Value> {
        self.inner.control(id)
    }

    fn set_control(&mut self, id: u32, val: &control::Value) -> io::Result<()> {
        self.inner.set_control(id, val)
    }

    fn preferred_stream(
        &self,
        f: &dyn Fn(StreamDescriptor, StreamDescriptor) -> StreamDescriptor,
    ) -> io::Result<StreamDescriptor> {
        self.inner.preferred_stream(f)
    }

    fn start_stream(&self, desc: &StreamDescriptor) -> io::Result<FrameStream<'a>> {
        if let Some(source_fmt) = self.emulated_formats.get(&desc.pixfmt) {
            // start the native stream with the base pixfmt
            let mut desc = desc.clone();
            desc.pixfmt = source_fmt.clone();
            let native_stream = self.inner.start_stream(&desc)?;

            // create the instance that converts the frames for us
            Ok(FrameStream::new(Box::new(ConvertStream {
                inner: native_stream,
                map: (desc.pixfmt.clone(), source_fmt.clone()),
            })))
        } else {
            // no emulation required
            self.inner.start_stream(desc)
        }
    }
}
