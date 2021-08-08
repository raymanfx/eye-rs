use crate::device;
use crate::error::{Error, ErrorKind, Result};
use crate::platform::uvc::device::Handle as DeviceHandle;
use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl<'a> ContextTrait<'a> for Context {
    type Device = DeviceHandle<'a>;

    fn devices(&self) -> Result<Vec<device::Description>> {
        let ctx = match uvc::Context::new() {
            Ok(ctx) => ctx,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        };

        let devices = match ctx.devices() {
            Ok(devices) => devices
                .into_iter()
                .map(|dev| {
                    let mut description = device::Description {
                        uri: format!("uvc://{}:{}", dev.bus_number(), dev.device_address()),
                        product: "Unknown UVC device".to_string(),
                    };

                    if let Ok(desc) = dev.description() {
                        if let Some(product) = desc.product {
                            description.product = product;
                        }
                    }

                    // If the parsing of hardware information (e.g. product name) failed, there's
                    // a very high chance that the device cannot be opened successfully. This could
                    // be due to insufficient permissions or other issues.
                    //
                    // We present the device to consumers of this crate anyways and require them to
                    // deal with it when they call ctx.open_device(..).
                    description
                })
                .collect(),
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        };

        Ok(devices)
    }

    fn open_device(&self, uri: &str) -> Result<Self::Device> {
        if uri.starts_with("uvc://") {
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
