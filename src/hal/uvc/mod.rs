//! UVC (universal video class) backend
//!
//! # Related Links
//! * <https://de.wikipedia.org/wiki/USB_Video_Class> - UVC Wikipedia page

pub mod device;
pub mod stream;

pub fn devices() -> Vec<String> {
    let ctx = if let Ok(ctx) = uvc::Context::new() {
        ctx
    } else {
        return Vec::new();
    };

    let devices = if let Ok(devices) = ctx.devices() {
        devices
            .into_iter()
            .map(|dev| format!("{}:{}", dev.bus_number(), dev.device_address()))
            .collect()
    } else {
        Vec::new()
    };

    devices
}
