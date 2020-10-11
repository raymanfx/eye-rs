//! Hardware Abstraction Layer (HAL)
//!
//! Multiple backends can be implemented for a given platform.

pub mod common;
pub mod traits;

#[cfg(feature = "v4l")]
pub(crate) mod v4l2;

#[cfg(feature = "openpnp")]
pub(crate) mod openpnp;
