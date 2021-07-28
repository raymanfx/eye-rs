use eye_hal::traits::Context;
use eye_hal::{PlatformContext, Result};

fn main() -> Result<()> {
    // Iterate through all available platforms
    for ctx in PlatformContext::all() {
        // Create a list of valid capture devices in the system.
        let list = ctx.devices()?;

        // Print the info for each device.
        for desc in list {
            println!("{}", desc.uri);
            println!("  product : {}", desc.product);
        }
    }

    Ok(())
}
