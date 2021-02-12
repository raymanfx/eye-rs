use v4l::context;

use crate::hal::v4l2::device::Handle;
use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl ContextTrait for Context {
    fn enumerate_devices() -> Vec<String> {
        context::enum_devices()
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

                Some(format!("/dev/video{}", index))
            })
            .collect()
    }
}
