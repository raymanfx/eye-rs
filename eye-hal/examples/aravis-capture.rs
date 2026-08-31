//! Proves we can talk to a USB3 Vision / GenICam camera (e.g. the Daheng MER2 series)
//! through the Aravis library. eye-hal's V4L2/UVC/openpnp backends only understand UVC
//! devices and can never see a USB3 Vision camera like this one, no matter the OS.
//!
//! Captures a single BayerRG8 frame, debayers it to RGB8, and writes it to disk so the
//! result can be visually inspected.
//!
//! Uses aravis-sys directly rather than the `aravis` safe wrapper crate: the wrapper's
//! `Camera::new` fails to find any device (including by exact device ID) even when
//! enumeration finds it fine, which looks like a `None`/empty-string marshaling bug in
//! that binding.
//!
//! Run with: cargo run --release --example aravis-capture --features aravis

use std::ffi::CStr;
use std::ptr;

use glib_sys::GError;

unsafe fn check(err: *mut GError) -> Result<(), Box<dyn std::error::Error>> {
    if err.is_null() {
        Ok(())
    } else {
        let msg = CStr::from_ptr((*err).message)
            .to_string_lossy()
            .into_owned();
        glib_sys::g_error_free(err);
        Err(msg.into())
    }
}

fn clamp_coord(v: i32, max: i32) -> u32 {
    v.clamp(0, max) as u32
}

/// Bilinear-interpolated debayer for the RGGB pattern (Aravis "BayerRG8"): row 0 is
/// R,G,R,G,..., row 1 is G,B,G,B,... Edge pixels clamp to the nearest in-bounds neighbor.
fn debayer_rggb(raw: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as i32;
    let h = height as i32;
    let get = |x: i32, y: i32| -> u32 {
        let x = clamp_coord(x, w - 1);
        let y = clamp_coord(y, h - 1);
        raw[(y * width + x) as usize] as u32
    };

    let mut rgb = vec![0u8; (width * height * 3) as usize];
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
            rgb[i] = r as u8;
            rgb[i + 1] = g as u8;
            rgb[i + 2] = b as u8;
        }
    }
    rgb
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "aravis-frame.png".to_string());

    unsafe {
        let mut err: *mut GError = ptr::null_mut();
        let camera = aravis_sys::arv_camera_new(ptr::null(), &mut err);
        check(err)?;
        if camera.is_null() {
            return Err("arv_camera_new returned null".into());
        }

        let mut err: *mut GError = ptr::null_mut();
        let model_ptr = aravis_sys::arv_camera_get_model_name(camera, &mut err);
        check(err)?;
        let model = CStr::from_ptr(model_ptr).to_string_lossy();

        let mut err: *mut GError = ptr::null_mut();
        let fmt_ptr = aravis_sys::arv_camera_get_pixel_format_as_string(camera, &mut err);
        check(err)?;
        let pixel_format = CStr::from_ptr(fmt_ptr).to_string_lossy();
        println!("camera: {model} ({pixel_format})");

        let (mut min_w, mut max_w) = (0i32, 0i32);
        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_get_width_bounds(camera, &mut min_w, &mut max_w, &mut err);
        check(err)?;
        let (mut min_h, mut max_h) = (0i32, 0i32);
        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_get_height_bounds(camera, &mut min_h, &mut max_h, &mut err);
        check(err)?;

        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_set_region(camera, 0, 0, max_w, max_h, &mut err);
        check(err)?;

        let (mut x, mut y, mut width, mut height) = (0i32, 0i32, 0i32, 0i32);
        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_get_region(
            camera,
            &mut x,
            &mut y,
            &mut width,
            &mut height,
            &mut err,
        );
        check(err)?;
        println!("resolution: {width}x{height}");

        // The camera's power-on default exposure/gain is far too conservative for normal
        // room lighting (the first capture came back essentially black). Enable continuous
        // auto-exposure/gain and grab several frames so the algorithm has time to converge
        // before we keep one.
        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_set_exposure_time_auto(
            camera,
            aravis_sys::ARV_AUTO_CONTINUOUS,
            &mut err,
        );
        check(err)?;
        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_set_gain_auto(camera, aravis_sys::ARV_AUTO_CONTINUOUS, &mut err);
        check(err)?;

        let mut err: *mut GError = ptr::null_mut();
        let stream = aravis_sys::arv_camera_create_stream(camera, None, ptr::null_mut(), &mut err);
        check(err)?;
        if stream.is_null() {
            return Err("arv_camera_create_stream returned null".into());
        }

        let mut err: *mut GError = ptr::null_mut();
        let payload = aravis_sys::arv_camera_get_payload(camera, &mut err);
        check(err)?;

        for _ in 0..5 {
            let buf = aravis_sys::arv_buffer_new_allocate(payload as libc::size_t);
            aravis_sys::arv_stream_push_buffer(stream, buf);
        }

        // Deliberately avoid arv_camera_set_chunks / the CLI test tool's default
        // chunk-mode toggle here: this camera's firmware NAKs that specific register
        // write ("wrong-config"), which is a quirk of that toggle, not of streaming.
        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_start_acquisition(camera, &mut err);
        check(err)?;

        const WARMUP_FRAMES: usize = 30;
        let mut buffer: *mut aravis_sys::ArvBuffer = ptr::null_mut();
        for i in 0..WARMUP_FRAMES {
            if !buffer.is_null() {
                aravis_sys::arv_stream_push_buffer(stream, buffer);
            }
            buffer = aravis_sys::arv_stream_timeout_pop_buffer(stream, 5_000_000);
            if buffer.is_null() {
                return Err(format!("no buffer received within 5s timeout (frame {i})").into());
            }
            // A fresh, null err per call: GLib requires the GError out-parameter to point at
            // NULL on entry, so reusing one across calls both breaks that contract and drops
            // the previous error on the floor. check() frees whatever it is handed.
            let mut err: *mut GError = ptr::null_mut();
            let exposure = aravis_sys::arv_camera_get_exposure_time(camera, &mut err);
            check(err)?;
            let mut err: *mut GError = ptr::null_mut();
            let gain = aravis_sys::arv_camera_get_gain(camera, &mut err);
            check(err)?;
            println!("frame {i}: exposure={exposure}us gain={gain}dB");
        }

        let mut err: *mut GError = ptr::null_mut();
        aravis_sys::arv_camera_stop_acquisition(camera, &mut err);
        check(err)?;

        if buffer.is_null() {
            return Err("no buffer received within 5s timeout".into());
        }

        let status = aravis_sys::arv_buffer_get_status(buffer);
        if status != aravis_sys::ARV_BUFFER_STATUS_SUCCESS {
            return Err(format!("buffer status: {status}").into());
        }

        let mut size: libc::size_t = 0;
        let data_ptr = aravis_sys::arv_buffer_get_data(buffer, &mut size);
        let raw = std::slice::from_raw_parts(data_ptr, size as usize);
        println!("got frame: {size} bytes");

        let rgb = debayer_rggb(raw, width as u32, height as u32);
        image::ImageBuffer::<image::Rgb<u8>, &[u8]>::from_raw(width as u32, height as u32, &rgb)
            .ok_or("failed to convert bytes to an image")?
            .save(&out_path)?;
        println!("wrote {out_path}");
    }

    Ok(())
}
