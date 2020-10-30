# Eye

[![crates.io](https://img.shields.io/crates/v/eye.svg?style=for-the-badge)](https://crates.io/crates/eye)
[![license](https://img.shields.io/github/license/raymanfx/eye-rs?style=for-the-badge)](https://github.com/raymanfx/eye-rs/blob/master/LICENSE.txt)
[![Build Status](https://img.shields.io/travis/raymanfx/eye-rs/master.svg?style=for-the-badge&logo=travis)](https://travis-ci.org/raymanfx/eye-rs)

Eye is a cross platform camera capture and control library written in native Rust.
It features multiple platform backends, such as v4l2 for Linux. Buffers are captured by accessing
'streams'. The stream concept is used to facilitate additional features such as colorspace
conversion.

A backend is also called 'HAL' aka hardware abstraction layer in eye-rs. An OS may support multiple HALs at runtime, depending on which HALs were selected at compile time. Capture devices are then identified via their URI so you can choose the HAL which best fits your needs.

Eye is a very young library and its API is subject to change (as denoted by the 0.x.x version
number). We follow the semver approach, meaning each new feature will bump the minor version by one.

## Goals

Eye strives to provide a common feature set on all platforms. Some devices, mostly more expensive
ones, will always offer more features than others though. Eye shall expose a dynamic featureset API
which can be queried at runtime so device parameters can be configured accordingly.

#### Common Features

 * [x] Transparent pixel format conversion

#### OS Feature Matrix

| Feature                                       | Linux     | Windows   | macOS     |
| --------------------------------------------- |:---------:|:---------:|:---------:|
| Image capture                                 | &check;   | &#10540;  | &check;   |
| Device enumeration                            | &check;   | &#10540;  | &check;   |
| Device parameters (Focus, White Balance, ...) | &check;   | &#10540;  | &#10540;  |

There are various HAL specific properties. For example, the v4l2 HAL on Linux supports zero-copy capture (as far as userspace is concerned - the kernel driver may still perform a copy). Those will be enumerated here in the future.

## Usage
Below you can find a quick example usage of this crate. It introduces the basics necessary for image capturing.

```rust
use eye::prelude::*;

fn main() {
    // Query for available devices.
    let devices = Context::enumerate_devices();
    if devices.len() == 0 {
        println!("No devices available");
        return;
    }

    // First, we need a capture device to read images from. For this example, let's just choose
    // whatever device is first in the list.
    let dev = Context::open_device(&devices[0]).expect("Failed to open video device");

    // Now fetch the current device format. The format contains parameters such as frame width,
    // height and the buffer format (RGB, JPEG, etc).
    let format = dev.get_format().expect("Failed to read native format");

    // Since we want to capture images, we need to access the native image stream of the device.
    // The backend will internally select a suitable implementation for the platform stream. On
    // Linux for example, most devices support memory-mapped buffers.
    //
    // Keep in mind that no format conversion is performed by default, so the frames you get in
    // this stream are directly handed to you without any copy. If you need a common frame format
    // such as raw RGB, you will have to create a seperate stream to perform the conversion.
    let mut stream = dev.stream().expect("Failed to setup capture stream");

    // Here we create a loop and just capture images as long as the device produces them. Normally,
    // this loop will run forever unless we unplug the camera or exit the program.
    loop {
        let frame = stream
            .next()
            .expect("Stream is dead")
            .expect("Failed to capture frame");
    }
}
```

Have a look at the provided `examples` for more sample applications.
