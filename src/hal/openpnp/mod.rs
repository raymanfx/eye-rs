//! OpenPnP (pick and place) backend
//!
//! # Related Links
//! * <https://openpnp.org/> - OpenPnP project homepage

pub mod device;
pub mod stream;

use openpnp_capture as pnp;

pub fn devices() -> Vec<String> {
    pnp::Device::enumerate()
        .into_iter()
        .filter_map(|index| {
            match pnp::Device::new(index) {
                Some(_) => {}
                None => return None,
            };

            Some(format!("{}", index))
        })
        .collect()
}
