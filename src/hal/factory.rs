use std::io;

use crate::hal::traits::Device;

/// Platform device factory
///
/// Automatically selects a suitable backend.
pub struct DeviceFactory {}

impl DeviceFactory {
    #[allow(unreachable_code)]
    /// Returns a new platform device abstraction
    pub fn create(_index: usize) -> io::Result<Box<dyn Device>> {
        #[cfg(feature = "v4l")]
        {
            let dev = crate::hal::v4l2::device::PlatformDevice::new(_index)?;
            return Ok(Box::new(dev));
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "No suitable backend available",
        ))
    }
}
