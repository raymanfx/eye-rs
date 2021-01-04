//! Hardware Abstraction Layer (HAL)
//!
//! Multiple backends can be implemented for a given platform.

#[cfg(target_os = "linux")]
pub(crate) mod v4l2;

#[cfg(feature = "hal-uvc")]
pub(crate) mod uvc;
