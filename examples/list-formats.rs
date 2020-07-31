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

        println!("  Formats:");
        for fmt in &formats {
            println!("    Pixelformat   : {}", fmt.pixfmt);
            println!("    Resolutions   : {:?}", fmt.resolutions);
            println!("    Emulated      : {}", fmt.emulated);
        }
    }
}
