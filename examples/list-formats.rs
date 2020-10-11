use std::io;

use eye::prelude::*;

fn main() -> io::Result<()> {
    // Create a list of valid capture devices in the system.
    let list = Device::enumerate();

    // Print the supported formats for each device.
    for uri in list {
        println!("{}", uri);
        let dev = Device::with_uri(&uri)?;
        let formats = dev.query_formats()?;

        // group by pixelformat
        let mut grouped: Vec<Vec<Format>> = Vec::new();
        for fmt in formats {
            let mut new = true;
            for group in &mut grouped {
                let first = group[0];
                if first.pixfmt == fmt.pixfmt {
                    group.push(fmt);
                    new = false;
                }
            }

            if new {
                grouped.push(vec![fmt]);
            }
        }

        // sort the resolutions
        for group in &mut grouped {
            group.sort_by(|a, b| a.width.cmp(&b.width));
        }

        println!("  Formats:");
        for group in &grouped {
            println!("    Pixelformat   : {}", group[0].pixfmt);
            print!("    Resolutions   : ");
            for fmt in group {
                print!("{}x{} ", fmt.width, fmt.height);
            }
            println!();
        }
    }

    Ok(())
}
