use bitflags::bitflags;

use crate::control;
use crate::format::PixelFormat;

#[derive(Clone)]
/// Image buffer format description
pub struct FormatInfo {
    /// PixelFormat
    pub pixfmt: PixelFormat,
    /// Length of a pixel row in bytes
    pub resolutions: Vec<(u32, u32)>,

    /// Whether this format is emulated or natively supported
    pub emulated: bool,
}

#[derive(Clone)]
/// Device control description
pub struct ControlInfo {
    /// Implementation specific ID (unique)
    pub id: u32,
    /// Name of the control
    pub name: String,

    /// State flags
    pub flags: ControlFlags,

    /// The actual control representation
    pub repr: control::Representation,
}

impl ControlInfo {
    /// Returns true if the control value can be read
    pub fn readable(&self) -> bool {
        !(self.flags & ControlFlags::WRITE_ONLY == ControlFlags::WRITE_ONLY
            || self.flags & ControlFlags::INACTIVE == ControlFlags::INACTIVE)
    }

    /// Returns true if the control value can be written
    pub fn writable(&self) -> bool {
        !(self.flags & ControlFlags::READ_ONLY == ControlFlags::READ_ONLY
            || self.flags & ControlFlags::GRABBED == ControlFlags::GRABBED)
    }
}

bitflags! {
    /// Control state flags
    pub struct ControlFlags: u32 {
        /// No flags are set
        const NONE                  = 0x000;
        /// Permanently read-only
        const READ_ONLY             = 0x001;
        /// Permanently write-only
        const WRITE_ONLY            = 0x002;
        /// Grabbed by another process, temporarily read-only
        const GRABBED               = 0x004;
        /// Not applicable in the current context
        const INACTIVE              = 0x008;
    }
}

#[derive(Clone)]
/// Platform device info
///
/// Only fields supported by all backends shall be added here.
pub struct Info {
    /// Index of the device (unique)
    pub index: u32,
    /// Name of the device (non unique)
    pub name: String,
}
