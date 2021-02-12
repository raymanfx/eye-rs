use std::io;

use crate::control;
use crate::format::ImageFormat;
use crate::stream::ImageStream;
use crate::traits::Device as DeviceTrait;

/// A transparent wrapper type for native platform devices.
pub struct Device<'a> {
    inner: Box<dyn 'a + DeviceTrait<'a>>,
}

impl<'a> Device<'a> {
    pub fn with_uri<S: AsRef<str>>(_uri: S) -> io::Result<Self> {
        let _uri = _uri.as_ref();

        #[cfg(target_os = "linux")]
        if _uri.starts_with("v4l://") {
            let path = _uri[6..].to_string();
            let inner = crate::hal::v4l2::device::Handle::with_path(path)?;
            return Ok(Device {
                inner: Box::new(inner),
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

            let inner = crate::hal::uvc::device::Handle::new(bus_number, device_address);
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
            });
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "No suitable backend available",
        ))
    }
}

impl<'a> DeviceTrait<'a> for Device<'a> {
    fn query_formats(&self) -> io::Result<Vec<ImageFormat>> {
        self.inner.query_formats()
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

    fn format(&self) -> io::Result<ImageFormat> {
        self.inner.format()
    }

    fn set_format(&mut self, fmt: &ImageFormat) -> io::Result<()> {
        self.inner.set_format(&fmt)
    }

    fn stream(&self) -> io::Result<ImageStream<'a>> {
        let stream = self.inner.stream()?;
        Ok(stream)
    }
}
