use openpnp_capture as pnp;

use crate::device;
use crate::error::{Error, ErrorKind, Result};
use crate::platform::openpnp::device::Handle as DeviceHandle;
use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl<'a> ContextTrait<'a> for Context {
    type Device = DeviceHandle;

    fn devices(&self) -> Result<Vec<device::Description>> {
        let devices = pnp::Device::enumerate()
            .into_iter()
            .map(|i| device::Description {
                uri: format!("pnp://{}", i),
                product: "Unknown OpenPnP device".to_string(),
            })
            .collect();

        Ok(devices)
    }

    fn open_device(&self, uri: &str) -> Result<Self::Device> {
        if uri.starts_with("pnp://") {
            let handle = match DeviceHandle::with_uri(uri) {
                Ok(handle) => handle,
                Err(e) => return Err(Error::new(ErrorKind::Other, format!("UVC: {}", e))),
            };
            Ok(handle)
        } else {
            Err(Error::new(ErrorKind::Other, "invalid URI"))
        }
    }
}
