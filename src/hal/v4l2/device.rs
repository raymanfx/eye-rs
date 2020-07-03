use std::{io, path::Path};

use v4l::capture::{Device as CaptureDevice, Format as CaptureFormat};
use v4l::DeviceList;
use v4l::FourCC as FourCC_;

use ffimage::packed::DynamicImageView;

use crate::format::{Format, FourCC};
use crate::hal::traits::Device;
use crate::hal::v4l2::stream::PlatformStream;
use crate::hal::DeviceInfo;
use crate::traits::Stream;

pub(crate) struct PlatformList {}

impl PlatformList {
    pub fn enumerate() -> Vec<DeviceInfo> {
        let mut list = Vec::new();
        let platform_list = DeviceList::new();

        for dev in platform_list {
            let index = dev.index();
            let name = dev.name();
            let caps = dev.query_caps();
            if index.is_none() || name.is_none() || caps.is_err() {
                continue;
            }

            let caps = caps.unwrap();
            // For now, require video capture and streaming capabilities.
            // Very old devices may only support the read() I/O mechanism, so support for those
            // might be added in the future. Every recent (released during the last ten to twenty
            // years) webcam should support streaming though.
            let capture_flag = v4l::capability::Flags::VIDEO_CAPTURE;
            let streaming_flag = v4l::capability::Flags::STREAMING;
            if caps.capabilities & capture_flag != capture_flag
                || caps.capabilities & streaming_flag != streaming_flag
            {
                continue;
            }

            list.push(DeviceInfo {
                index: index.unwrap() as u32,
                name: name.unwrap(),
            })
        }

        list
    }
}

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
