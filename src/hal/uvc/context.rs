use std::io;

use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl ContextTrait for Context {
    fn query_devices(&self) -> io::Result<Vec<String>> {
        let ctx = match uvc::Context::new() {
            Ok(ctx) => ctx,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };

        let devices = match ctx.devices() {
            Ok(devices) => devices
                .into_iter()
                .map(|dev| format!("{}:{}", dev.bus_number(), dev.device_address()))
                .collect(),
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };

        Ok(devices)
    }
}
