//! Eye is a cross platform camera capture and control library.
//!
//! This crate provides a high-level API on top of the low-level parts like `eye-hal`.
//!
//! Additional documentation can currently also be found in the
//! [README.md file which is most easily viewed on github](https://github.com/raymanfx/eye-rs/blob/master/README.md).
//!
//! [Jump forward to crate content](#reexports)
//!
//! # Overview
//!
//! The device abstraction provided in this crate builds on top of a PlatformDevice acquired
//! through the `Context` struct of the `eye-hal` subcrate. It performs transparent frame format
//! conversion (e.g. JPEG -> RGB decoding) by leveraging the `colorconvert` module.

pub mod colorconvert;
pub mod device;
pub mod stream;

pub use eye_hal as hal;
