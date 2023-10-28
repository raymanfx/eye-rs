# Changelog

### 0.5
> #### Added
> * Device product description (`device::Description`)
> * Glium example
> * Restored openpnp-capture HAL
>   - libuvc does not support iSight cameras of current-gen MacBooks
>   - libuvc remains as the default Windows backend, while macOS defaults to openpnp-capture
> * Device controls (brightness, saturation, ...) for openpnp-capture HAL
> * Support for YUYV formats (autoconversion to RGB) when using the high-level `eye` crate

### 0.4
> #### Added
> * Universal Video Class (UVC) HAL
>   - Tested on macOS and working well
>   - Can work on Linux and Windows as well in theory
> #### Changed
> * HAL is now a leaky abstraction layer
>   - Implementation details are available by matching the enum type
> * Less `unsafe` in the HAL implementations
> #### Removed
> * openpnp-capture HAL
>   - Too buggy

### 0.3
> * Linux/macOS: OpenPnP HAL
>   - pulls in the Rust bindings for the C++ library
> * Common: Context struct for device instance creation
>   - replaces DeviceFactory
> * Common: CowImage for easy frame data handling
>   - frame data can now be sent between threads easily
>   - supports the zero-copy design of eye-rs
> * Common: Decouple stream lifetime from device lifetime
>   - controls can now be changed while capturing

### 0.2
> * Common: PixelFormat struct
> * Common: Transparent pixel format conversion for streams
> * Common: JPEG decoding (feature = "jpeg")
> * Common: JPEG -> {BGRA, RGB, RGBA} conversion
> * Linux: device enumeration
> * Linux: control parameters

### 0.1
> * Linux: zero-copy capture
> * Initial release
