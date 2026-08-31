//! Proves the aravis backend works through eye_hal's normal Context/Device/Stream trait
//! abstraction (not the raw aravis-sys FFI calls in aravis-capture.rs) — i.e. this is the
//! path a normal consumer of the crate goes through.
//!
//! Run with: cargo run --release --example aravis-hal-capture --features aravis

use eye_hal::traits::{Context, Device, Stream};
use eye_hal::PlatformContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "aravis-hal-frame.png".to_string());

    let ctx = PlatformContext::all()
        .find(|ctx| {
            ctx.devices()
                .map(|devs| devs.iter().any(|d| d.uri.starts_with("aravis://")))
                .unwrap_or(false)
        })
        .ok_or("no platform context exposes an aravis:// device")?;

    let devices = ctx.devices()?;
    let desc = devices
        .iter()
        .find(|d| d.uri.starts_with("aravis://"))
        .ok_or("no aravis:// device found")?;
    println!("opening {} ({})", desc.uri, desc.product);

    let dev = ctx.open_device(&desc.uri)?;
    let streams = dev.streams()?;
    let stream_desc = streams
        .first()
        .ok_or("device advertised no streams")?
        .clone();
    println!(
        "stream: {}x{} {}",
        stream_desc.width, stream_desc.height, stream_desc.pixfmt
    );

    let mut stream = dev.start_stream(&stream_desc)?;
    let frame = stream.next().ok_or("stream produced no frame")??;
    println!("got frame: {} bytes", frame.len());

    image::ImageBuffer::<image::Rgb<u8>, &[u8]>::from_raw(
        stream_desc.width,
        stream_desc.height,
        frame,
    )
    .ok_or("failed to convert bytes to an image")?
    .save(&out_path)?;
    println!("wrote {out_path}");

    Ok(())
}
