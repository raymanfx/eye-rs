# Changelog

#### 0.3 (released)
> * Linux/macOS: OpenPnP HAL
>   - pulls in the Rust bindings for the C++ library
> * Common: Context struct for device instance creation
>   - replaces DeviceFactory
> * Common: CowImage for easy frame data handling
>   - frame data can now be sent between threads easily
>   - supports the zero-copy design of eye-rs
> * Common: Decouple stream lifetime from device lifetime
>   - controls can now be changed while capturing

#### 0.2 (released)
> * Common: PixelFormat struct
> * Common: Transparent pixel format conversion for streams
> * Common: JPEG decoding (feature = "jpeg")
> * Common: JPEG -> {BGRA, RGB, RGBA} conversion
> * Linux: device enumeration
> * Linux: control parameters

#### 0.1 (released)
> * Linux: zero-copy capture
> * Initial release
