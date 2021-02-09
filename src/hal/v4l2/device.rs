use std::{convert::TryInto, io, path::Path};

use v4l::control::{Control, MenuItem as ControlMenuItem, Type as ControlType};
use v4l::video::Capture;
use v4l::Device;
use v4l::Format as CaptureFormat;
use v4l::FourCC as FourCC_;

use crate::control;
use crate::format::{ImageFormat, PixelFormat};
use crate::hal::v4l2::stream::PlatformStream;
use crate::stream::ImageStream;
use crate::traits::Device as DeviceTrait;

pub struct PlatformDevice {
    inner: Device,
}

impl PlatformDevice {
    pub fn new(index: usize) -> io::Result<Self> {
        let dev = PlatformDevice {
            inner: Device::new(index)?,
        };
        Ok(dev)
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let dev = PlatformDevice {
            inner: Device::with_path(path)?,
        };
        Ok(dev)
    }

    pub fn inner(&self) -> &Device {
        &self.inner
    }
}

impl<'a> DeviceTrait<'a> for PlatformDevice {
    fn query_formats(&self) -> io::Result<Vec<ImageFormat>> {
        let mut formats = Vec::new();
        let plat_formats = self.inner.enum_formats()?;

        for format in plat_formats {
            for framesize in self.inner.enum_framesizes(format.fourcc)? {
                // TODO: consider stepwise formats
                if let v4l::framesize::FrameSizeEnum::Discrete(size) = framesize.size {
                    formats.push(ImageFormat::new(
                        size.width,
                        size.height,
                        PixelFormat::from(&format.fourcc.repr),
                    ));
                }
            }
        }

        Ok(formats)
    }

    fn query_controls(&self) -> io::Result<Vec<control::Control>> {
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
                    repr = control::Representation::Integer {
                        range: (control.minimum as i64, control.maximum as i64),
                        step: control.step as u64,
                        default: control.default as i64,
                    };
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

            let mut flags = control::Flags::NONE;
            if control.flags & v4l::control::Flags::READ_ONLY == v4l::control::Flags::READ_ONLY {
                flags |= control::Flags::READ_ONLY;
            }
            if control.flags & v4l::control::Flags::WRITE_ONLY == v4l::control::Flags::WRITE_ONLY {
                flags |= control::Flags::WRITE_ONLY;
            }
            if control.flags & v4l::control::Flags::GRABBED == v4l::control::Flags::GRABBED {
                flags |= control::Flags::GRABBED;
            }
            if control.flags & v4l::control::Flags::INACTIVE == v4l::control::Flags::INACTIVE {
                flags |= control::Flags::INACTIVE;
            }

            controls.push(control::Control {
                id: control.id,
                name: control.name,
                repr,
                flags,
            })
        }

        Ok(controls)
    }

    fn control(&self, id: u32) -> io::Result<control::Value> {
        let ctrl = self.inner.control(id)?;
        match ctrl {
            Control::Value(val) => Ok(control::Value::Integer(val as i64)),
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

    fn format(&self) -> io::Result<ImageFormat> {
        let fmt = self.inner.format()?;
        Ok(
            ImageFormat::new(fmt.width, fmt.height, PixelFormat::from(&fmt.fourcc.repr))
                .stride(fmt.stride as usize),
        )
    }

    fn set_format(&mut self, fmt: &ImageFormat) -> io::Result<()> {
        let fourcc: &[u8; 4] = if let Ok(fourcc) = fmt.pixfmt.clone().try_into() {
            fourcc
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to map pixelformat to fourcc",
            ));
        };

        let fmt = CaptureFormat::new(fmt.width, fmt.height, FourCC_::new(fourcc));
        self.inner.set_format(&fmt)?;
        Ok(())
    }

    fn stream(&self) -> io::Result<ImageStream<'a>> {
        let format = self.format()?;
        let stream = PlatformStream::new(self)?;
        Ok(ImageStream::new(Box::new(stream), format))
    }
}
