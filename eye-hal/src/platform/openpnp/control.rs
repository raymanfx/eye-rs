use openpnp_capture_sys as sys;

use crate::control;
use crate::error::{Error, ErrorKind, Result};

pub(crate) enum Typ {
    Limited,
    Auto,
}

const ALL: [(u32, &str, Typ); 17] = [
    (sys::CAPPROPID_EXPOSURE, "Exposure", Typ::Limited),
    (sys::CAPPROPID_EXPOSURE, "Auto Exposure", Typ::Auto),
    (sys::CAPPROPID_FOCUS, "Focus", Typ::Limited),
    (sys::CAPPROPID_FOCUS, "Auto Focus", Typ::Auto),
    (sys::CAPPROPID_ZOOM, "Zoom", Typ::Limited),
    (sys::CAPPROPID_WHITEBALANCE, "White Balance", Typ::Limited),
    (sys::CAPPROPID_WHITEBALANCE, "Auto White Balance", Typ::Auto),
    (sys::CAPPROPID_GAIN, "Gain", Typ::Limited),
    (sys::CAPPROPID_GAIN, "Auto Gain", Typ::Auto),
    (sys::CAPPROPID_BRIGHTNESS, "Brightness", Typ::Limited),
    (sys::CAPPROPID_CONTRAST, "Contrast", Typ::Limited),
    (sys::CAPPROPID_SATURATION, "Saturation", Typ::Limited),
    (sys::CAPPROPID_GAMMA, "Gamma", Typ::Limited),
    (sys::CAPPROPID_HUE, "Hue", Typ::Limited),
    (sys::CAPPROPID_SHARPNESS, "Sharpness", Typ::Limited),
    (
        sys::CAPPROPID_BACKLIGHTCOMP,
        "Backlight Compensation",
        Typ::Limited,
    ),
    (
        sys::CAPPROPID_POWERLINEFREQ,
        "Powerline Frequency",
        Typ::Limited,
    ),
];

pub fn all(
    ctx: sys::CapContext,
    stream: sys::CapStream,
) -> impl IntoIterator<Item = control::Descriptor> {
    ALL.iter()
        .enumerate()
        .filter_map(move |(i, (id, name, typ))| {
            // check whether the control is available and parse its properties
            match typ {
                Typ::Limited => unsafe {
                    let (mut min, mut max, mut default) = (0, 0, 0);
                    match sys::Cap_getPropertyLimits(
                        ctx,
                        stream,
                        *id,
                        &mut min,
                        &mut max,
                        &mut default,
                    ) {
                        sys::CAPRESULT_OK => Some(control::Descriptor {
                            id: i as u32,
                            name: name.to_string(),
                            typ: control::Type::Number {
                                range: (min as f64, max as f64),
                                step: 1.0,
                            },
                            flags: control::Flags::READ | control::Flags::WRITE,
                        }),
                        _ => {
                            // either the property is not available or there was an error
                            None
                        }
                    }
                },
                Typ::Auto => unsafe {
                    // probe to see whether the property is available at all
                    let mut on_off = 0;
                    match sys::Cap_getAutoProperty(ctx, stream, *id, &mut on_off) {
                        sys::CAPRESULT_OK => Some(control::Descriptor {
                            id: i as u32,
                            name: name.to_string(),
                            typ: control::Type::Boolean,
                            flags: control::Flags::READ | control::Flags::WRITE,
                        }),
                        _ => {
                            // either the property is not available or there was an error
                            None
                        }
                    }
                },
            }
        })
}

pub fn read(ctx: sys::CapContext, stream: sys::CapStream, id: u32) -> Result<control::State> {
    let (id, _name, typ) = &ALL[id as usize];
    match typ {
        Typ::Limited => unsafe {
            let mut value = 0;
            match sys::Cap_getProperty(ctx, stream, *id, &mut value) {
                sys::CAPRESULT_OK => Ok(control::State::Number(value as f64)),
                sys::CAPRESULT_PROPERTYNOTSUPPORTED => {
                    Err(Error::new(ErrorKind::Other, "property not available"))
                }
                _ => Err(Error::new(ErrorKind::Other, "unknown error")),
            }
        },
        Typ::Auto => unsafe {
            let mut on_off = 0;
            match sys::Cap_getAutoProperty(ctx, stream, *id, &mut on_off) {
                sys::CAPRESULT_OK => Ok(control::State::Boolean(on_off != 0)),
                sys::CAPRESULT_PROPERTYNOTSUPPORTED => {
                    Err(Error::new(ErrorKind::Other, "property not available"))
                }
                _ => Err(Error::new(ErrorKind::Other, "unknown error")),
            }
        },
    }
}

pub fn write(
    ctx: sys::CapContext,
    stream: sys::CapStream,
    id: u32,
    value: &control::State,
) -> Result<()> {
    let (id, _name, typ) = &ALL[id as usize];
    match typ {
        Typ::Limited => unsafe {
            let value = if let control::State::Number(value) = value {
                value
            } else {
                return Err(Error::new(ErrorKind::Other, "invalid control state"));
            };
            match sys::Cap_setProperty(ctx, stream, *id, *value as i32) {
                sys::CAPRESULT_OK => Ok(()),
                sys::CAPRESULT_PROPERTYNOTSUPPORTED => {
                    Err(Error::new(ErrorKind::Other, "property not available"))
                }
                _ => Err(Error::new(ErrorKind::Other, "unknown error")),
            }
        },
        Typ::Auto => unsafe {
            let mut on_off = if let control::State::Boolean(on) = value {
                if *on {
                    1
                } else {
                    0
                }
            } else {
                return Err(Error::new(ErrorKind::Other, "invalid control state"));
            };
            match sys::Cap_getAutoProperty(ctx, stream, *id, &mut on_off) {
                sys::CAPRESULT_OK => Ok(()),
                sys::CAPRESULT_PROPERTYNOTSUPPORTED => {
                    Err(Error::new(ErrorKind::Other, "property not available"))
                }
                _ => Err(Error::new(ErrorKind::Other, "unknown error")),
            }
        },
    }
}
