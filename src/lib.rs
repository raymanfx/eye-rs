pub mod control;
pub mod device;
pub mod format;
pub mod traits;

pub mod hal;

pub mod prelude {
    pub use crate::{
        device::Info as DeviceInfo,
        format::{Format, FourCC},
        hal::traits::Device,
        hal::{DeviceFactory, DeviceList},
        traits::Stream,
    };
}
