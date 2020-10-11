use std::io;

use crate::hal::common::device::TransparentDevice;
use crate::hal::traits::Device as DeviceTrait;

/// Capture device
///
/// This struct is used to create the actual platform-specific capture device instances. All
/// structs returned implement the Device HAL trait, allowing for device manipulation and image
/// streaming.
pub struct Device {}

impl Device {
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

        #[cfg(feature = "openpnp")]
        {
            let _list: Vec<String> = crate::hal::openpnp::devices()
                .into_iter()
                .map(|uri| format!("pnp://{}", uri))
                .collect();
            list.extend(_list);
        }

        list
    }

    /// Returns a new platform device abstraction
    pub fn with_uri(_uri: &str) -> io::Result<Box<dyn DeviceTrait>> {
        #[cfg(feature = "v4l")]
        if _uri.starts_with("v4l://") {
            let path = _uri[6..].to_string();
            let dev = crate::hal::v4l2::device::PlatformDevice::with_path(path)?;
            let dev = TransparentDevice::new(Box::new(dev));
            return Ok(Box::new(dev));
        }

        #[cfg(feature = "openpnp")]
        if _uri.starts_with("pnp://") {
            let index = _uri[6..].to_string();
            let index = match index.parse::<u32>() {
                Ok(index) => index,
                Err(_) => return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid URI")),
            };
            let dev = match crate::hal::openpnp::device::PlatformDevice::new(index) {
                Some(dev) => dev,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "failed to open device",
                    ))
                }
            };
            let dev = TransparentDevice::new(Box::new(dev));
            return Ok(Box::new(dev));
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "No suitable backend available",
        ))
    }
}
