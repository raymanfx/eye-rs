use std::collections::HashMap;

use eye_hal::error::Result;
use eye_hal::format::PixelFormat;
use eye_hal::platform::Context as PlatformContext;
use eye_hal::platform::{Device as PlatformDevice, Stream as PlatformStream};
use eye_hal::traits::{Context, Device as DeviceTrait};
use eye_hal::{control, stream};

use crate::colorconvert::Converter;
use crate::stream::ConvertStream;

/// A transparent wrapper type for native platform devices.
pub struct Device<'a> {
    // actual platform device implementation
    inner: PlatformDevice<'a>,
    // pixelformat emulation
    emulated_formats: HashMap<PixelFormat, PixelFormat>,
}

impl<'a> Device<'a> {
    pub fn new(dev: PlatformDevice<'a>) -> Result<Self> {
        let inner = dev;
        let native_streams = inner.streams()?;
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

    pub fn with_uri<S: AsRef<str>>(uri: S) -> Result<Self> {
        let uri = uri.as_ref();
        let ctx = PlatformContext::default();
        let inner = ctx.open_device(uri)?;

        Self::new(inner)
    }
}

impl<'a> DeviceTrait<'a> for Device<'a> {
    fn streams(&self) -> Result<Vec<stream::Descriptor>> {
        // get all the native streams
        let native = self.inner.streams()?;

        // now check which formats we can emulate
        let mut emulated: Vec<stream::Descriptor> = Vec::new();
        self.emulated_formats.iter().for_each(|mapping| {
            let streams = native.iter().filter_map(|stream| {
                if &stream.pixfmt == mapping.1 {
                    Some(stream)
                } else {
                    None
                }
            });

            streams.for_each(|stream| {
                emulated.push(stream::Descriptor {
                    width: stream.width,
                    height: stream.height,
                    pixfmt: mapping.0.clone(),
                    interval: stream.interval,
                });
            });
        });

        let streams = native.into_iter().chain(emulated.into_iter()).collect();
        Ok(streams)
    }

    fn start_stream(&self, desc: &stream::Descriptor) -> Result<PlatformStream<'a>> {
        if let Some(source_pixfmt) = self.emulated_formats.get(&desc.pixfmt) {
            // start the native stream with the base pixfmt
            let mut source_fmt = desc.clone();
            source_fmt.pixfmt = source_pixfmt.clone();
            let native_stream = self.inner.start_stream(&source_fmt)?;

            // create the instance that converts the frames for us
            Ok(PlatformStream::Custom(Box::new(ConvertStream {
                inner: native_stream,
                desc: desc.clone(),
                map: (source_pixfmt.clone(), desc.pixfmt.clone()),
            })))
        } else {
            // no emulation required
            self.inner.start_stream(desc)
        }
    }

    fn controls(&self) -> Result<Vec<control::Descriptor>> {
        self.inner.controls()
    }

    fn control(&self, id: u32) -> Result<control::State> {
        self.inner.control(id)
    }

    fn set_control(&mut self, id: u32, val: &control::State) -> Result<()> {
        self.inner.set_control(id, val)
    }
}
