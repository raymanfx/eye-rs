use std::io;

use crate::control;

pub(crate) enum Control {
    ScanningMode,
    AutoExposureMode,
    AutoExposurePriority,
    ExposureAbsolute,
    ExposureRelative,
    FocusAbsolute,
    FocusRelative,
}

impl Control {
    pub fn all() -> impl IntoIterator<Item = Control> {
        vec![
            Control::ScanningMode,
            Control::AutoExposureMode,
            Control::AutoExposurePriority,
            Control::ExposureAbsolute,
            Control::ExposureRelative,
            Control::FocusAbsolute,
            Control::FocusRelative,
        ]
    }

    pub fn from_id(id: u32) -> Option<Self> {
        for ctrl in Control::all() {
            if ctrl.id() == id {
                return Some(ctrl);
            }
        }

        None
    }

    pub fn id(&self) -> u32 {
        match self {
            Control::ScanningMode => 1,
            Control::AutoExposureMode => 2,
            Control::AutoExposurePriority => 3,
            Control::ExposureAbsolute => 4,
            Control::ExposureRelative => 5,
            Control::FocusAbsolute => 6,
            Control::FocusRelative => 7,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Control::ScanningMode => "Scanning Mode",
            Control::AutoExposureMode => "Auto Exposure Mode",
            Control::AutoExposurePriority => "Auto Exposure Priority",
            Control::ExposureAbsolute => "Exposure (Absolute)",
            Control::ExposureRelative => "Exposure (Relative)",
            Control::FocusAbsolute => "Focus (Absolute)",
            Control::FocusRelative => "Focus (Relative)",
        }
    }

    pub fn get(&self, handle: &uvc::DeviceHandle) -> io::Result<control::State> {
        match self {
            Control::ScanningMode => match handle.scanning_mode() {
                Ok(mode) => Ok(control::State::String(format!("{:?}", mode))),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::AutoExposureMode => match handle.ae_mode() {
                Ok(mode) => Ok(control::State::String(format!("{:?}", mode))),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::AutoExposurePriority => match handle.ae_priority() {
                Ok(mode) => Ok(control::State::String(format!("{:?}", mode))),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::ExposureAbsolute => match handle.exposure_abs() {
                Ok(val) => Ok(control::State::Number(val as f64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::ExposureRelative => match handle.exposure_rel() {
                Ok(val) => Ok(control::State::Number(val as f64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::FocusAbsolute => match handle.focus_abs() {
                Ok(val) => Ok(control::State::Number(val as f64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::FocusRelative => match handle.focus_rel() {
                Ok((val, _speed)) => Ok(control::State::Number(val as f64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
        }
    }
}

impl From<&Control> for control::Descriptor {
    fn from(ctrl: &Control) -> Self {
        match ctrl {
            Control::ScanningMode => control::Descriptor {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: control::Flags::READ,
                typ: control::Type::Menu(vec![
                    control::MenuItem::String(String::from("Interlaced")),
                    control::MenuItem::String(String::from("Progressive")),
                ]),
            },
            Control::AutoExposureMode => control::Descriptor {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: control::Flags::READ,
                typ: control::Type::Menu(vec![
                    control::MenuItem::String(String::from("Manual")),
                    control::MenuItem::String(String::from("Auto")),
                    control::MenuItem::String(String::from("ShutterPriority")),
                    control::MenuItem::String(String::from("AperturePriority")),
                ]),
            },
            Control::AutoExposurePriority => control::Descriptor {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: control::Flags::READ,
                typ: control::Type::Menu(vec![
                    control::MenuItem::String(String::from("Constant")),
                    control::MenuItem::String(String::from("Variable")),
                ]),
            },
            Control::ExposureAbsolute => control::Descriptor {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: control::Flags::READ,
                typ: control::Type::Number {
                    range: (u32::MIN as f64, u32::MAX as f64),
                    step: 1.0,
                },
            },
            Control::ExposureRelative => control::Descriptor {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: control::Flags::READ,
                typ: control::Type::Number {
                    range: (i8::MIN as f64, i8::MAX as f64),
                    step: 1.0,
                },
            },
            Control::FocusAbsolute => control::Descriptor {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: control::Flags::READ,
                typ: control::Type::Number {
                    range: (u16::MIN as f64, u16::MAX as f64),
                    step: 1.0,
                },
            },
            Control::FocusRelative => control::Descriptor {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: control::Flags::READ,
                typ: control::Type::Number {
                    range: (i8::MIN as f64, i8::MAX as f64),
                    step: 1.0,
                },
            },
        }
    }
}
