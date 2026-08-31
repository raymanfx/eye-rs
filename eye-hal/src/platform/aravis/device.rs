use std::ffi::CString;
use std::ptr;

use crate::control;
use crate::error::{Error, ErrorKind, Result};
use crate::format::PixelFormat;
use crate::platform::aravis::check_error;
use crate::platform::aravis::stream::Handle as StreamHandle;
use crate::stream;
use crate::traits::Device;

/// Advertised when the camera does not report an acquisition frame rate of its own.
const DEFAULT_FRAME_RATE: f64 = 30.0;

pub struct Handle {
    pub(crate) camera: *mut aravis_sys::ArvCamera,
}

// ArvCamera is a plain GObject: refcounting is thread-safe, and we never call methods on it
// concurrently from multiple threads (the eye_hal API takes &self/&mut self, so normal Rust
// aliasing rules already prevent that within a single Handle).
unsafe impl Send for Handle {}

impl Handle {
    /// Opens a camera by its Aravis device ID (as returned by Context::devices()), or the
    /// first available camera if `id` is empty.
    pub fn with_id(id: &str) -> Result<Self> {
        let cstr = if id.is_empty() {
            None
        } else {
            Some(CString::new(id).map_err(|e| Error::new(ErrorKind::Other, e))?)
        };

        unsafe {
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            let name_ptr = cstr.as_ref().map_or(ptr::null(), |c| c.as_ptr());
            let camera = aravis_sys::arv_camera_new(name_ptr, &mut err);
            check_error(err)?;
            if camera.is_null() {
                return Err(Error::new(ErrorKind::Other, "arv_camera_new returned null"));
            }

            Ok(Handle { camera })
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            gobject_sys::g_object_unref(self.camera as *mut _);
        }
    }
}

impl<'a> Device<'a> for Handle {
    type Stream = StreamHandle;

    fn streams(&self) -> Result<Vec<stream::Descriptor>> {
        unsafe {
            let (mut min_w, mut max_w) = (0i32, 0i32);
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_get_width_bounds(self.camera, &mut min_w, &mut max_w, &mut err);
            check_error(err)?;

            let (mut min_h, mut max_h) = (0i32, 0i32);
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_get_height_bounds(self.camera, &mut min_h, &mut max_h, &mut err);
            check_error(err)?;

            // Report the rate the camera is actually configured for, since callers use
            // Descriptor::interval for timing decisions such as capture deadlines and buffer
            // counts. AcquisitionFrameRate is a standard SFNC feature, but a camera may leave
            // it unimplemented or unset, in which case the call fails or yields a
            // non-positive rate — fall back to 30 fps there.
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            let fps = aravis_sys::arv_camera_get_frame_rate(self.camera, &mut err);
            let fps = if err.is_null() && fps > 0.0 {
                fps
            } else {
                if !err.is_null() {
                    glib_sys::g_error_free(err);
                }
                DEFAULT_FRAME_RATE
            };

            // GenICam cameras don't expose a discrete list of supported resolutions the way
            // UVC devices enumerate through V4L2 — advertise the sensor's full resolution as a
            // single stream. start_stream() debayers the native BayerRG8 sensor data to RGB24
            // internally, so downstream consumers (which only know YUYV/UYVY/RGB from the
            // other backends) see a familiar pixel format.
            Ok(vec![stream::Descriptor {
                width: max_w as u32,
                height: max_h as u32,
                pixfmt: PixelFormat::Rgb(24),
                interval: std::time::Duration::from_secs_f64(1.0 / fps),
            }])
        }
    }

    fn controls(&self) -> Result<Vec<control::Descriptor>> {
        // TODO: map GenICam features (ExposureTime, Gain, WhiteBalance, ...) to eye_hal
        // controls. For now start_stream() unconditionally enables continuous auto-exposure
        // and auto-gain, which is enough to get a usable image.
        Ok(Vec::new())
    }

    fn control(&self, _id: u32) -> Result<control::State> {
        Err(Error::new(
            ErrorKind::NotSupported,
            "controls not yet implemented for the aravis backend",
        ))
    }

    fn set_control(&mut self, _id: u32, _val: &control::State) -> Result<()> {
        Err(Error::new(
            ErrorKind::NotSupported,
            "controls not yet implemented for the aravis backend",
        ))
    }

    fn start_stream(&self, desc: &stream::Descriptor) -> Result<Self::Stream> {
        unsafe {
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_set_region(
                self.camera,
                0,
                0,
                desc.width as i32,
                desc.height as i32,
                &mut err,
            );
            check_error(err)?;

            // The camera's power-on default exposure/gain is far too conservative for normal
            // room lighting (an initial capture with no controls set came back essentially
            // black); continuous auto-exposure/gain gets a usable image without needing a
            // control mapping yet (see the TODO in controls() above).
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_set_exposure_time_auto(
                self.camera,
                aravis_sys::ARV_AUTO_CONTINUOUS,
                &mut err,
            );
            check_error(err)?;
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_set_gain_auto(
                self.camera,
                aravis_sys::ARV_AUTO_CONTINUOUS,
                &mut err,
            );
            check_error(err)?;

            // Aravis has no dedicated ArvCamera convenience wrapper for BalanceWhiteAuto (the
            // way it does for ExposureAuto/GainAuto above), so set it through the generic
            // named-feature API instead; it's still a standard GenICam SFNC enumeration
            // feature (Off/Once/Continuous) on this camera. Without it the raw Bayer channels
            // come out with a strong red/orange cast that debayering alone can't fix.
            //
            // Best-effort: BalanceWhiteAuto is optional in the SFNC, so a monochrome sensor or
            // a colour camera with a different AWB model will reject it. That costs image
            // quality on this camera but is not a reason to refuse to stream on others, unlike
            // the auto-exposure and auto-gain calls above, without which frames come back
            // essentially black.
            let feature = std::ffi::CString::new("BalanceWhiteAuto").unwrap();
            let value = std::ffi::CString::new("Continuous").unwrap();
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_set_string(
                self.camera,
                feature.as_ptr(),
                value.as_ptr(),
                &mut err,
            );
            if !err.is_null() {
                log::warn!(
                    "camera rejected BalanceWhiteAuto=Continuous, continuing without automatic \
                     white balance: {}",
                    std::ffi::CStr::from_ptr((*err).message).to_string_lossy()
                );
                glib_sys::g_error_free(err);
            }

            StreamHandle::new(self.camera, desc.width, desc.height)
        }
    }
}
