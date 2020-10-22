use eye::prelude::*;

fn main() {
    // Create a list of valid capture devices in the system.
    let list = Context::enumerate_devices();

    // Print the info for each device.
    for uri in list {
        println!("{}", uri);
    }
}
