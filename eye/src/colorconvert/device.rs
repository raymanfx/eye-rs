use eye_hal::error::{Error, ErrorKind, Result};
use eye_hal::platform::Context as PlatformContext;
use eye_hal::platform::{Device as PlatformDevice, Stream as PlatformStream};
use eye_hal::traits::{Context, Device as DeviceTrait};
use eye_hal::{control, stream};

use crate::colorconvert::codec;
use crate::colorconvert::stream::CodecStream;

/// A transparent wrapper type for native platform devices.
pub struct Device<'a> {
    // actual platform device implementation
    inner: PlatformDevice<'a>,
}

impl<'a> Device<'a> {
    pub fn new(dev: PlatformDevice<'a>) -> Result<Self> {
        Ok(Device { inner: dev })
    }

    pub fn with_uri<S: AsRef<str>>(uri: S) -> Result<Self> {
        let uri = uri.as_ref();
        let ctx = PlatformContext::all().next().ok_or(Error::new(
            ErrorKind::Other,
            "no platform context available",
        ))?;
        let inner = ctx.open_device(uri)?;

        Self::new(inner)
    }
}

impl<'a> DeviceTrait<'a> for Device<'a> {
    type Stream = PlatformStream<'a>;

    fn streams(&self) -> Result<Vec<stream::Descriptor>> {
        // get all the native streams
        let mut streams = self.inner.streams()?;

        // now check which formats we can emulate
        for blueprint in codec::blueprints() {
            for chain in blueprint.src_fmts().iter().zip(blueprint.dst_fmts().iter()) {
                if streams.iter().any(|stream| stream.pixfmt == *chain.0)
                    && !streams.iter().any(|stream| stream.pixfmt == *chain.1)
                {
                    // collect all streams with this pixfmt
                    let _streams: Vec<stream::Descriptor> = streams
                        .iter()
                        .filter_map(|stream| {
                            if stream.pixfmt == *chain.0 {
                                Some(stream.clone())
                            } else {
                                None
                            }
                        })
                        .collect();

                    _streams.into_iter().for_each(|stream| {
                        streams.push(stream::Descriptor {
                            width: stream.width,
                            height: stream.height,
                            pixfmt: chain.1.clone(),
                            interval: stream.interval,
                        });
                    });
                }
            }
        }

        Ok(streams)
    }

    fn start_stream(&self, desc: &stream::Descriptor) -> Result<Self::Stream> {
        let native_streams = self.inner.streams()?;
        if native_streams
            .iter()
            .any(|stream| stream.pixfmt == desc.pixfmt)
        {
            // no emulation required
            return self.inner.start_stream(desc);
        }

        // find a supported format mapping
        let blueprints: Vec<Box<dyn codec::Blueprint>> = codec::blueprints()
            .into_iter()
            .filter(|bp| bp.dst_fmts().iter().any(|pixfmt| *pixfmt == desc.pixfmt))
            .collect();
        let src_fmt = if let Some(pixfmt) = blueprints.iter().find_map(|bp| {
            for pixfmt in bp.src_fmts() {
                match native_streams.iter().find(|desc| desc.pixfmt == pixfmt) {
                    Some(desc) => return Some(desc.pixfmt.clone()),
                    None => continue,
                }
            }

            None
        }) {
            pixfmt
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "no codec blueprint for native pixfmt",
            ));
        };

        // get the codec blueprint
        let blueprint = if let Some(bp) = blueprints
            .into_iter()
            .find(|bp| bp.src_fmts().into_iter().any(|pixfmt| pixfmt == src_fmt))
        {
            bp
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                format!("no codec blueprint for {} -> {}", src_fmt, desc.pixfmt),
            ));
        };

        // create the codec instance
        let inparams = codec::Parameters {
            pixfmt: src_fmt.clone(),
            width: desc.width,
            height: desc.height,
        };
        let outparams = codec::Parameters {
            pixfmt: desc.pixfmt.clone(),
            width: desc.width,
            height: desc.height,
        };
        let codec = match blueprint.instantiate(inparams, outparams) {
            Ok(instance) => instance,
            Err(_e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "failed to create codec instance",
                ))
            }
        };

        // start the native stream with the base pixfmt
        let mut source_fmt = desc.clone();
        source_fmt.pixfmt = src_fmt;
        let native_stream = self.inner.start_stream(&source_fmt)?;

        // create the instance that converts the frames for us
        return Ok(PlatformStream::Custom(Box::new(CodecStream {
            inner: native_stream,
            codec,
            buf: Vec::new(),
        })));
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
