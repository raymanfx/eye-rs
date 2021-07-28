use eye_hal::control::{MenuItem, Type};
use eye_hal::traits::{Context, Device};
use eye_hal::{PlatformContext, Result};

fn main() -> Result<()> {
    // Create a context
    let ctx = PlatformContext::default();

    // Create a list of valid capture devices in the system.
    let list = ctx.devices()?;

    // Print the supported controls for each device.
    for desc in list {
        println!("{}", desc.uri);
        let dev = ctx.open_device(&desc.uri)?;
        let controls = dev.controls()?;

        println!("  Controls:");
        for ctrl in &controls {
            println!("    * {}", ctrl.name);
            #[allow(unreachable_patterns)]
            match &ctrl.typ {
                Type::Stateless => {
                    println!("      Type    : Button");
                }
                Type::Boolean => {
                    println!("      Type    : Boolean");
                }
                Type::Number { range, step } => {
                    println!("      Type    : Number");
                    println!("      Range   : ({}, {})", range.0, range.1);
                    println!("      Step    : {}", step);
                }
                Type::String => {
                    println!("      Type    : String");
                }
                Type::Bitmask => {
                    println!("      Type    : Bitmask");
                }
                Type::Menu(items) => {
                    println!("      Type    : Menu ==>");
                    for item in items {
                        match item {
                            MenuItem::String(str) => {
                                println!("       - {}", str);
                            }
                            MenuItem::Number(val) => {
                                println!("       - {}", val);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
