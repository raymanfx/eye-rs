use std::ffi::CStr;

use crate::device;
use crate::error::{Error, ErrorKind, Result};
use crate::platform::aravis::device::Handle as DeviceHandle;
use crate::traits::Context as ContextTrait;

/// Runtime context
pub struct Context {}

impl<'a> ContextTrait<'a> for Context {
    type Device = DeviceHandle;

    fn devices(&self) -> Result<Vec<device::Description>> {
        unsafe {
            aravis_sys::arv_update_device_list();
            let n = aravis_sys::arv_get_n_devices();

            let mut out = Vec::with_capacity(n as usize);
            for i in 0..n {
                let id_ptr = aravis_sys::arv_get_device_id(i);
                if id_ptr.is_null() {
                    continue;
                }
                let id = CStr::from_ptr(id_ptr).to_string_lossy().into_owned();

                let vendor_ptr = aravis_sys::arv_get_device_vendor(i);
                let model_ptr = aravis_sys::arv_get_device_model(i);
                let vendor = if vendor_ptr.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr(vendor_ptr).to_string_lossy().into_owned()
                };
                let model = if model_ptr.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr(model_ptr).to_string_lossy().into_owned()
                };

                out.push(device::Description {
                    uri: format!("aravis://{id}"),
                    product: format!("{vendor} {model}").trim().to_string(),
                });
            }

            Ok(out)
        }
    }

    fn open_device(&self, uri: &str) -> Result<Self::Device> {
        if let Some(id) = uri.strip_prefix("aravis://") {
            DeviceHandle::with_id(id)
        } else {
            Err(Error::new(ErrorKind::Other, "invalid URI"))
        }
    }
}
