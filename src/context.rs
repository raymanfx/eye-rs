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
            ctx.query_devices()?
                .into_iter()
                .for_each(|uri| list.push(uri));
        }

        #[cfg(feature = "hal-uvc")]
        {
            let ctx = crate::hal::uvc::Context {};
            ctx.query_devices()?
                .into_iter()
                .for_each(|uri| list.push(uri));
        }

        Ok(list)
    }
}
