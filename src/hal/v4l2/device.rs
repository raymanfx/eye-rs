use std::{convert::TryFrom, io, path::Path};

use v4l::capture::{Device as CaptureDevice, Format as CaptureFormat};
use v4l::control::{Control, MenuItem as ControlMenuItem, Type as ControlType};
use v4l::device::List;
use v4l::device::QueryDevice;
use v4l::FourCC as FourCC_;

use ffimage::packed::dynamic::ImageView;

use crate::control;
use crate::device::{ControlFlags, ControlInfo, FormatInfo, Info as DeviceInfo};
use crate::format::{Format, FourCC, PixelFormat};
use crate::hal::traits::{Device, Stream};
use crate::hal::v4l2::stream::PlatformStream;

pub struct PlatformList {}

impl PlatformList {
    pub fn enumerate() -> Vec<DeviceInfo> {
        let mut list = Vec::new();
        let platform_list = List::new();

        for dev in platform_list {
            let index = dev.index();
            let name = dev.name();

            if index.is_none() || name.is_none() {
                continue;
            }
            let index = index.unwrap();
            let name = name.unwrap();

            let dev = PlatformDevice::new(index);
            if dev.is_err() {
                continue;
            }
            let dev = dev.unwrap();

            let caps = dev.inner.query_caps();
            if caps.is_err() {
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
                index: index as u32,
                name,
            })
        }

        list
    }
}

pub struct PlatformDevice {
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
}

impl Device for PlatformDevice {
    fn query_formats(&self) -> io::Result<Vec<FormatInfo>> {
        let mut formats = Vec::new();
        let plat_formats = self.inner.enum_formats()?;

        for format in plat_formats {
            let plat_sizes = self.inner.enum_framesizes(format.fourcc);
            if plat_sizes.is_err() {
                continue;
            }
            let mut info = FormatInfo {
                pixfmt: PixelFormat::from(FourCC::new(&format.fourcc.repr)),
                resolutions: Vec::new(),
                emulated: format.flags & v4l::format::Flags::EMULATED
                    == v4l::format::Flags::EMULATED,
            };
            for plat_size in plat_sizes.unwrap() {
                // TODO: consider stepwise formats
                if let v4l::framesize::FrameSizeEnum::Discrete(size) = plat_size.size {
                    info.resolutions.push((size.width, size.height));
                }
            }
            formats.push(info);
        }

        Ok(formats)
    }

    fn query_controls(&self) -> io::Result<Vec<ControlInfo>> {
        let mut controls = Vec::new();
        let plat_controls = self.inner.query_controls()?;

        for control in plat_controls {
            // The v4l docs say applications should ignore permanently disabled controls.
            if control.flags & v4l::control::Flags::DISABLED == v4l::control::Flags::DISABLED {
                continue;
            }

            let mut repr = control::Representation::Unknown;
            match control.typ {
                ControlType::Integer | ControlType::Integer64 => {
                    let constraints = control::Integer {
                        range: (control.minimum as i64, control.maximum as i64),
                        step: control.step as u64,
                        default: control.default as i64,
                    };
                    repr = control::Representation::Integer(constraints);
                }
                ControlType::Boolean => {
                    repr = control::Representation::Boolean;
                }
                ControlType::Menu => {
                    let mut items = Vec::new();
                    if let Some(plat_items) = control.items {
                        for plat_item in plat_items {
                            match plat_item.1 {
                                ControlMenuItem::Name(name) => {
                                    items.push(control::MenuItem::String(name));
                                }
                                ControlMenuItem::Value(value) => {
                                    items.push(control::MenuItem::Integer(value));
                                }
                            }
                        }
                    }
                    repr = control::Representation::Menu(items);
                }
                ControlType::Button => {
                    repr = control::Representation::Button;
                }
                ControlType::String => {
                    repr = control::Representation::String;
                }
                ControlType::Bitmask => {
                    repr = control::Representation::Bitmask;
                }
                _ => {}
            }

            let mut flags = ControlFlags::NONE;
            if control.flags & v4l::control::Flags::READ_ONLY == v4l::control::Flags::READ_ONLY {
                flags |= ControlFlags::READ_ONLY;
            }
            if control.flags & v4l::control::Flags::WRITE_ONLY == v4l::control::Flags::WRITE_ONLY {
                flags |= ControlFlags::WRITE_ONLY;
            }
            if control.flags & v4l::control::Flags::GRABBED == v4l::control::Flags::GRABBED {
                flags |= ControlFlags::GRABBED;
            }
            if control.flags & v4l::control::Flags::INACTIVE == v4l::control::Flags::INACTIVE {
                flags |= ControlFlags::INACTIVE;
            }

            controls.push(ControlInfo {
                id: control.id,
                name: control.name,
                flags,
                repr,
            })
        }

        Ok(controls)
    }

    fn control(&self, id: u32) -> io::Result<Option<control::Value>> {
        let ctrl = self.inner.control(id)?;
        match ctrl {
            Control::Value(val) => Ok(Some(control::Value::Integer(val as i64))),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "control type cannot be mapped",
            )),
        }
    }

    fn set_control(&mut self, id: u32, val: &control::Value) -> io::Result<()> {
        match val {
            control::Value::Integer(val) => {
                let ctrl = Control::Value(*val as i32);
                self.inner.set_control(id, ctrl)?;
            }
            control::Value::Boolean(val) => {
                let ctrl = Control::Value(*val as i32);
                self.inner.set_control(id, ctrl)?;
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "control type cannot be mapped",
                ))
            }
        }

        Ok(())
    }

    fn format(&self) -> io::Result<Format> {
        let fmt = self.inner.format()?;
        Ok(Format::with_stride(
            fmt.width,
            fmt.height,
            PixelFormat::from(FourCC::new(&fmt.fourcc.repr)),
            fmt.stride as usize,
        ))
    }

    fn set_format(&mut self, fmt: &Format) -> io::Result<Format> {
        let fourcc = FourCC::try_from(fmt.pixfmt);
        if fourcc.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to map pixelformat to fourcc",
            ));
        }

        let fmt = CaptureFormat::new(fmt.width, fmt.height, FourCC_::new(&fourcc.unwrap().repr));
        self.inner.set_format(&fmt)?;
        self.format()
    }

    fn stream<'a>(&self) -> io::Result<Box<dyn Stream<Item = ImageView<'a>> + 'a>> {
        let stream = PlatformStream::new(self)?;
        Ok(Box::new(stream))
    }
}
