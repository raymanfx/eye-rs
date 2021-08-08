//! Video for Linux (2) backend
//!
//! V4L2 is the standard API for video input and output on Linux.
//!
//! # Related Links
//! * <https://linuxtv.org/downloads/v4l-dvb-apis-new/userspace-api/v4l/v4l2.html> - Video for Linux API

pub mod context;
pub mod device;
pub mod stream;

pub use context::Context;

use std::{convert::TryInto, str};

use crate::format::PixelFormat;

impl From<&[u8; 4]> for PixelFormat {
    fn from(fourcc: &[u8; 4]) -> Self {
        // We use the Linux fourccs as defined here:
        // https://www.kernel.org/doc/html/v5.5/media/uapi/v4l/videodev.html.

        // Mono (single-component) formats
        if fourcc == b"GREY" {
            PixelFormat::Gray(8)
        } else if fourcc == b"Y16 " {
            PixelFormat::Gray(16)
        } else if fourcc == b"Z16 " {
            PixelFormat::Depth(16)
        }
        // RGB formats
        else if fourcc == b"BGR3" {
            PixelFormat::Bgr(24)
        } else if fourcc == b"RGB3" {
            PixelFormat::Rgb(24)
        }
        // Compressed formats
        else if fourcc == b"MJPG" {
            PixelFormat::Jpeg
        }
        // Misc
        else {
            PixelFormat::Custom(String::from(str::from_utf8(fourcc).unwrap()))
        }
    }
}

impl TryInto<[u8; 4]> for PixelFormat {
    type Error = ();

    fn try_into(self) -> Result<[u8; 4], Self::Error> {
        // We use the Linux fourccs as defined here:
        // https://www.kernel.org/doc/html/v5.5/media/uapi/v4l/videodev.html.

        match self {
            PixelFormat::Custom(repr) => {
                // If the representation does not take up more than 4 four bytes, we can safely
                // convert it into a four character code.
                let repr_bytes = repr.as_bytes();
                if repr_bytes.len() <= 4 {
                    let mut bytes = [0u8; 4];
                    bytes.clone_from_slice(&repr_bytes[..repr_bytes.len()]);
                    Ok(bytes)
                } else {
                    Err(())
                }
            }
            PixelFormat::Gray(8) => Ok(*b"GREY"),
            PixelFormat::Gray(16) => Ok(*b"Y16 "),
            PixelFormat::Depth(16) => Ok(*b"Z16 "),
            PixelFormat::Bgr(24) => Ok(*b"BGR3"),
            PixelFormat::Rgb(24) => Ok(*b"RGB3"),
            PixelFormat::Rgb(32) => Ok(*b"AB24"),
            PixelFormat::Jpeg => Ok(*b"MJPG"),
            _ => Err(()),
        }
    }
}
