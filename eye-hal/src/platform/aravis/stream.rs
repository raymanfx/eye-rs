use std::ptr;

use crate::error::{Error, ErrorKind, Result};
use crate::platform::aravis::check_error;
use crate::traits::Stream;

pub struct Handle {
    stream: *mut aravis_sys::ArvStream,
    camera: *mut aravis_sys::ArvCamera,
    width: u32,
    height: u32,
    rgb: Vec<u8>,
    acquiring: bool,
}

// See the Send justification on aravis::device::Handle; the same reasoning applies to the
// stream and camera pointers held here.
unsafe impl Send for Handle {}

impl Handle {
    pub(crate) fn new(camera: *mut aravis_sys::ArvCamera, width: u32, height: u32) -> Result<Self> {
        unsafe {
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            let stream =
                aravis_sys::arv_camera_create_stream(camera, None, ptr::null_mut(), &mut err);
            check_error(err)?;
            if stream.is_null() {
                return Err(Error::new(
                    ErrorKind::Other,
                    "arv_camera_create_stream returned null",
                ));
            }

            let mut err: *mut glib_sys::GError = ptr::null_mut();
            let payload = aravis_sys::arv_camera_get_payload(camera, &mut err);
            check_error(err)?;

            // A handful of in-flight buffers so the stream can keep filling one while we
            // process the last one popped.
            for _ in 0..5 {
                let buf = aravis_sys::arv_buffer_new_allocate(payload as libc::size_t);
                aravis_sys::arv_stream_push_buffer(stream, buf);
            }

            // Deliberately avoid arv_camera_set_chunks / the chunk-mode toggle some Aravis
            // tools (e.g. arv-camera-test) apply by default: this camera's firmware NAKs that
            // specific register write ("wrong-config"), which is a quirk of that toggle, not
            // of streaming itself.
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_start_acquisition(camera, &mut err);
            check_error(err)?;

            // Take our own reference on the camera. `Device::start_stream` takes `&self` with
            // an elided lifetime and `impl<'a> Device<'a>` leaves `'a` free, so the returned
            // stream is not borrow-checked against the device: the caller is free to drop the
            // device::Handle — whose Drop unrefs the camera — while this stream is still
            // alive. Holding a reference of our own keeps the camera alive for as long as
            // stop() and the other camera calls below need it. Taken here rather than on
            // entry so the `?` paths above have nothing to unwind.
            gobject_sys::g_object_ref(camera as *mut _);

            Ok(Handle {
                stream,
                camera,
                width,
                height,
                rgb: vec![0u8; (width * height * 3) as usize],
                acquiring: true,
            })
        }
    }

    fn stop(&mut self) {
        if !self.acquiring {
            return;
        }
        unsafe {
            let mut err: *mut glib_sys::GError = ptr::null_mut();
            aravis_sys::arv_camera_stop_acquisition(self.camera, &mut err);
            // Best-effort on shutdown; ignore the result but still free the GError.
            if !err.is_null() {
                glib_sys::g_error_free(err);
            }
        }
        self.acquiring = false;
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        self.stop();
        unsafe {
            gobject_sys::g_object_unref(self.stream as *mut _);
            // Releases the reference taken in new(); stop() above is the last camera use.
            gobject_sys::g_object_unref(self.camera as *mut _);
        }
    }
}

fn clamp_coord(v: i32, max: i32) -> u32 {
    v.clamp(0, max) as u32
}

/// Bilinear-interpolated debayer for the RGGB pattern (Aravis "BayerRG8"): row 0 is
/// R,G,R,G,..., row 1 is G,B,G,B,... Edge pixels clamp to the nearest in-bounds neighbor.
fn debayer_rggb_into(raw: &[u8], width: u32, height: u32, out: &mut [u8]) {
    let w = width as i32;
    let h = height as i32;
    let get = |x: i32, y: i32| -> u32 {
        let x = clamp_coord(x, w - 1);
        let y = clamp_coord(y, h - 1);
        raw[(y * width + x) as usize] as u32
    };

    for y in 0..h {
        for x in 0..w {
            let red_row = y % 2 == 0;
            let red_col = x % 2 == 0;
            let (r, g, b) = match (red_row, red_col) {
                (true, true) => (
                    get(x, y),
                    (get(x - 1, y) + get(x + 1, y) + get(x, y - 1) + get(x, y + 1)) / 4,
                    (get(x - 1, y - 1) + get(x + 1, y - 1) + get(x - 1, y + 1) + get(x + 1, y + 1))
                        / 4,
                ),
                (true, false) => (
                    (get(x - 1, y) + get(x + 1, y)) / 2,
                    get(x, y),
                    (get(x, y - 1) + get(x, y + 1)) / 2,
                ),
                (false, true) => (
                    (get(x, y - 1) + get(x, y + 1)) / 2,
                    get(x, y),
                    (get(x - 1, y) + get(x + 1, y)) / 2,
                ),
                (false, false) => (
                    (get(x - 1, y - 1) + get(x + 1, y - 1) + get(x - 1, y + 1) + get(x + 1, y + 1))
                        / 4,
                    (get(x - 1, y) + get(x + 1, y) + get(x, y - 1) + get(x, y + 1)) / 4,
                    get(x, y),
                ),
            };
            let i = ((y * w + x) * 3) as usize;
            out[i] = r as u8;
            out[i + 1] = g as u8;
            out[i + 2] = b as u8;
        }
    }
}

impl<'a> Stream<'a> for Handle {
    type Item = Result<&'a [u8]>;

    fn next(&'a mut self) -> Option<Self::Item> {
        unsafe {
            let buffer = aravis_sys::arv_stream_timeout_pop_buffer(self.stream, 5_000_000);
            if buffer.is_null() {
                return Some(Err(Error::new(
                    ErrorKind::Other,
                    "timed out waiting for a frame",
                )));
            }

            let status = aravis_sys::arv_buffer_get_status(buffer);
            if status != aravis_sys::ARV_BUFFER_STATUS_SUCCESS {
                aravis_sys::arv_stream_push_buffer(self.stream, buffer);
                return Some(Err(Error::new(
                    ErrorKind::Other,
                    format!("buffer status: {status}"),
                )));
            }

            let mut size: libc::size_t = 0;
            let data_ptr = aravis_sys::arv_buffer_get_data(buffer, &mut size);
            // from_raw_parts requires a non-null pointer even for a zero length, and a buffer
            // carrying no payload data hands back null.
            if data_ptr.is_null() {
                aravis_sys::arv_stream_push_buffer(self.stream, buffer);
                return Some(Err(Error::new(
                    ErrorKind::Other,
                    "arv_buffer_get_data returned null",
                )));
            }
            let raw = std::slice::from_raw_parts(data_ptr, size as usize);

            // debayer_rggb_into clamps its sampling coordinates to the width/height it is
            // given, so it reads up to index width * height - 1 and panics on anything
            // shorter. The payload can come up short if the camera did not honour the ROI set
            // in start_stream, or if it is not emitting 8-bit Bayer at all.
            let expected = (self.width as usize) * (self.height as usize);
            if raw.len() < expected {
                aravis_sys::arv_stream_push_buffer(self.stream, buffer);
                return Some(Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "payload too small: got {} bytes, expected {expected}",
                        raw.len()
                    ),
                )));
            }

            debayer_rggb_into(raw, self.width, self.height, &mut self.rgb);

            aravis_sys::arv_stream_push_buffer(self.stream, buffer);

            Some(Ok(&self.rgb))
        }
    }
}
