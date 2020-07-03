//! Video for Linux (2) backend
//!
//! V4L2 is the standard API for video input and output on Linux.
//!
//! # Related Links
//! * <https://linuxtv.org/downloads/v4l-dvb-apis-new/userspace-api/v4l/v4l2.html> - Video for Linux API
//!

pub mod traits;

pub mod device;
pub use device::{Factory as DeviceFactory, Info as DeviceInfo, List as DeviceList};

#[cfg(feature = "v4l")]
pub mod v4l2;
