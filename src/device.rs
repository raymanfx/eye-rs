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

    /// The actual control representation
    pub repr: control::Representation,
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
