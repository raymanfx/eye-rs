use v4l::context;

use crate::device;
use crate::error::{Error, ErrorKind, Result};
use crate::platform::v4l2::device::Handle;
use crate::platform::Device as PlatformDevice;
use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl ContextTrait for Context {
    fn devices(&self) -> Result<Vec<device::Description>> {
        let nodes = context::enum_devices()
            .into_iter()
            .filter_map(|dev| {
                let index = dev.index();
                let dev = match Handle::new(index) {
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

    fn open_device<'a>(&self, uri: &str) -> Result<PlatformDevice<'a>> {
        if uri.starts_with("v4l://") {
            let handle = crate::platform::v4l2::device::Handle::with_uri(uri)?;
            Ok(PlatformDevice::V4l2(handle))
        } else {
            Err(Error::new(ErrorKind::Other, "invalid URI"))
        }
    }
}
