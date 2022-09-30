use std::cell::Cell;
use std::convert::TryInto;
use std::{io, time};

use openpnp_capture as pnp;
use openpnp_capture_sys as sys;

use crate::control;
use crate::error::{Error, ErrorKind, Result};
use crate::format::PixelFormat;
use crate::platform::openpnp::control as pnp_ctrl;
use crate::platform::openpnp::stream::Handle as StreamHandle;
use crate::stream;
use crate::traits::Device;

pub struct Handle {
    inner: pnp::Device,
    stream_id: Cell<Option<sys::CapStream>>,
}

impl Handle {
    pub fn new(index: u32) -> Option<Self> {
        let dev = Handle {
            inner: pnp::Device::new(index)?,
            stream_id: Cell::new(None),
        };
        Some(dev)
    }

    pub fn with_uri<S: Into<String>>(uri: S) -> io::Result<Self> {
        let uri = uri.into();
        if uri.starts_with("pnp://") {
            let elems: Vec<&str> = uri[6..].split(':').collect();
            if elems.is_empty() {
                return Err(io::Error::new(io::ErrorKind::Other, "missing index in URI"));
            }

            let index = if let Ok(index) = elems[0].parse::<u32>() {
                index
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "malformed index in URI",
                ));
            };

            Self::new(index).ok_or(io::Error::new(
                io::ErrorKind::Other,
                "failed to create OpenPnP capture instance",
            ))
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "not a pnp:// URI!"))
        }
    }
}

impl<'a> Device<'a> for Handle {
    type Stream = StreamHandle;

    fn streams(&self) -> Result<Vec<stream::Descriptor>> {
        let plat_formats = self.inner.formats();
        let mut formats: Vec<stream::Descriptor> = plat_formats
            .into_iter()
            .map(|fmt| stream::Descriptor {
                width: fmt.width,
                height: fmt.height,
                pixfmt: PixelFormat::Custom(fmt.fourcc.to_string()),
                interval: time::Duration::from_secs_f32(1.0 / fmt.fps as f32),
            })
            .collect();

        // check whether RGB24 is natively supported
        let mut rgb24_native = false;
        for fmt in &formats {
            if fmt.pixfmt == PixelFormat::Rgb(24) {
                rgb24_native = true;
                break;
            }
        }

        if !rgb24_native {
            // emulate RGB24
            // add all the resolutions, but make RGB24 the only pixelformat
            let mut rgb_formats: Vec<stream::Descriptor> = Vec::new();
            for fmt in &formats {
                let mut found = false;
                for rgb_fmt in &rgb_formats {
                    if rgb_fmt.width == fmt.width && rgb_fmt.height == fmt.height {
                        found = true;
                        break;
                    }
                }

                if !found {
                    rgb_formats.push(stream::Descriptor {
                        width: fmt.width,
                        height: fmt.height,
                        pixfmt: PixelFormat::Rgb(24),
                        interval: fmt.interval,
                    })
                }
            }

            formats.extend(rgb_formats);
        }

        Ok(formats)
    }

    fn start_stream(&self, desc: &stream::Descriptor) -> Result<Self::Stream> {
        let fourcc = match desc.pixfmt.clone() {
            PixelFormat::Rgb(24) => *b"RGB3",
            PixelFormat::Custom(repr) => {
                if repr.len() == 4 {
                    repr.as_bytes().try_into().unwrap()
                } else {
                    return Err(Error::new(ErrorKind::NotSupported, "invalid fourcc"));
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::NotSupported,
                    "cannot map pixfmt to fourcc",
                ))
            }
        };

        let fmt = pnp::Format {
            width: desc.width,
            height: desc.height,
            fourcc: pnp::format::FourCC::new(&fourcc),
            bpp: 0,
            fps: 0,
        };

        let handle = StreamHandle::new(&self.inner, &fmt)?;
        self.stream_id.set(Some(handle.inner.id()));
        Ok(handle)
    }

    fn controls(&self) -> Result<Vec<control::Descriptor>> {
        // Ensure a stream is currently open.
        // This is required because openpnp is weird in that it insists to perform control
        // operations on stream object instead of device ones.
        let stream_id = self
            .stream_id
            .get()
            .ok_or(Error::new(ErrorKind::Other, "stream not running"))?;

        let pnp_ctx = pnp::context::CONTEXT.lock().unwrap().inner;
        let controls = pnp_ctrl::all(pnp_ctx, stream_id).into_iter().collect();
        Ok(controls)
    }

    fn control(&self, id: u32) -> Result<control::State> {
        // Ensure a stream is currently open.
        // This is required because openpnp is weird in that it insists to perform control
        // operations on stream object instead of device ones.
        let stream_id = self
            .stream_id
            .get()
            .ok_or(Error::new(ErrorKind::Other, "stream not running"))?;

        let pnp_ctx = pnp::context::CONTEXT.lock().unwrap().inner;
        pnp_ctrl::read(pnp_ctx, stream_id, id)
    }

    fn set_control(&mut self, id: u32, val: &control::State) -> Result<()> {
        // Ensure a stream is currently open.
        // This is required because openpnp is weird in that it insists to perform control
        // operations on stream object instead of device ones.
        let stream_id = self
            .stream_id
            .get()
            .ok_or(Error::new(ErrorKind::Other, "stream not running"))?;

        let pnp_ctx = pnp::context::CONTEXT.lock().unwrap().inner;
        pnp_ctrl::write(pnp_ctx, stream_id, id, val)
    }
}
