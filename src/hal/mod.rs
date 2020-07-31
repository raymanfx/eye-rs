//! Hardware Abstraction Layer (HAL)
//!
//! Multiple backends can be implemented for a given platform.

pub mod traits;

pub mod device;
pub use device::Factory as DeviceFactory;

pub mod common;

#[cfg(feature = "v4l")]
pub(crate) mod v4l2;
