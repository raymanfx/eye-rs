//! Hardware Abstraction Layer (HAL)
//!
//! Multiple backends can be implemented for a given platform.

pub mod common;
pub mod traits;

#[cfg(target_os = "linux")]
pub(crate) mod v4l2;

#[cfg(feature = "hal-openpnp")]
pub(crate) mod openpnp;
