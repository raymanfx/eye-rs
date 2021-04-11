use eye::control::{MenuItem, Type};
use eye::prelude::*;
use eye::Result;

fn main() -> Result<()> {
    // Create a context
    let ctx = Context::new();

    // Create a list of valid capture devices in the system.
    let list = ctx.query_devices()?;

    // Print the supported controls for each device.
    for uri in list {
        println!("{}", uri);
        let dev = Device::with_uri(&uri)?;
        let controls = dev.query_controls()?;

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
