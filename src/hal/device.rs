use std::io;

use crate::device::Info;
use crate::hal::traits::Device;

use crate::hal::common::device::TransparentDevice;

pub struct List {}

impl List {
    #[allow(unreachable_code)]
    /// Returns a list of device info structs
    pub fn enumerate() -> Vec<Info> {
        #[cfg(feature = "v4l")]
        {
            let list = crate::hal::v4l2::device::PlatformList::enumerate();
            return list;
        }

        Vec::new()
    }
}

/// Platform device factory
///
/// Automatically selects a suitable backend.
pub struct Factory {}

impl Factory {
    #[allow(unreachable_code)]
    /// Returns a new platform device abstraction
    pub fn create(_index: usize) -> io::Result<Box<dyn Device>> {
        #[cfg(feature = "v4l")]
        {
            let dev = crate::hal::v4l2::device::PlatformDevice::new(_index)?;
            let dev = TransparentDevice::new(Box::new(dev));
            return Ok(Box::new(dev));
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "No suitable backend available",
        ))
    }
}
