use eye_hal::format::PixelFormat;
use eye_hal::traits::{Context, Device, Stream};
use eye_hal::PlatformContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a context
    let ctx = PlatformContext::default();

    // Query for available devices.
    let devices = ctx.devices()?;

    // First, we need a capture device to read images from. For this example, let's just choose
    // whatever device is first in the list.
    let dev = ctx.open_device(&devices[0].uri)?;

    // Query for available streams and just choose the first one.
    let streams = dev.streams()?;
    let stream_desc = streams[0].clone();
    println!("Stream: {:?}", stream_desc);

    // Since we want to capture images, we need to access the native image stream of the device.
    // The backend will internally select a suitable implementation for the platform stream. On
    // Linux for example, most devices support memory-mapped buffers.
    let mut stream = dev.start_stream(&stream_desc)?;

    // Here we create a loop and just capture images as long as the device produces them. Normally,
    // this loop will run forever unless we unplug the camera or exit the program.
    loop {
        let frame = stream
            .next()
            .expect("Stream is dead")
            .expect("Failed to capture frame");
        let frame = match stream_desc.pixfmt {
            PixelFormat::Custom(fmt) if fmt == "YUYV" => {
                frame.chunks_exact(4).fold(vec![], |mut acc, v| {
                    // convert form YUYV to RGB
                    let [y, u, _, v]: [u8; 4] = std::convert::TryFrom::try_from(v).unwrap();
                    let y = y as f32;
                    let u = u as f32;
                    let v = v as f32;

                    let b = 1.164 * (y - 16.) + 2.018 * (u - 128.);

                    let g = 1.164 * (y - 16.) - 0.813 * (v - 128.) - 0.391 * (u - 128.);

                    let r = 1.164 * (y - 16.) + 1.596 * (v - 128.);
                    let r = r as u8;
                    let g = g as u8;
                    let b = b as u8;

                    acc.push(r);
                    acc.push(g);
                    acc.push(b);
                    acc.push(r);
                    acc.push(g);
                    acc.push(b);
                    acc
                })
            }
            _ => unimplemented!(),
        };

        image::ImageBuffer::<image::Rgb<u8>, &[u8]>::from_raw(
            stream_desc.width,
            stream_desc.height,
            &frame,
        )
        .ok_or("failed to convert bytes to an image")?
        .save("image.png")?;

        // one picture is enough
        break Ok(());
    }
}
