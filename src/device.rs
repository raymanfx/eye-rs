use std::io;

use crate::colorconvert::Converter;
use crate::control;
use crate::format::{Format, PixelFormat};
use crate::stream::TransparentStream;
use crate::traits::{Device as DeviceTrait, ImageStream};

/// A transparent wrapper type for native platform devices.
pub struct Device<'a> {
    inner: Box<dyn 'a + DeviceTrait<'a>>,

    // active format
    emulated_format: Option<Format>,
}

impl<'a> Device<'a> {
    pub fn with_uri<S: AsRef<str>>(_uri: S) -> io::Result<Self> {
        let _uri = _uri.as_ref();

        #[cfg(target_os = "linux")]
        if _uri.starts_with("v4l://") {
            let path = _uri[6..].to_string();
            let inner = crate::hal::v4l2::device::PlatformDevice::with_path(path)?;
            return Ok(Device {
                inner: Box::new(inner),
                emulated_format: None,
            });
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

            let inner = crate::hal::uvc::device::PlatformDevice::new(bus_number, device_address);
            let inner = if let Ok(inner) = inner {
                inner
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to create UVC context",
                ));
            };
            return Ok(Device {
                inner: Box::new(inner),
                emulated_format: None,
            });
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "No suitable backend available",
        ))
    }

    /// Returns a list of format mappings where the first field is a native format and the second
    /// is a format which can be emulated by the common converter abstraction layer.
    fn query_emulated_formats(&self) -> io::Result<Vec<(PixelFormat, PixelFormat)>> {
        let converter_formats = Converter::formats();
        let mut emulated_formats: Vec<(PixelFormat, PixelFormat)> = Vec::new();

        let native_formats = self.inner.query_formats()?;
        for native_format in &native_formats {
            for emulated in &converter_formats {
                if emulated.0 == native_format.pixfmt {
                    // we can emulate formats based on this native format
                    for emulated_dst in &emulated.1 {
                        // first check if the format we can emulate is actually already supported
                        // natively
                        let mut emulate = true;
                        for format in &native_formats {
                            if format.pixfmt == *emulated_dst {
                                emulate = false;
                                break;
                            }
                        }

                        // now check whether we already emulate this format
                        for format in &emulated_formats {
                            if format.1 == *emulated_dst {
                                emulate = false;
                                break;
                            }
                        }

                        // skip emulation if we already support the format
                        if !emulate {
                            continue;
                        }

                        // looks like the format is not already supported, so let's add it
                        emulated_formats.push((native_format.pixfmt, *emulated_dst));
                    }
                }
            }
        }

        Ok(emulated_formats)
    }
}

impl<'a> DeviceTrait<'a> for Device<'a> {
    fn query_formats(&self) -> io::Result<Vec<Format>> {
        let mut formats = self.inner.query_formats()?;

        // add emulated formats
        let mut emulated_formats = Vec::new();

        for plat_format in &formats {
            for mapping in self.query_emulated_formats()? {
                if mapping.0 == plat_format.pixfmt {
                    // transparently add the emulated format
                    emulated_formats.push(Format {
                        width: plat_format.width,
                        height: plat_format.height,
                        pixfmt: mapping.1,
                        stride: None,
                    });
                }
            }
        }

        formats.extend(emulated_formats);
        Ok(formats)
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

    fn format(&self) -> io::Result<Format> {
        // in case of active format emulation, we don't need to query the actual device
        if self.emulated_format.is_some() {
            return Ok(self.emulated_format.unwrap());
        }

        self.inner.format()
    }

    fn set_format(&mut self, fmt: &Format) -> io::Result<Format> {
        let mut fmt = *fmt;

        // check whether we need to emulate the requested format
        let mut emulate = None;
        for format in self.query_emulated_formats()? {
            if format.1 == fmt.pixfmt {
                fmt.pixfmt = format.0;
                emulate = Some((format.0, format.1));
                break;
            }
        }

        if let Some(mapping) = emulate {
            // setup emulation
            let emulated_format = Format::new(fmt.width, fmt.height, mapping.1);
            self.emulated_format = Some(emulated_format);
        }

        self.inner.set_format(&fmt)?;
        self.format()
    }

    fn stream(&self) -> io::Result<Box<ImageStream<'a>>> {
        let native_format = self.inner.format()?;
        let native_stream = self.inner.stream()?;
        let mut stream = TransparentStream::new(native_stream, native_format);

        if let Some(format) = self.emulated_format {
            stream.map(native_format.pixfmt, format.pixfmt);
        }

        Ok(Box::new(stream))
    }
}
