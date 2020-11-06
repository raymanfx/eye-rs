use std::io;

use crate::hal::common::device::TransparentDevice;
use crate::traits::Device;

/// Runtime context
pub struct Context {}

impl Context {
    /// Returns a list of available devices
    pub fn enumerate_devices() -> Vec<String> {
        let mut list = Vec::new();

        #[cfg(target_os = "linux")]
        {
            let _list: Vec<String> = crate::hal::v4l2::devices()
                .into_iter()
                .map(|uri| format!("v4l://{}", uri))
                .collect();
            list.extend(_list);
        }

        #[cfg(feature = "hal-openpnp")]
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
    pub fn open_device<S: AsRef<str>>(_uri: S) -> io::Result<Box<dyn Device + Send>> {
        let _uri = _uri.as_ref();

        #[cfg(target_os = "linux")]
        if _uri.starts_with("v4l://") {
            let path = _uri[6..].to_string();
            let dev = crate::hal::v4l2::device::PlatformDevice::with_path(path)?;
            let dev = TransparentDevice::new(Box::new(dev));
            return Ok(Box::new(dev));
        }

        #[cfg(feature = "hal-openpnp")]
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
