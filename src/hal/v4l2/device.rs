use std::{io, path::Path};

use v4l::capture::{Device as CaptureDevice, Format as CaptureFormat};
use v4l::FourCC as FourCC_;

use ffimage::packed::DynamicImageView;

use crate::format::{Format, FourCC};
use crate::hal::traits::Device;
use crate::hal::v4l2::stream::PlatformStream;
use crate::traits::Stream;

pub(crate) struct PlatformDevice {
    inner: CaptureDevice,
}

impl PlatformDevice {
    pub fn new(index: usize) -> io::Result<Self> {
        let dev = PlatformDevice {
            inner: CaptureDevice::new(index)?,
        };
        Ok(dev)
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let dev = PlatformDevice {
            inner: CaptureDevice::with_path(path)?,
        };
        Ok(dev)
    }

    pub fn inner(&self) -> &CaptureDevice {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut CaptureDevice {
        &mut self.inner
    }
}

impl Device for PlatformDevice {
    fn get_format(&mut self) -> io::Result<Format> {
        let fmt = self.inner.get_format()?;
        Ok(Format::with_stride(
            fmt.width,
            fmt.height,
            FourCC::new(&fmt.fourcc.repr),
            fmt.stride as usize,
        ))
    }

    fn set_format(&mut self, fmt: &Format) -> io::Result<Format> {
        let fmt = CaptureFormat::new(fmt.width, fmt.height, FourCC_::new(&fmt.fourcc.repr));
        self.inner.set_format(&fmt)?;
        self.get_format()
    }

    fn stream<'a>(&'a mut self) -> io::Result<Box<dyn Stream<Item = DynamicImageView> + 'a>> {
        let stream = PlatformStream::new(self)?;
        Ok(Box::new(stream))
    }
}
