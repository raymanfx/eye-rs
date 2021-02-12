use crate::traits::Context as _;

/// Runtime context
pub struct Context {}

impl Context {
    /// Returns a list of available devices
    pub fn enumerate_devices() -> Vec<String> {
        let mut list = Vec::new();

        #[cfg(target_os = "linux")]
        {
            let _list: Vec<String> = crate::hal::v4l2::context::Context::enumerate_devices()
                .into_iter()
                .map(|uri| format!("v4l://{}", uri))
                .collect();
            list.extend(_list);
        }

        #[cfg(feature = "hal-uvc")]
        {
            let _list: Vec<String> = crate::hal::uvc::context::Context::enumerate_devices()
                .into_iter()
                .map(|uri| format!("uvc://{}", uri))
                .collect();
            list.extend(_list);
        }

        list
    }
}
