# eye

`eye` provides high-level abstractions for camera hardware interaction in Rust.
It leverages the other parts of the eye stack such as `eye-hal` to provide a cross-plattform
abstraction layer.

Where `eye-hal` facilitates a rather low level abstraction, `eye` is designed to expand on that
and leverage modern programming patterns such as async code. The main goal of this crate is to
provide an easy to use, high-level API.

Other features of the high-level `eye` crate include transparent frame format conversion and more.
