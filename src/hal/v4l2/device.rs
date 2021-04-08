use std::{convert::TryInto, io, path::Path, time::Duration};

use v4l::control::{Control, MenuItem as ControlMenuItem, Type as ControlType};
use v4l::video::Capture;
use v4l::Device as CaptureDevice;
use v4l::Format as CaptureFormat;
use v4l::FourCC as FourCC_;

use crate::control;
use crate::format::PixelFormat;
use crate::hal::v4l2::stream::Handle as StreamHandle;
use crate::hal::Stream;
use crate::stream::{Descriptor as StreamDescriptor, Flags as StreamFlags};
use crate::traits::Device;

pub struct Handle {
    inner: CaptureDevice,
}

impl Handle {
    pub fn new(index: usize) -> io::Result<Self> {
        let dev = Handle {
            inner: CaptureDevice::new(index)?,
        };
        Ok(dev)
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let dev = Handle {
            inner: CaptureDevice::with_path(path)?,
        };
        Ok(dev)
    }

    pub fn inner(&self) -> &CaptureDevice {
        &self.inner
    }
}

impl<'a> Device<'a> for Handle {
    fn query_streams(&self) -> io::Result<Vec<StreamDescriptor>> {
        let mut streams = Vec::new();
        let plat_formats = self.inner.enum_formats()?;

        for format in plat_formats {
            for framesize in self.inner.enum_framesizes(format.fourcc)? {
                // TODO: consider stepwise formats
                if let v4l::framesize::FrameSizeEnum::Discrete(size) = framesize.size {
                    for frameinterval in
                        self.inner
                            .enum_frameintervals(format.fourcc, size.width, size.height)?
                    {
                        // TODO: consider stepwise intervals
                        if let v4l::frameinterval::FrameIntervalEnum::Discrete(fraction) =
                            frameinterval.interval
                        {
                            streams.push(StreamDescriptor {
                                width: size.width,
                                height: size.height,
                                pixfmt: PixelFormat::from(&format.fourcc.repr),
                                interval: Duration::from_secs_f64(
                                    fraction.numerator as f64 / fraction.denominator as f64,
                                ),
                                flags: StreamFlags::NONE,
                            });
                        }
                    }
                }
            }
        }

        Ok(streams)
    }

    fn query_controls(&self) -> io::Result<Vec<control::Descriptor>> {
        let mut controls = Vec::new();
        let plat_controls = self.inner.query_controls()?;

        for control in plat_controls {
            // The v4l docs say applications should ignore permanently disabled controls.
            if control.flags & v4l::control::Flags::DISABLED == v4l::control::Flags::DISABLED {
                continue;
            }

            let state_type = match control.typ {
                ControlType::Integer | ControlType::Integer64 => control::Type::Number {
                    range: (control.minimum as f64, control.maximum as f64),
                    step: control.step as f32,
                },
                ControlType::Boolean => control::Type::Boolean,
                ControlType::Menu => {
                    let mut items = Vec::new();
                    if let Some(plat_items) = control.items {
                        for plat_item in plat_items {
                            match plat_item.1 {
                                ControlMenuItem::Name(name) => {
                                    items.push(control::MenuItem::String(name));
                                }
                                ControlMenuItem::Value(value) => {
                                    items.push(control::MenuItem::Number(value as f64));
                                }
                            }
                        }
                    }
                    control::Type::Menu(items)
                }
                ControlType::Button => control::Type::Stateless,
                ControlType::String => control::Type::String,
                ControlType::Bitmask => control::Type::Bitmask,
                _ => continue,
            };

            // assume controls to be readable and writable by default
            let mut flags = control::Flags::READ | control::Flags::WRITE;

            if control.flags & v4l::control::Flags::READ_ONLY == v4l::control::Flags::READ_ONLY {
                flags.remove(control::Flags::WRITE);
                flags.insert(control::Flags::READ);
            }
            if control.flags & v4l::control::Flags::WRITE_ONLY == v4l::control::Flags::WRITE_ONLY {
                flags.remove(control::Flags::READ);
                flags.insert(control::Flags::WRITE);
            }
            if control.flags & v4l::control::Flags::GRABBED == v4l::control::Flags::GRABBED {
                flags.remove(control::Flags::WRITE);
            }
            if control.flags & v4l::control::Flags::INACTIVE == v4l::control::Flags::INACTIVE {
                flags.remove(control::Flags::READ);
                flags.remove(control::Flags::WRITE);
            }

            controls.push(control::Descriptor {
                id: control.id,
                name: control.name,
                typ: state_type,
                flags,
            })
        }

        Ok(controls)
    }

    fn read_control(&self, id: u32) -> io::Result<control::State> {
        let ctrl = self.inner.control(id)?;
        match ctrl {
            Control::Value(val) => Ok(control::State::Number(val as f64)),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "control type cannot be mapped",
            )),
        }
    }

    fn write_control(&mut self, id: u32, val: &control::State) -> io::Result<()> {
        match val {
            control::State::Number(val) => {
                let ctrl = Control::Value(*val as i32);
                self.inner.set_control(id, ctrl)?;
            }
            control::State::Boolean(val) => {
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

    fn start_stream(&self, desc: &StreamDescriptor) -> io::Result<Stream<'a>> {
        let fourcc = if let Ok(fourcc) = desc.pixfmt.clone().try_into() {
            fourcc
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to map pixelformat to fourcc",
            ));
        };
        // configure frame format
        let format = CaptureFormat::new(desc.width, desc.height, FourCC_::new(&fourcc));
        self.inner.set_format(&format)?;

        // configure frame timing
        let fps = (1.0 / desc.interval.as_secs_f32()) as u32;
        let mut params = self.inner.params()?;
        params.interval = v4l::Fraction::new(1, fps);
        self.inner.set_params(&params)?;

        let handle = StreamHandle::new(self)?;
        Ok(Stream::V4l2(handle))
    }
}
