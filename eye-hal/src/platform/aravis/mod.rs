//! Aravis (GenICam / USB3 Vision / GigE Vision) backend
//!
//! Handles industrial cameras (e.g. Daheng, Basler, FLIR) that speak the GenICam standard
//! over a USB3 Vision or GigE Vision transport instead of USB Video Class. The V4L2/UVC/
//! openpnp backends can never see these devices: they expose no UVC interface at all, so
//! there is no /dev/videoN node (Linux) or AVFoundation/DirectShow entry (macOS/Windows)
//! for those backends to find, regardless of host OS.
//!
//! # Related Links
//! * <https://github.com/AravisProject/aravis> - Aravis GenICam library

pub mod context;
pub mod device;
pub mod stream;

use std::ffi::CStr;

use crate::error::{Error, ErrorKind, Result};

/// Checks a GError out-parameter as populated by an arv_* call, freeing it if set.
///
/// # Safety
/// `err` must be a valid, fully-initialized GError pointer (null or otherwise) as written by
/// an aravis-sys FFI call.
pub(crate) unsafe fn check_error(err: *mut glib_sys::GError) -> Result<()> {
    if err.is_null() {
        Ok(())
    } else {
        let msg = CStr::from_ptr((*err).message)
            .to_string_lossy()
            .into_owned();
        glib_sys::g_error_free(err);
        Err(Error::new(ErrorKind::Other, msg))
    }
}
