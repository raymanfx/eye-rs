use std::io;

use eye::prelude::*;

fn main() -> io::Result<()> {
    // Create a list of valid capture devices in the system.
    let list = Context::enumerate_devices();

    // Print the supported formats for each device.
    for uri in list {
        println!("{}", uri);
        let dev = Device::with_uri(&uri)?;
        let streams = dev.query_streams()?;

        println!("  Streams:");
        for (pixfmt, mut streams) in streams.group_by(|desc| desc.pixfmt.clone()) {
            // sort by width, smallest first
            streams.sort_by(|a, b| a.width.cmp(&b.width));

            println!("");
            println!("    Pixelformat : {}", pixfmt);
            for (res, mut streams) in streams.group_by(|desc| (desc.width, desc.height)) {
                // sort by frame interval, smallest first
                streams.sort_by(|a, b| a.interval.cmp(&b.interval));

                println!("      Resolution : {}x{}", res.0, res.1);
                print!("        Intervals  : ");
                for stream in streams {
                    print!("{:?}, ", stream.interval);
                }
                println!("");
            }
        }
    }

    Ok(())
}
