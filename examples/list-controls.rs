extern crate eye;

use eye::control::{MenuItem, Representation};
use eye::prelude::*;

fn main() {
    // Create a list of valid capture devices in the system.
    let list = DeviceFactory::enumerate();

    // Print the supported controls for each device.
    for info in list {
        println!("Index {}", info.index);
        println!("  Name    : {}", info.name);
        let dev = DeviceFactory::create(info.index as usize);
        if dev.is_err() {
            println!("Failed to open video device {}", info.index);
        }
        let dev = dev.unwrap();

        let controls = dev.query_controls();
        if controls.is_err() {
            println!("Failed to query controls");
        }
        let controls = controls.unwrap();

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
                Representation::Integer(constraints) => {
                    println!("      Type    : Integer");
                    println!(
                        "      Range   : ({}, {})",
                        constraints.range.0, constraints.range.1
                    );
                    println!("      Step    : {}", constraints.step);
                    println!("      Default : {}", constraints.default);
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
}
