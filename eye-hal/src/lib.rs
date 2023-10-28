//! Eye is a cross platform camera capture and control library.
//!
//! Things like zero-copy buffer capture and transparent pixel format conversion are hallmark
//! features of Eye which distinguish it from other camera capture crates.
//!
//! Additional documentation can currently also be found in the
//! [README.md file which is most easily viewed on github](https://github.com/raymanfx/eye-rs/blob/master/README.md).
//!
//! [Jump forward to crate content](#reexports)
//!
//! # Overview
//!
//! Devices are created from platform specific indices. On Linux for example, the index 0 maps to
//! the node present at /dev/video0 which is usually the first device seen by the kernel.
//!
//! Device instances can be used to perform various management and control tasks such as modifying
//! the capture format, altering hardware parameters and more. For example, you would use the
//! device API to set the exposure level or enable the autofocus mode of a connected camera.
//!
//! When it comes to actually capturing frames from the camera, you use the stream API instead.
//! All devices offer a stream() call which returns the 'native' buffer stream, i.e. no copying is
//! involved in userspace unless transparent format conversion is explicitly requested through the
//! device API.
//! Streams should abstract the native platform capture concepts and automatically choose the best
//! way to grab frames from the camera as exposed by the OS drivers. For example, Linux platforms
//! will almost always use mapped buffers or user mode buffers instead of just reading bytes into
//! a userspace buffer which involves expensive copying in the driver.
//!
//! The common user of this crate will mainly be interested in frame capturing.
//! Here is a very brief example of streaming I/O:
//!
//! ```no_run
//! use eye_hal::PlatformContext;
//! use eye_hal::traits::{Context, Device, Stream};
//!
//! // Create a context
//! let ctx = PlatformContext::default();
//!
//! // Query for available devices.
//! let devices = ctx.devices().expect("Failed to query devices");
//! if devices.is_empty() {
//!     panic!("No devices available");
//! }
//!
//! // First, we need a capture device to read images from. For this example, let's just choose
//! // whatever device is first in the list.
//! let dev = ctx.open_device(&devices[0].uri).expect("Failed to open video device");
//!
//! // Query for available streams and just choose the first one.
//! let streams = dev.streams().expect("Failed to query streams");
//! let stream_desc = streams[0].clone();
//! println!("Stream: {:?}", stream_desc);
//!
//! // Since we want to capture images, we need to access the native image stream of the device.
//! // The backend will internally select a suitable implementation for the platform stream. On
//! // Linux for example, most devices support memory-mapped buffers.
//! let mut stream = dev.start_stream(&stream_desc).expect("Failed to setup capture stream");
//!
//! // Now we are all set to start capturing some frames!
//! let _buf = stream
//!     .next()
//!     .expect("Stream is dead")
//!     .expect("Failed to capture buffer");
//!```

//!
//! Have a look at the examples to learn more about device and stream management.

pub mod control;
pub mod device;
pub mod error;
pub mod format;
pub mod stream;
pub mod traits;

pub mod platform;

pub use error::{Error, ErrorKind, Result};
pub use platform::Context as PlatformContext;
