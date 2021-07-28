use itertools::Itertools;

use eye_hal::traits::{Context, Device};
use eye_hal::{PlatformContext, Result};

fn main() -> Result<()> {
    // Create a context
    let ctx = PlatformContext::default();

    // Create a list of valid capture devices in the system.
    let list = ctx.devices()?;

    // Print the supported formats for each device.
    for desc in list {
        println!("{}", desc.uri);
        let dev = ctx.open_device(&desc.uri)?;
        let streams = dev.streams()?;

        println!("  Streams:");
        for (pixfmt, streams) in &streams.into_iter().group_by(|desc| desc.pixfmt.clone()) {
            println!("");
            println!("    Pixelformat : {}", pixfmt);

            // sort by resolution, smallest first
            let streams = streams.into_iter().sorted_by(|a, b| a.width.cmp(&b.width));

            for (res, streams) in &streams.group_by(|desc| (desc.width, desc.height)) {
                // sort by frame interval, smallest first
                let streams = streams
                    .into_iter()
                    .sorted_by(|a, b| a.interval.cmp(&b.interval));

                print!("      {}x{}", res.0, res.1);
                print!(" : [");
                for stream in streams {
                    print!("{:?}, ", stream.interval);
                }
                println!("]");
            }
        }
    }

    Ok(())
}
