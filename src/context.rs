use std::io;

use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl Context {
    pub fn new() -> Self {
        Context {}
    }
}

impl ContextTrait for Context {
    fn query_devices(&self) -> io::Result<Vec<String>> {
        let mut list = Vec::new();

        #[cfg(target_os = "linux")]
        {
            let ctx = crate::hal::v4l2::Context {};
            let _list: Vec<String> = ctx
                .query_devices()?
                .into_iter()
                .map(|uri| format!("v4l://{}", uri))
                .collect();
            list.extend(_list);
        }

        #[cfg(feature = "hal-uvc")]
        {
            let ctx = crate::hal::uvc::Context {};
            let _list: Vec<String> = ctx
                .query_devices()?
                .into_iter()
                .map(|uri| format!("uvc://{}", uri))
                .collect();
            list.extend(_list);
        }

        Ok(list)
    }
}
