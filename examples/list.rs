extern crate eye;

use eye::prelude::*;

fn main() {
    // Create a list of valid capture devices in the system.
    let list = DeviceFactory::enumerate();

    // Print the info for each device.
    for info in list {
        println!("Index {}", info.index);
        println!("  Name    : {}", info.name);
    }
}
