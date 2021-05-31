use crate::error::{Error, ErrorKind, Result};
use crate::platform::Device as PlatformDevice;
use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl ContextTrait for Context {
    fn devices(&self) -> Result<Vec<String>> {
        let ctx = match uvc::Context::new() {
            Ok(ctx) => ctx,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        };

        let devices = match ctx.devices() {
            Ok(devices) => devices
                .into_iter()
                .map(|dev| format!("uvc://{}:{}", dev.bus_number(), dev.device_address()))
                .collect(),
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        };

        Ok(devices)
    }

    fn open_device<'a>(&self, uri: &str) -> Result<PlatformDevice<'a>> {
        if uri.starts_with("uvc://") {
            let handle = match crate::platform::uvc::device::Handle::with_uri(uri) {
                Ok(handle) => handle,
                Err(e) => return Err(Error::new(ErrorKind::Other, format!("UVC: {}", e))),
            };
            Ok(PlatformDevice::Uvc(handle))
        } else {
            Err(Error::new(ErrorKind::Other, "invalid URI"))
        }
    }
}
