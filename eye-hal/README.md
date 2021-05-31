# eye-hal

`eye-hal` strives to provide a hardware abstraction layer (HAL) for camera-like devices in Rust.
This includes common consumer grade RGB/YUV webcams as well as more sophisticated (infrared, mono)
hardware. All output buffer types should be supported, even multi-planar buffers.

Apart from buffer capturing, `eye-hal` also provides an abstraction for hardware control. An
example use-case would be white-balance or focus control.

## Design

All platform (aka backend) specific code goes into src/platform. The rest should be platform
agnostic code. Traits shared between platform implementations go into src/traits.rs.

There are three main entities when it comes to a HAL implementation:
- `Context`
  A platform specific context used to query devices and general system information.
  It is also used to actually open a device by acquiring a handle.
- `Device`
  This represents the sensor hardware itself. It can be used to manipulate hardware controls such
  as focus, white balance or gain levels. The main functionality however is in the `start_stream()`
  function, where you can create an input stream for capturing buffers.
- `Stream`
  A stream is an entitiy which provides access to the buffers captured by the camera sensor. Only
  one buffer is available at any time - this is a design decision made in the current code.
  Whenever possible, we try to implement the stream as a zero-copy mechanism (only a view of the
  buffer data is returned to the caller).

## Usage
Below you can find a quick example usage of this crate. It introduces the basics necessary for
frame capturing.

```rust
use eye_hal::PlatformContext;
use eye_hal::traits::{Context, Device, Stream};

fn main() -> Result<()> {
    // Create a context
    let ctx = PlatformContext::default();

    // Query for available devices.
    let devices = ctx.devices()?;

    // First, we need a capture device to read images from. For this example, let's just choose
    // whatever device is first in the list.
    let dev = ctx.open_device(&devices[0])?;

    // Query for available streams and just choose the first one.
    let streams = dev.streams()?;
    let stream_desc = streams[0].clone();
    println!("Stream: {:?}", stream_desc);

    // Since we want to capture images, we need to access the native image stream of the device.
    // The backend will internally select a suitable implementation for the platform stream. On
    // Linux for example, most devices support memory-mapped buffers.
    let mut stream = dev.start_stream(&stream_desc)?;

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
