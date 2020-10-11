use std::io;

use crate::hal::common::device::TransparentDevice;
use crate::hal::traits::Device;

/// Platform device factory
///
/// Automatically selects a suitable backend.
pub struct Factory {}

impl Factory {
    #[allow(unreachable_code)]
    /// Returns a list of available devices
    pub fn enumerate() -> Vec<String> {
        let mut list = Vec::new();

        #[cfg(feature = "v4l")]
        {
            let _list: Vec<String> = crate::hal::v4l2::devices()
                .into_iter()
                .map(|uri| format!("v4l://{}", uri))
                .collect();
            list.extend(_list);
        }

        list
    }

    #[allow(unreachable_code)]
    /// Returns a new platform device abstraction
    pub fn create(_uri: &str) -> io::Result<Box<dyn Device>> {
        #[cfg(feature = "v4l")]
        if _uri.starts_with("v4l://") {
            let path = _uri[6..].to_string();
            let dev = crate::hal::v4l2::device::PlatformDevice::with_path(path)?;
            let dev = TransparentDevice::new(Box::new(dev));
            return Ok(Box::new(dev));
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "No suitable backend available",
        ))
    }
}
