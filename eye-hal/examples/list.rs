use eye_hal::traits::Context;
use eye_hal::{PlatformContext, Result};

fn main() -> Result<()> {
    // Create a context
    let ctx = PlatformContext::default();

    // Create a list of valid capture devices in the system.
    let list = ctx.devices()?;

    // Print the info for each device.
    for uri in list {
        println!("{}", uri);
    }

    Ok(())
}
