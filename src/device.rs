use crate::format::FourCC;

/// Image buffer format description
pub struct FormatInfo {
    /// Pixelformat
    pub fourcc: FourCC,
    /// Length of a pixel row in bytes
    pub resolutions: Vec<(u32, u32)>,

    /// Whether this format is emulated or natively supported
    pub emulated: bool,
}

/// Platform device info
///
/// Only fields supported by all backends shall be added here.
pub struct Info {
    /// Index of the device (unique)
    pub index: u32,
    /// Name of the device (non unique)
    pub name: String,

    /// Formats supported by the device
    pub formats: Vec<FormatInfo>,
}
