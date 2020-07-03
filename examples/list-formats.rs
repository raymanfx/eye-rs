extern crate eye;

use eye::prelude::*;

fn main() {
    // Create a list of valid capture devices in the system.
    let list = DeviceList::enumerate();

    // Print the supported formats for each device.
    for info in list {
        println!("Index {}", info.index);
        println!("  Name    : {}", info.name);
        println!("  Formats:");
        for fmt in &info.formats {
            println!("    Fourcc        : {}", fmt.fourcc);
            println!("    Resolutions   : {:?}", fmt.resolutions);
            println!("    Emulated      : {}", fmt.emulated);
        }
    }
}
