pub mod format;
pub mod traits;

pub mod hal;

pub mod prelude {
    pub use crate::{
        format::{Format, FourCC},
        hal::traits::Device,
        hal::{DeviceFactory, DeviceInfo, DeviceList},
        traits::Stream,
    };
}
