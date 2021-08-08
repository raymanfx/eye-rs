use v4l::context;

use crate::device;
use crate::error::{Error, ErrorKind, Result};
use crate::platform::v4l2::device::Handle as DeviceHandle;
use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl<'a> ContextTrait<'a> for Context {
    type Device = DeviceHandle;

    fn devices(&self) -> Result<Vec<device::Description>> {
        let nodes = context::enum_devices()
            .into_iter()
            .filter_map(|dev| {
                let index = dev.index();
                let dev = match DeviceHandle::new(index) {
                    Ok(dev) => dev,
                    Err(_) => return None,
                };

                let caps = match dev.inner().query_caps() {
                    Ok(caps) => caps,
                    Err(_) => return None,
                };

                // For now, require video capture and streaming capabilities.
                // Very old devices may only support the read() I/O mechanism, so support for those
                // might be added in the future. Every recent (released during the last ten to twenty
                // years) webcam should support streaming though.
                let capture_flag = v4l::capability::Flags::VIDEO_CAPTURE;
                let streaming_flag = v4l::capability::Flags::STREAMING;
                if caps.capabilities & capture_flag != capture_flag
                    || caps.capabilities & streaming_flag != streaming_flag
                {
                    return None;
                }

                Some(device::Description {
                    uri: format!("v4l:///dev/video{}", index),
                    product: caps.card,
                })
            })
            .collect();

        Ok(nodes)
    }

    fn open_device(&self, uri: &str) -> Result<Self::Device> {
        if uri.starts_with("v4l://") {
            let handle = crate::platform::v4l2::device::Handle::with_uri(uri)?;
            Ok(handle)
        } else {
            Err(Error::new(ErrorKind::Other, "invalid URI"))
        }
    }
}
