use itertools::Itertools;

use eye::prelude::*;
use eye::Result;

fn main() -> Result<()> {
    // Create a context
    let ctx = Context::new();

    // Create a list of valid capture devices in the system.
    let list = ctx.query_devices()?;

    // Print the supported formats for each device.
    for uri in list {
        println!("{}", uri);
        let dev = Device::with_uri(&uri)?;
        let streams = dev.query_streams()?;

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
