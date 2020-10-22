use std::io;

use eye::control::{MenuItem, Representation};
use eye::prelude::*;

fn main() -> io::Result<()> {
    // Create a list of valid capture devices in the system.
    let list = Context::enumerate_devices();

    // Print the supported controls for each device.
    for uri in list {
        println!("{}", uri);
        let dev = Context::open_device(&uri)?;
        let controls = dev.query_controls()?;

        println!("  Controls:");
        for ctrl in &controls {
            println!("    * {}", ctrl.name);
            #[allow(unreachable_patterns)]
            match &ctrl.repr {
                Representation::Unknown => {
                    println!("      Type    : Unknown");
                }
                Representation::Button => {
                    println!("      Type    : Button");
                }
                Representation::Boolean => {
                    println!("      Type    : Boolean");
                }
                Representation::Integer {
                    range,
                    step,
                    default,
                } => {
                    println!("      Type    : Integer");
                    println!("      Range   : ({}, {})", range.0, range.1);
                    println!("      Step    : {}", step);
                    println!("      Default : {}", default);
                }
                Representation::String => {
                    println!("      Type    : String");
                }
                Representation::Bitmask => {
                    println!("      Type    : Bitmask");
                }
                Representation::Menu(items) => {
                    println!("      Type    : Menu ==>");
                    for item in items {
                        match item {
                            MenuItem::String(str) => {
                                println!("       - {}", str);
                            }
                            MenuItem::Integer(val) => {
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
