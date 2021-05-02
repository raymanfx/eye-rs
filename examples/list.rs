use eye::prelude::*;
use eye::Result;

fn main() -> Result<()> {
    // Create a context
    let ctx = Context::new();

    // Create a list of valid capture devices in the system.
    let list = ctx.query_devices()?;

    // Print the info for each device.
    for uri in list {
        println!("{}", uri);
    }

    Ok(())
}
