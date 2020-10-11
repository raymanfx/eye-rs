extern crate eye;

use eye::prelude::*;

fn main() {
    // Create a list of valid capture devices in the system.
    let list = DeviceFactory::enumerate();

    // Print the supported formats for each device.
    for info in list {
        println!("Index {}", info.index);
        println!("  Name    : {}", info.name);
        let dev = DeviceFactory::create(info.index as usize);
        if dev.is_err() {
            println!("Failed to open video device {}", info.index);
        }
        let dev = dev.unwrap();

        let formats = dev.query_formats();
        if formats.is_err() {
            println!("Failed to query formats");
        }
        let formats = formats.unwrap();

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
}
