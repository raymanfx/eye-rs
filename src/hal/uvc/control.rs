use std::io;

use crate::control as EyeControl;

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

    pub fn get(&self, handle: &uvc::DeviceHandle) -> io::Result<EyeControl::Value> {
        match self {
            Control::ScanningMode => match handle.scanning_mode() {
                Ok(mode) => Ok(EyeControl::Value::String(format!("{:?}", mode))),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::AutoExposureMode => match handle.ae_mode() {
                Ok(mode) => Ok(EyeControl::Value::String(format!("{:?}", mode))),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::AutoExposurePriority => match handle.ae_priority() {
                Ok(mode) => Ok(EyeControl::Value::String(format!("{:?}", mode))),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::ExposureAbsolute => match handle.exposure_abs() {
                Ok(val) => Ok(EyeControl::Value::Integer(val as i64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::ExposureRelative => match handle.exposure_rel() {
                Ok(val) => Ok(EyeControl::Value::Integer(val as i64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::FocusAbsolute => match handle.focus_abs() {
                Ok(val) => Ok(EyeControl::Value::Integer(val as i64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
            Control::FocusRelative => match handle.focus_rel() {
                Ok((val, _speed)) => Ok(EyeControl::Value::Integer(val as i64)),
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
            },
        }
    }
}

impl From<&Control> for EyeControl::Control {
    fn from(ctrl: &Control) -> Self {
        match ctrl {
            Control::ScanningMode => EyeControl::Control {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: EyeControl::Flags::READ_ONLY,
                repr: EyeControl::Representation::Menu(vec![
                    EyeControl::MenuItem::String(String::from("Interlaced")),
                    EyeControl::MenuItem::String(String::from("Progressive")),
                ]),
            },
            Control::AutoExposureMode => EyeControl::Control {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: EyeControl::Flags::READ_ONLY,
                repr: EyeControl::Representation::Menu(vec![
                    EyeControl::MenuItem::String(String::from("Manual")),
                    EyeControl::MenuItem::String(String::from("Auto")),
                    EyeControl::MenuItem::String(String::from("ShutterPriority")),
                    EyeControl::MenuItem::String(String::from("AperturePriority")),
                ]),
            },
            Control::AutoExposurePriority => EyeControl::Control {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: EyeControl::Flags::READ_ONLY,
                repr: EyeControl::Representation::Menu(vec![
                    EyeControl::MenuItem::String(String::from("Constant")),
                    EyeControl::MenuItem::String(String::from("Variable")),
                ]),
            },
            Control::ExposureAbsolute => EyeControl::Control {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: EyeControl::Flags::READ_ONLY,
                repr: EyeControl::Representation::Integer {
                    range: (u32::MIN as i64, u32::MAX as i64),
                    step: 1,
                    default: 0,
                },
            },
            Control::ExposureRelative => EyeControl::Control {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: EyeControl::Flags::READ_ONLY,
                repr: EyeControl::Representation::Integer {
                    range: (i8::MIN as i64, i8::MAX as i64),
                    step: 1,
                    default: 0,
                },
            },
            Control::FocusAbsolute => EyeControl::Control {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: EyeControl::Flags::READ_ONLY,
                repr: EyeControl::Representation::Integer {
                    range: (u16::MIN as i64, u16::MAX as i64),
                    step: 1,
                    default: 0,
                },
            },
            Control::FocusRelative => EyeControl::Control {
                id: ctrl.id(),
                name: String::from(ctrl.name()),
                flags: EyeControl::Flags::READ_ONLY,
                repr: EyeControl::Representation::Integer {
                    range: (i8::MIN as i64, i8::MAX as i64),
                    step: 1,
                    default: 0,
                },
            },
        }
    }
}
