use std::io;

use ffimage::packed::DynamicImageView;

use crate::control::Value as ControlValue;
use crate::format::Format;
use crate::traits::Stream;

/// Platform device abstraction
pub trait Device {
    /// Returns the current control value for an ID
    fn get_control(&mut self, id: u32) -> io::Result<ControlValue>;

    /// Sets the control value, returns error for incompatible value types
    fn set_control(&mut self, id: u32, val: &ControlValue) -> io::Result<()>;

    /// Returns the current format in use by the device
    fn get_format(&mut self) -> io::Result<Format>;

    /// Attempts to match the requested format to a device format on a best-effort basis and
    /// returns the actual format in use
    fn set_format(&mut self, fmt: &Format) -> io::Result<Format>;

    /// Returns a zero-copy stream for direct frame access
    fn stream<'a>(&'a mut self) -> io::Result<Box<dyn Stream<Item = DynamicImageView> + 'a>>;
}
