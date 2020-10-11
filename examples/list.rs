use eye::prelude::*;

fn main() {
    // Create a list of valid capture devices in the system.
    let list = DeviceFactory::enumerate();

    // Print the info for each device.
    for uri in list {
        println!("{}", uri);
    }
}
